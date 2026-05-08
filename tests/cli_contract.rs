use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_agent-calc")
}

#[test]
fn describe_emits_contract_json() {
    let output = Command::new(bin()).arg("describe").output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["name"], "agent-calc");
    assert_eq!(json["contract_version"], "calc1/0.1.0");
    assert!(json["invariants"].as_array().unwrap().len() >= 4);
}

#[test]
fn schema_emits_request_schema() {
    let output = Command::new(bin()).arg("schema").output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(json["$defs"]["Rational"]["type"], "object");
}

#[test]
fn schema_units_emits_unit_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("units")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 units request");
    assert_eq!(
        json["$defs"]["Dimension"]["enum"],
        serde_json::json!(["length", "mass", "time", "velocity"])
    );
}

#[test]
fn schema_matrix_emits_matrix_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("matrix")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 matrix request");
    assert_eq!(json["$defs"]["Matrix"]["type"], "object");
}

#[test]
fn schema_stats_emits_stats_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("stats")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 stats request");
    assert_eq!(json["$defs"]["NormalCdf"]["type"], "object");
}

#[test]
fn schema_optimize_emits_optimize_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("optimize")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 optimize request");
    assert_eq!(json["$defs"]["Quadratic"]["type"], "object");
}

#[test]
fn schema_linear_emits_linear_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("linear")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 linear request");
    assert_eq!(json["$defs"]["LinearConstraint"]["type"], "object");
}

#[test]
fn schema_complex_emits_complex_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("complex")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 complex request");
    assert_eq!(json["$defs"]["Complex"]["type"], "object");
}

#[test]
fn schema_simplify_emits_symbolic_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("simplify")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 simplify request");
    assert_eq!(json["$defs"]["Symbol"]["type"], "object");
}

#[test]
fn schema_trace_emits_trace_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("trace")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 trace request");
    assert_eq!(
        json["properties"]["intent"]["enum"],
        serde_json::json!(["eval", "simplify"])
    );
    assert_eq!(json["properties"]["decimal_places"]["default"], 12);
}

#[test]
fn schema_substitute_emits_binding_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("substitute")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 substitute request");
    assert_eq!(
        json["properties"]["bindings"]["propertyNames"]["pattern"],
        "^[A-Za-z_][A-Za-z0-9_]*$"
    );
}

#[test]
fn schema_assumptions_emits_assumptions_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("assumptions")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 assumptions request");
    assert_eq!(
        json["$defs"]["Domain"]["enum"],
        serde_json::json!([
            "rational",
            "integer",
            "nonzero",
            "positive",
            "negative",
            "nonnegative",
            "nonpositive"
        ])
    );
}

#[test]
fn schema_solve_emits_solve_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("solve")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 solve request");
    assert_eq!(json["$defs"]["Equation"]["type"], "object");
}

#[test]
fn schema_calculus_emits_calculus_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("calculus")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 calculus request");
    assert_eq!(json["properties"]["intent"]["const"], "derivative");
}

#[test]
fn schema_inequality_emits_inequality_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("inequality")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 inequality request");
    assert_eq!(
        json["$defs"]["Relation"]["enum"],
        serde_json::json!(["lt", "lte", "gt", "gte", "eq", "neq"])
    );
}

#[test]
fn schema_polynomial_emits_polynomial_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("polynomial")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 polynomial request");
    assert_eq!(json["$defs"]["Polynomial"]["type"], "object");
}

#[test]
fn schema_interval_emits_interval_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("interval")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 interval request");
    assert_eq!(json["$defs"]["Interval"]["type"], "object");
}

#[test]
fn schema_finance_emits_finance_request_schema() {
    let output = Command::new(bin())
        .arg("schema")
        .arg("finance")
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 finance request");
    assert_eq!(
        json["$defs"]["NetPresentValue"]["properties"]["cash_flows"]["minItems"],
        1
    );
}

