//! calcd — the kernel as a persistent process.
//!
//! Measured before this existed: a solve costs 0.26ms, and spawning the CLI to
//! perform it costs 1.36ms. The process was five times the work. That is the
//! whole argument for a server — not that the arithmetic is slow, but that
//! `fork+exec` around it is.
//!
//! Which is also why the interesting endpoint is `/v1/batch` and not `/v1/eval`.
//! A caller with 212 solves to do — one stress test — must be able to ask once.
//! Paying a network round trip per solve would be far worse than the CLI ever
//! was, and a batch endpoint that ran them serially would waste 63 idle cores.
//!
//! Every request goes through `execute_tool_json`, the same function the CLI and
//! the WebAssembly build call. Not a port of it, not a second implementation —
//! the same code. Three transports over one kernel is the only arrangement in
//! which "you get the same answer everywhere" is a fact rather than a promise.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use agent_calc::{ALLOWED_TOOL_COMMANDS, CONTRACT_VERSION, execute_tool_json};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// One unit of work: the same pair the CLI takes as `agent-calc <command> <file>`.
#[derive(Deserialize)]
struct Call {
    command: String,
    /// Accepted as a JSON value rather than a string so callers do not have to
    /// double-encode. It is re-serialised before the kernel sees it, which also
    /// means the kernel is handed canonical JSON regardless of how it arrived.
    request: Value,
}

#[derive(Deserialize)]
struct BatchBody {
    calls: Vec<Call>,
}

#[derive(Serialize)]
struct BatchOut {
    contract_version: &'static str,
    count: usize,
    /// Wall-clock for the whole batch, so a caller can see the difference between
    /// compute and the network without guessing.
    elapsed_ms: f64,
    responses: Vec<Value>,
}

struct Metrics {
    calls: AtomicU64,
    batches: AtomicU64,
}

/// Bounded so one caller cannot ask for a million solves and take the process
/// with them. A stress test is 212; a survival map is 121.
const MAX_CALLS: usize = 4096;
const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

fn run_one(call: &Call) -> Value {
    let input = call.request.to_string();
    let raw = execute_tool_json(&call.command, &input);
    // execute_tool_json always returns JSON, including for its own errors; if it
    // ever did not, the batch must still be well formed rather than collapse.
    serde_json::from_str(&raw).unwrap_or_else(|_| {
        json!({
            "status": "rejected",
            "contract_version": CONTRACT_VERSION,
            "command": call.command,
            "error": { "code": "kernel_output_unreadable", "reason": "kernel did not return JSON" }
        })
    })
}

async fn batch(
    State(m): State<Arc<Metrics>>,
    Json(body): Json<BatchBody>,
) -> (StatusCode, Json<Value>) {
    if body.calls.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "error": { "code": "empty_batch", "reason": "calls must not be empty" } }),
            ),
        );
    }
    if body.calls.len() > MAX_CALLS {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "error": { "code": "batch_too_large", "reason": format!("at most {MAX_CALLS} calls") }
            })),
        );
    }

    let started = Instant::now();
    // The 121 cells of a survival map do not depend on each other, and neither do
    // the eight levers of a tornado. Order is preserved because par_iter collects
    // in order — a caller matching responses to calls by index must be able to.
    let responses: Vec<Value> = body.calls.par_iter().map(run_one).collect();
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    m.calls
        .fetch_add(body.calls.len() as u64, Ordering::Relaxed);
    m.batches.fetch_add(1, Ordering::Relaxed);

    let out = BatchOut {
        contract_version: CONTRACT_VERSION,
        count: responses.len(),
        elapsed_ms,
        responses,
    };
    (StatusCode::OK, Json(serde_json::to_value(out).unwrap()))
}

/// The single-call form, for callers with one thing to ask. It is the batch path
/// with a batch of one rather than a second code path, so the two can never
/// disagree about what a command means.
async fn one(State(m): State<Arc<Metrics>>, Json(call): Json<Call>) -> Json<Value> {
    m.calls.fetch_add(1, Ordering::Relaxed);
    Json(run_one(&call))
}

async fn health(State(m): State<Arc<Metrics>>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "contract_version": CONTRACT_VERSION,
        "commands": ALLOWED_TOOL_COMMANDS,
        "threads": rayon::current_num_threads(),
        "served": { "calls": m.calls.load(Ordering::Relaxed), "batches": m.batches.load(Ordering::Relaxed) },
    }))
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("CALCD_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);
    // Loopback by default. This process will happily solve anything it is sent,
    // so exposing it is a decision somebody should have to make on purpose.
    let host = std::env::var("CALCD_HOST").unwrap_or_else(|_| "127.0.0.1".into());

    let metrics = Arc::new(Metrics {
        calls: AtomicU64::new(0),
        batches: AtomicU64::new(0),
    });

    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/call", post(one))
        .route("/v1/batch", post(batch))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(metrics);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("cannot bind {addr}: {e}"));
    println!(
        "calcd {CONTRACT_VERSION} on http://{addr} · {} threads",
        rayon::current_num_threads()
    );
    axum::serve(listener, app).await.unwrap();
}
