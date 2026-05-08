use agent_calc::{
    AssumptionsRequest, CalculusRequest, ComplexRequest, Describe, EvalRequest, FinanceRequest,
    InequalityRequest, IntervalRequest, LinearRequest, MatrixRequest, OptimizeRequest,
    PolynomialRequest, SimplifyRequest, SolveRequest, StatsRequest, SubstituteRequest,
    TraceRequest, UnitRequest, assumptions_schema_json, calculus_schema_json, complex_schema_json,
    finance_schema_json, inequality_schema_json, interval_schema_json, linear_schema_json,
    matrix_schema_json, optimize_schema_json, polynomial_schema_json, schema_json,
    simplify_schema_json, solve_schema_json, stats_schema_json, substitute_schema_json,
    trace_schema_json, units_schema_json,
};
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("describe") => write_json(&Describe::current(), "describe"),
        Some("schema") => schema(&args[1..]),
        Some("eval") => eval(&args[1..]),
        Some("simplify") => simplify(&args[1..]),
        Some("trace") => trace(&args[1..]),
        Some("substitute") => substitute(&args[1..]),
        Some("assumptions") => assumptions(&args[1..]),
        Some("solve") => solve(&args[1..]),
        Some("calculus") => calculus(&args[1..]),
        Some("inequality") => inequality(&args[1..]),
        Some("polynomial") => polynomial(&args[1..]),
        Some("interval") => interval(&args[1..]),
        Some("finance") => finance(&args[1..]),
        Some("units") => units(&args[1..]),
        Some("matrix") => matrix(&args[1..]),
        Some("stats") => stats(&args[1..]),
        Some("optimize") => optimize(&args[1..]),
        Some("linear") => linear(&args[1..]),
        Some("complex") => complex(&args[1..]),
        Some("--help") | Some("-h") | None => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some("--version") => {
            println!("agent-calc {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("agent-calc: unknown command `{other}`");
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn eval(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "eval") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<EvalRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc eval: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "eval")
}

fn simplify(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "simplify") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<SimplifyRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc simplify: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.simplify(), "simplify")
}

fn trace(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "trace") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<TraceRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc trace: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "trace")
}

fn substitute(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "substitute") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<SubstituteRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc substitute: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.substitute(), "substitute")
}

fn assumptions(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "assumptions") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<AssumptionsRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc assumptions: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "assumptions")
}

fn solve(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "solve") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<SolveRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc solve: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "solve")
}

fn calculus(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "calculus") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<CalculusRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc calculus: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "calculus")
}

fn inequality(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "inequality") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<InequalityRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc inequality: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "inequality")
}

fn polynomial(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "polynomial") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<PolynomialRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc polynomial: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "polynomial")
}

fn interval(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "interval") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<IntervalRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc interval: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "interval")
}

fn finance(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "finance") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<FinanceRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc finance: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "finance")
}

fn units(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "units") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<UnitRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc units: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "units")
}

fn matrix(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "matrix") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<MatrixRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc matrix: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "matrix")
}

fn stats(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "stats") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<StatsRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc stats: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "stats")
}

fn optimize(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "optimize") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<OptimizeRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc optimize: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "optimize")
}

fn linear(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "linear") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<LinearRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc linear: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "linear")
}

fn complex(args: &[String]) -> ExitCode {
    let input = match read_json_input(args, "complex") {
        Ok(input) => input,
        Err(code) => return code,
    };

    let request = match serde_json::from_str::<ComplexRequest>(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("agent-calc complex: invalid request JSON: {e}");
            return ExitCode::from(2);
        }
    };

    write_json(&request.evaluate(), "complex")
}