#[test]
fn schema_rational_and_unknown_domain_have_stable_behavior() {
    let rational = Command::new(bin())
        .arg("schema")
        .arg("rational")
        .output()
        .unwrap();
    assert!(rational.status.success());
    let json: serde_json::Value = serde_json::from_slice(&rational.stdout).unwrap();
    assert_eq!(json["title"], "agent-calc calc1 request");

    let unknown = Command::new(bin())
        .arg("schema")
        .arg("not-a-domain")
        .output()
        .unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        String::from_utf8(unknown.stderr)
            .unwrap()
            .contains("expected no args")
    );
}

#[test]
fn eval_reads_stdin_and_returns_exact_result() {
    let mut child = Command::new(bin())
        .arg("eval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "add",
                    "left": {"kind": "rational", "numerator": "1", "denominator": "3"},
                    "right": {"kind": "rational", "numerator": "1", "denominator": "6"}
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "solved");
    assert_eq!(json["exact"]["display"], "1/2");
}

#[test]
fn eval_preserves_arbitrary_precision() {
    let mut child = Command::new(bin())
        .arg("eval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "add",
                    "left": {
                        "kind": "integer",
                        "value": "170141183460469231731687303715884105728"
                    },
                    "right": {
                        "kind": "integer",
                        "value": "1"
                    }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["exact"]["display"],
        "170141183460469231731687303715884105729"
    );
}

#[test]
fn eval_supports_exact_integer_power() {
    let mut child = Command::new(bin())
        .arg("eval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "pow",
                    "base": {"kind": "rational", "numerator": "2", "denominator": "3"},
                    "exponent": -2
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "solved");
    assert_eq!(json["exact"]["display"], "9/4");
    assert_eq!(json["decimal"], "2.250000000000");
}

#[test]
fn eval_rejects_unbound_symbol() {
    let mut child = Command::new(bin())
        .arg("eval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "symbol",
                    "name": "x"
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "unbound symbol `x`");
}

#[test]
fn simplify_reads_stdin_and_applies_identity() {
    let mut child = Command::new(bin())
        .arg("simplify")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "mul",
                    "left": { "kind": "symbol", "name": "x" },
                    "right": { "kind": "integer", "value": "1" }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "simplified");
    assert_eq!(
        json["expr"],
        serde_json::json!({"kind": "symbol", "name": "x"})
    );
    assert_eq!(
        json["checks"][0],
        serde_json::json!({"name": "symbolic_identities_applied", "passed": true})
    );
}

#[test]
fn simplify_rejects_invalid_symbol_name() {
    let mut child = Command::new(bin())
        .arg("simplify")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "symbol",
                    "name": "1x"
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "invalid symbol name `1x`");
}

#[test]
fn trace_reads_stdin_and_evaluates_with_steps() {
    let mut child = Command::new(bin())
        .arg("trace")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "eval",
                "decimal_places": 3,
                "expr": {
                    "kind": "add",
                    "left": {"kind": "integer", "value": "2"},
                    "right": {"kind": "integer", "value": "3"}
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "evaluated");
    assert_eq!(json["exact"]["display"], "5");
    assert_eq!(json["decimal"], "5.000");
    assert_eq!(json["trace"][0]["step"], 1);
    assert_eq!(json["trace"][2]["rule"], "eval.add");
    assert_eq!(json["trace"][2]["output"]["exact"]["display"], "5");
}

#[test]
fn trace_reads_stdin_and_simplifies_with_steps() {
    let mut child = Command::new(bin())
        .arg("trace")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "simplify",
                "expr": {
                    "kind": "add",
                    "left": { "kind": "symbol", "name": "x" },
                    "right": { "kind": "integer", "value": "0" }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "simplified");
    assert_eq!(
        json["expr"],
        serde_json::json!({"kind": "symbol", "name": "x"})
    );
    assert_eq!(json["trace"][2]["rule"], "simplify.add_zero_right");
    assert_eq!(
        json["trace"][2]["output"]["expr"],
        serde_json::json!({"kind": "symbol", "name": "x"})
    );
}

#[test]
fn trace_reports_eval_error_with_trace() {
    let mut child = Command::new(bin())
        .arg("trace")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "eval",
                "expr": { "kind": "symbol", "name": "x" }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "unbound symbol `x`");
    assert_eq!(json["trace"][0]["rule"], "eval.symbol");
    assert_eq!(json["trace"][0]["output"]["kind"], "error");
}

#[test]
fn substitute_reads_stdin_and_applies_binding() {
    let mut child = Command::new(bin())
        .arg("substitute")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": {
                    "kind": "mul",
                    "left": { "kind": "symbol", "name": "x" },
                    "right": { "kind": "integer", "value": "1" }
                },
                "bindings": {
                    "x": { "kind": "rational", "numerator": "2", "denominator": "3" }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "substituted");
    assert_eq!(
        json["expr"],
        serde_json::json!({"kind": "rational", "numerator": "2", "denominator": "3"})
    );
    assert_eq!(
        json["checks"][0],
        serde_json::json!({"name": "all_symbols_bound", "passed": true})
    );
}

#[test]
fn substitute_rejects_missing_binding() {
    let mut child = Command::new(bin())
        .arg("substitute")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "expr": { "kind": "symbol", "name": "x" },
                "bindings": {}
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "missing binding for symbol `x`");
}

#[test]
fn assumptions_reads_stdin_and_derives_bounds() {
    let mut child = Command::new(bin())
        .arg("assumptions")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "bounds",
                "symbol": "x",
                "assumptions": [
                    {"kind": "domain", "symbol": "x", "domain": "positive"},
                    {"kind": "compare", "symbol": "x", "op": "lte", "value": {"kind": "integer", "value": "5"}},
                    {"kind": "compare", "symbol": "x", "op": "neq", "value": {"kind": "integer", "value": "3"}}
                ]
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "bounds");
    assert_eq!(json["symbol"], "x");
    assert_eq!(json["lower"]["value"]["display"], "0");
    assert_eq!(json["lower"]["inclusive"], false);
    assert_eq!(json["upper"]["value"]["display"], "5");
    assert_eq!(json["upper"]["inclusive"], true);
    assert_eq!(json["excluded"][0]["display"], "3");
}

#[test]
fn assumptions_reads_stdin_and_answers_entailment() {
    let mut child = Command::new(bin())
        .arg("assumptions")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "entails",
                "assumptions": [
                    {"kind": "compare", "symbol": "x", "op": "gte", "value": {"kind": "integer", "value": "2"}}
                ],
                "query": {"kind": "domain", "symbol": "x", "domain": "positive"}
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "entailment");
    assert_eq!(json["entailed"], true);
    assert!(!json["evidence"].as_array().unwrap().is_empty());
}

#[test]
fn assumptions_rejects_contradictory_context() {
    let mut child = Command::new(bin())
        .arg("assumptions")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "validate",
                "assumptions": [
                    {"kind": "compare", "symbol": "x", "op": "gt", "value": {"kind": "integer", "value": "2"}},
                    {"kind": "compare", "symbol": "x", "op": "lte", "value": {"kind": "integer", "value": "2"}}
                ]
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "assumptions for `x` have empty bounds");
}

#[test]
fn solve_reads_stdin_and_solves_affine_equation() {
    let mut child = Command::new(bin())
        .arg("solve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "variable": "x",
                "equation": {
                    "left": {
                        "kind": "add",
                        "left": {
                            "kind": "mul",
                            "left": {"kind": "integer", "value": "2"},
                            "right": {"kind": "symbol", "name": "x"}
                        },
                        "right": {"kind": "integer", "value": "3"}
                    },
                    "right": {"kind": "integer", "value": "7"}
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "solutions");
    assert_eq!(json["variable"], "x");
    assert_eq!(json["solutions"][0]["display"], "2");
}

#[test]
fn solve_reports_degenerate_and_nonlinear_cases() {
    let mut no_solution = Command::new(bin())
        .arg("solve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    no_solution
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "variable": "x",
                "equation": {
                    "left": {
                        "kind": "add",
                        "left": {"kind": "symbol", "name": "x"},
                        "right": {"kind": "integer", "value": "1"}
                    },
                    "right": {
                        "kind": "add",
                        "left": {"kind": "symbol", "name": "x"},
                        "right": {"kind": "integer", "value": "2"}
                    }
                }
            }"#,
        )
        .unwrap();
    let output = no_solution.wait_with_output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "no_solution");

    let mut nonlinear = Command::new(bin())
        .arg("solve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    nonlinear
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "variable": "x",
                "equation": {
                    "left": {
                        "kind": "pow",
                        "base": {"kind": "symbol", "name": "x"},
                        "exponent": 2
                    },
                    "right": {"kind": "integer", "value": "4"}
                }
            }"#,
        )
        .unwrap();
    let output = nonlinear.wait_with_output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "nonlinear power involving `x`");
}