fn schema(args: &[String]) -> ExitCode {
    match args {
        [] => write_json(&schema_json(), "schema"),
        [domain] if domain == "rational" => write_json(&schema_json(), "schema"),
        [domain] if domain == "units" => write_json(&units_schema_json(), "schema"),
        [domain] if domain == "matrix" => write_json(&matrix_schema_json(), "schema"),
        [domain] if domain == "stats" => write_json(&stats_schema_json(), "schema"),
        [domain] if domain == "optimize" => write_json(&optimize_schema_json(), "schema"),
        [domain] if domain == "linear" => write_json(&linear_schema_json(), "schema"),
        [domain] if domain == "complex" => write_json(&complex_schema_json(), "schema"),
        [domain] if domain == "simplify" => write_json(&simplify_schema_json(), "schema"),
        [domain] if domain == "trace" => write_json(&trace_schema_json(), "schema"),
        [domain] if domain == "substitute" => write_json(&substitute_schema_json(), "schema"),
        [domain] if domain == "assumptions" => write_json(&assumptions_schema_json(), "schema"),
        [domain] if domain == "solve" => write_json(&solve_schema_json(), "schema"),
        [domain] if domain == "calculus" => write_json(&calculus_schema_json(), "schema"),
        [domain] if domain == "inequality" => write_json(&inequality_schema_json(), "schema"),
        [domain] if domain == "polynomial" => write_json(&polynomial_schema_json(), "schema"),
        [domain] if domain == "interval" => write_json(&interval_schema_json(), "schema"),
        [domain] if domain == "finance" => write_json(&finance_schema_json(), "schema"),
        _ => {
            eprintln!(
                "agent-calc schema: expected no args, `rational`, `simplify`, `trace`, `substitute`, `assumptions`, `solve`, `calculus`, `inequality`, `polynomial`, `interval`, `finance`, `units`, `matrix`, `stats`, `optimize`, `linear`, or `complex`"
            );
            ExitCode::from(2)
        }
    }
}

fn read_json_input(args: &[String], ctx: &str) -> Result<String, ExitCode> {
    match args {
        [] => {
            let mut input = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut input) {
                eprintln!("agent-calc {ctx}: cannot read stdin: {e}");
                return Err(ExitCode::FAILURE);
            }
            Ok(input)
        }
        [path] => match std::fs::read_to_string(path) {
            Ok(input) => Ok(input),
            Err(e) => {
                eprintln!("agent-calc {ctx}: cannot read {path}: {e}");
                Err(ExitCode::FAILURE)
            }
        },
        _ => {
            eprintln!("agent-calc {ctx}: expected zero args for stdin or one JSON file path");
            Err(ExitCode::from(2))
        }
    }
}

fn write_json<T: serde::Serialize>(value: &T, ctx: &str) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("agent-calc {ctx}: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "agent-calc — AI-native exact computation kernel

Usage:
  agent-calc describe          emit executable contract
  agent-calc schema [domain]   emit request JSON Schema (rational|simplify|trace|substitute|assumptions|solve|calculus|inequality|polynomial|interval|finance|units|matrix|stats|optimize|linear|complex)
  agent-calc eval [file]       evaluate request JSON from file or stdin
  agent-calc simplify [file]   simplify symbolic expression JSON from file or stdin
  agent-calc trace [file]      evaluate or simplify expression JSON with deterministic trace steps
  agent-calc substitute [file] substitute exact bindings into expression JSON from file or stdin
  agent-calc assumptions [file] validate and query symbolic assumption context JSON from file or stdin
  agent-calc solve [file]      solve exact affine equation JSON from file or stdin
  agent-calc calculus [file]   evaluate symbolic calculus request JSON from file or stdin
  agent-calc inequality [file] solve exact affine inequality JSON from file or stdin
  agent-calc polynomial [file] evaluate exact polynomial request JSON from file or stdin
  agent-calc interval [file]   evaluate exact interval request JSON from file or stdin
  agent-calc finance [file]    evaluate exact time-value finance request JSON from file or stdin
  agent-calc units [file]      evaluate unit request JSON from file or stdin
  agent-calc matrix [file]     evaluate matrix request JSON from file or stdin
  agent-calc stats [file]      evaluate statistics request JSON from file or stdin
  agent-calc optimize [file]   evaluate optimization request JSON from file or stdin
  agent-calc linear [file]     evaluate linear-program request JSON from file or stdin
  agent-calc complex [file]    evaluate complex-number request JSON from file or stdin
  agent-calc --version
  agent-calc --help
"
    );
}