#[test]
fn calculus_reads_stdin_and_differentiates_polynomial_shape() {
    let mut child = Command::new(bin())
        .arg("calculus")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "derivative",
                "variable": "x",
                "expr": {
                    "kind": "add",
                    "left": {
                        "kind": "pow",
                        "base": {"kind": "symbol", "name": "x"},
                        "exponent": 3
                    },
                    "right": {
                        "kind": "mul",
                        "left": {"kind": "integer", "value": "2"},
                        "right": {"kind": "symbol", "name": "x"}
                    }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "derivative");
    assert_eq!(json["variable"], "x");
    assert_eq!(
        json["checks"][0]["name"],
        "symbolic_derivative_rule_applied"
    );
}

#[test]
fn calculus_rejects_invalid_variable_name() {
    let mut child = Command::new(bin())
        .arg("calculus")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "derivative",
                "variable": "1x",
                "expr": {"kind": "symbol", "name": "x"}
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "invalid symbol name `1x`");
}

#[test]
fn inequality_reads_stdin_and_solves_affine_bound() {
    let mut child = Command::new(bin())
        .arg("inequality")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "variable": "x",
                "inequality": {
                    "left": {
                        "kind": "add",
                        "left": {
                            "kind": "mul",
                            "left": {"kind": "integer", "value": "2"},
                            "right": {"kind": "symbol", "name": "x"}
                        },
                        "right": {"kind": "integer", "value": "3"}
                    },
                    "relation": "lt",
                    "right": {"kind": "integer", "value": "7"}
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "solution_set");
    assert_eq!(json["variable"], "x");
    assert_eq!(json["set"]["kind"], "interval");
    assert_eq!(json["set"]["upper"]["value"]["display"], "2");
    assert_eq!(json["set"]["upper"]["inclusive"], false);
}

#[test]
fn inequality_reports_nonlinear_error() {
    let mut child = Command::new(bin())
        .arg("inequality")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "variable": "x",
                "inequality": {
                    "left": {
                        "kind": "mul",
                        "left": {"kind": "symbol", "name": "x"},
                        "right": {"kind": "symbol", "name": "x"}
                    },
                    "relation": "lt",
                    "right": {"kind": "integer", "value": "1"}
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "nonlinear term involving `x`");
}

#[test]
fn polynomial_reads_stdin_and_multiplies() {
    let mut child = Command::new(bin())
        .arg("polynomial")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "mul",
                "left": {
                    "variable": "x",
                    "coefficients": [
                        { "kind": "integer", "value": "1" },
                        { "kind": "integer", "value": "1" }
                    ]
                },
                "right": {
                    "variable": "x",
                    "coefficients": [
                        { "kind": "integer", "value": "1" },
                        { "kind": "integer", "value": "-1" }
                    ]
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "polynomial");
    assert_eq!(json["variable"], "x");
    assert_eq!(
        json["coefficients"],
        serde_json::json!([
            {"numerator": "1", "denominator": "1", "display": "1"},
            {"numerator": "0", "denominator": "1", "display": "0"},
            {"numerator": "-1", "denominator": "1", "display": "-1"}
        ])
    );
}

#[test]
fn polynomial_reads_stdin_and_evaluates() {
    let mut child = Command::new(bin())
        .arg("polynomial")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "evaluate",
                "polynomial": {
                    "variable": "x",
                    "coefficients": [
                        { "kind": "integer", "value": "1" },
                        { "kind": "integer", "value": "2" },
                        { "kind": "integer", "value": "3" }
                    ]
                },
                "at": { "kind": "integer", "value": "2" }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "value");
    assert_eq!(json["exact"]["display"], "17");
}

#[test]
fn polynomial_reads_stdin_and_solves_quadratic() {
    let mut child = Command::new(bin())
        .arg("polynomial")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "polynomial": {
                    "variable": "x",
                    "coefficients": [
                        { "kind": "integer", "value": "6" },
                        { "kind": "integer", "value": "-5" },
                        { "kind": "integer", "value": "1" }
                    ]
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "roots");
    assert_eq!(json["variable"], "x");
    assert_eq!(
        json["roots"],
        serde_json::json!([
            {"numerator": "2", "denominator": "1", "display": "2"},
            {"numerator": "3", "denominator": "1", "display": "3"}
        ])
    );
}

#[test]
fn polynomial_rejects_irrational_quadratic_roots() {
    let mut child = Command::new(bin())
        .arg("polynomial")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve",
                "polynomial": {
                    "variable": "x",
                    "coefficients": [
                        { "kind": "integer", "value": "-2" },
                        { "kind": "integer", "value": "0" },
                        { "kind": "integer", "value": "1" }
                    ]
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "quadratic roots are irrational");
}

#[test]
fn polynomial_rejects_variable_mismatch() {
    let mut child = Command::new(bin())
        .arg("polynomial")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "add",
                "left": {
                    "variable": "x",
                    "coefficients": [{ "kind": "integer", "value": "1" }]
                },
                "right": {
                    "variable": "y",
                    "coefficients": [{ "kind": "integer", "value": "1" }]
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "variable mismatch `x` != `y`");
}

#[test]
fn interval_reads_stdin_and_multiplies() {
    let mut child = Command::new(bin())
        .arg("interval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "mul",
                "left": {
                    "lower": { "kind": "integer", "value": "-2" },
                    "upper": { "kind": "integer", "value": "3" }
                },
                "right": {
                    "lower": { "kind": "integer", "value": "4" },
                    "upper": { "kind": "integer", "value": "5" }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "interval");
    assert_eq!(json["lower"]["display"], "-10");
    assert_eq!(json["upper"]["display"], "15");
}

#[test]
fn interval_reads_stdin_and_computes_polynomial_range() {
    let mut child = Command::new(bin())
        .arg("interval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "polynomial_range",
                "polynomial": {
                    "variable": "x",
                    "coefficients": [
                        { "kind": "integer", "value": "0" },
                        { "kind": "integer", "value": "-2" },
                        { "kind": "integer", "value": "1" }
                    ]
                },
                "domain": {
                    "lower": { "kind": "integer", "value": "0" },
                    "upper": { "kind": "integer", "value": "3" }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "interval");
    assert_eq!(json["lower"]["display"], "-1");
    assert_eq!(json["upper"]["display"], "3");
}

#[test]
fn interval_rejects_division_by_zero_interval() {
    let mut child = Command::new(bin())
        .arg("interval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "div",
                "left": {
                    "lower": { "kind": "integer", "value": "1" },
                    "upper": { "kind": "integer", "value": "2" }
                },
                "right": {
                    "lower": { "kind": "integer", "value": "-1" },
                    "upper": { "kind": "integer", "value": "1" }
                }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "division interval must not contain zero");
}

#[test]
fn eval_reads_file_and_rejects_extra_args() {
    let mut path = std::env::temp_dir();
    path.push(format!("agent-calc-{}-request.json", std::process::id()));
    std::fs::write(
        &path,
        r#"{
            "expr": {
                "kind": "mul",
                "left": {"kind": "integer", "value": "7"},
                "right": {"kind": "rational", "numerator": "3", "denominator": "2"}
            }
        }"#,
    )
    .unwrap();

    let output = Command::new(bin()).arg("eval").arg(&path).output().unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["exact"]["display"], "21/2");

    let rejected = Command::new(bin())
        .arg("eval")
        .arg("one")
        .arg("two")
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(!rejected.stderr.is_empty());
}

#[test]
fn units_reads_stdin_and_converts_length() {
    let mut child = Command::new(bin())
        .arg("units")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "convert",
                "quantity": {
                    "dimension": "length",
                    "value": 1.5,
                    "unit": "km"
                },
                "to": "m"
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "solved");
    assert_eq!(json["dimension"], "length");
    assert_eq!(json["value"], 1500.0);
    assert_eq!(json["exactness"], "approximate_f64");
}

#[test]
fn units_rejects_dimension_mismatch() {
    let mut child = Command::new(bin())
        .arg("units")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "add",
                "left": {"dimension": "length", "value": 1.0, "unit": "m"},
                "right": {"dimension": "time", "value": 1.0, "unit": "s"},
                "to": "m"
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert!(
        json["reason"]
            .as_str()
            .unwrap()
            .contains("dimension mismatch")
    );
}

#[test]
fn finance_reads_stdin_and_computes_future_value() {
    let mut child = Command::new(bin())
        .arg("finance")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "future_value",
                "present_value": { "kind": "integer", "value": "100" },
                "rate": { "kind": "rational", "numerator": "1", "denominator": "10" },
                "periods": 2,
                "decimal_places": 2
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "value");
    assert_eq!(json["exact"]["display"], "121");
    assert_eq!(json["decimal"], "121.00");
    assert_eq!(json["checks"][0]["name"], "exact_rational_cashflow_math");
}

#[test]
fn finance_reads_stdin_and_computes_npv() {
    let mut child = Command::new(bin())
        .arg("finance")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "net_present_value",
                "rate": { "kind": "rational", "numerator": "1", "denominator": "10" },
                "cash_flows": [
                    { "kind": "integer", "value": "-100" },
                    { "kind": "integer", "value": "55" },
                    { "kind": "rational", "numerator": "121", "denominator": "2" }
                ],
                "decimal_places": 2
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "value");
    assert_eq!(json["exact"]["display"], "0");
    assert_eq!(json["decimal"], "0.00");
}

#[test]
fn finance_rejects_undefined_discount_factor() {
    let mut child = Command::new(bin())
        .arg("finance")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "discount_factor",
                "rate": { "kind": "integer", "value": "-1" },
                "period": 1
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(
        json["reason"],
        "discount factor undefined when 1 + rate is zero"
    );
}

#[test]
fn matrix_reads_stdin_and_multiplies() {
    let mut child = Command::new(bin())
        .arg("matrix")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "mul",
                "left": { "rows": 2, "cols": 2, "data": [1, 2, 3, 4] },
                "right": { "rows": 2, "cols": 2, "data": [5, 6, 7, 8] }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "matrix");
    assert_eq!(json["rows"], 2);
    assert_eq!(json["cols"], 2);
    assert_eq!(json["data"], serde_json::json!([19.0, 22.0, 43.0, 50.0]));
}

#[test]
fn matrix_rejects_bad_shape() {
    let mut child = Command::new(bin())
        .arg("matrix")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "mul",
                "left": { "rows": 1, "cols": 2, "data": [1, 2] },
                "right": { "rows": 1, "cols": 2, "data": [3, 4] }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert!(json["reason"].as_str().unwrap().contains("shape mismatch"));
}

#[test]
fn stats_reads_stdin_and_computes_normal_cdf() {
    let mut child = Command::new(bin())
        .arg("stats")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "normal_cdf",
                "mean": 0,
                "std_dev": 1,
                "x": 0
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "probability");
    assert_eq!(json["value"], 0.5);
    assert_eq!(json["exactness"], "approximate_f64");
}

#[test]
fn stats_rejects_invalid_distribution_parameters() {
    let mut child = Command::new(bin())
        .arg("stats")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "normal_quantile",
                "mean": 0,
                "std_dev": 1,
                "p": 1
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "p must be in (0, 1)");
}

#[test]
fn optimize_reads_stdin_and_minimizes_quadratic() {
    let mut child = Command::new(bin())
        .arg("optimize")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "minimize_1d",
                "objective": { "kind": "quadratic", "a": 1, "b": -6, "c": 9 },
                "lower": -10,
                "upper": 10,
                "abs_tolerance": 1e-10
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "optimum");
    assert!((json["minimizer"].as_f64().unwrap() - 3.0).abs() < 1e-6);
    assert!(json["minimum"].as_f64().unwrap().abs() < 1e-6);
}

#[test]
fn optimize_rejects_bad_bounds() {
    let mut child = Command::new(bin())
        .arg("optimize")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "minimize_1d",
                "objective": { "kind": "quadratic", "a": 1, "b": 0, "c": 0 },
                "lower": 10,
                "upper": 10
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "lower must be less than upper");
}

#[test]
fn linear_reads_stdin_and_solves_resource_allocation() {
    let mut child = Command::new(bin())
        .arg("linear")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve_lp",
                "direction": "maximize",
                "objective": [10, 11],
                "variables": [{"lower": 0}, {"lower": 0}],
                "constraints": [
                    {"coefficients": [1, 2], "relation": "le", "rhs": 5},
                    {"coefficients": [1, 1], "relation": "le", "rhs": 3}
                ]
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "optimal");
    assert!((json["objective_value"].as_f64().unwrap() - 32.0).abs() < 1e-8);
    let values = json["variables"].as_array().unwrap();
    assert!((values[0].as_f64().unwrap() - 1.0).abs() < 1e-8);
    assert!((values[1].as_f64().unwrap() - 2.0).abs() < 1e-8);
}

#[test]
fn linear_rejects_bad_constraint_shape() {
    let mut child = Command::new(bin())
        .arg("linear")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "solve_lp",
                "direction": "maximize",
                "objective": [1, 1],
                "constraints": [
                    {"coefficients": [1], "relation": "le", "rhs": 5}
                ]
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert!(
        json["reason"]
            .as_str()
            .unwrap()
            .contains("constraint coefficients length mismatch")
    );
}

#[test]
fn complex_reads_stdin_and_multiplies() {
    let mut child = Command::new(bin())
        .arg("complex")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "mul",
                "left": { "re": 1, "im": 2 },
                "right": { "re": 3, "im": 4 }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "complex");
    assert_eq!(json["re"], -5.0);
    assert_eq!(json["im"], 10.0);
}

#[test]
fn complex_rejects_division_by_zero() {
    let mut child = Command::new(bin())
        .arg("complex")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{
                "intent": "div",
                "left": { "re": 1, "im": 0 },
                "right": { "re": 0, "im": 0 }
            }"#,
        )
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "complex division by zero");
}

#[test]
fn help_and_unknown_commands_have_stable_exit_behavior() {
    let help = Command::new(bin()).arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8(help.stderr).unwrap().contains("Usage:"));

    let unknown = Command::new(bin()).arg("nope").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        String::from_utf8(unknown.stderr)
            .unwrap()
            .contains("unknown command")
    );
}

// --- #17 schema consistency regression tests ---

#[test]
fn matrix_solve_accepts_matrix_input_for_constants() {
    // Regression: constants was Vec<f64>; now MatrixInput like all other matrix fields.
    let input = serde_json::json!({
        "intent": "solve",
        "coefficients": { "rows": 2, "cols": 2, "data": [2.0, 1.0, 1.0, -1.0] },
        "constants":    { "rows": 2, "cols": 1, "data": [5.0, 1.0] }
    });
    let mut child = Command::new(bin())
        .arg("matrix")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "vector");
    let data = json["data"].as_array().unwrap();
    assert!((data[0].as_f64().unwrap() - 2.0).abs() < 1e-10);
    assert!((data[1].as_f64().unwrap() - 1.0).abs() < 1e-10);
}

#[test]
fn stats_describe_sample_accepts_data_alias() {
    // Regression: describe_sample only accepted `values`; `data` is now a valid alias.
    let input = serde_json::json!({ "intent": "describe_sample", "data": [1.0, 2.0, 3.0] });
    let mut child = Command::new(bin())
        .arg("stats")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "sample_summary");
    assert_eq!(json["n"], 3);
    assert!((json["mean"].as_f64().unwrap() - 2.0).abs() < 1e-12);
}

#[test]
fn complex_abs_accepts_modulus_alias() {
    // Regression: modulus is now an alias for abs to avoid collision with the
    // upcoming expression AST `abs` node.
    let input = serde_json::json!({ "intent": "modulus", "value": { "re": 3.0, "im": 4.0 } });
    let mut child = Command::new(bin())
        .arg("complex")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "scalar");
    assert!((json["value"].as_f64().unwrap() - 5.0).abs() < 1e-12);
}
