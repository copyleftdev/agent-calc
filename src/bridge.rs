use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    AssumptionsRequest, CONTRACT_VERSION, CalculusRequest, ComplexRequest, EvalRequest,
    FinanceRequest, InequalityRequest, IntervalRequest, LinearRequest, MatrixRequest,
    NumberRequest, OptimizeRequest, PolynomialRequest, SimplifyRequest, SolveRequest, StatsRequest,
    SubstituteRequest, TraceRequest, UnitRequest, assumptions_schema_json, calculus_schema_json,
    complex_schema_json, finance_schema_json, inequality_schema_json, interval_schema_json,
    linear_schema_json, matrix_schema_json, number_schema_json, optimize_schema_json,
    polynomial_schema_json, schema_json, simplify_schema_json, solve_schema_json,
    stats_schema_json, substitute_schema_json, trace_schema_json, units_schema_json,
};

pub const ALLOWED_TOOL_COMMANDS: &[&str] = &[
    "eval",
    "simplify",
    "trace",
    "substitute",
    "assumptions",
    "solve",
    "calculus",
    "inequality",
    "polynomial",
    "interval",
    "finance",
    "units",
    "matrix",
    "stats",
    "optimize",
    "linear",
    "complex",
    "number",
];

const MAX_TOOL_INPUT_BYTES: usize = 128 * 1024;

fn parse_and_run<T, R>(input: &str, run: impl FnOnce(T) -> R) -> Result<Value, String>
where
    T: serde::de::DeserializeOwned,
    R: Serialize,
{
    let request = serde_json::from_str::<T>(input)
        .map_err(|error| format!("invalid request JSON: {error}"))?;
    serde_json::to_value(run(request))
        .map_err(|error| format!("cannot serialize calculator response: {error}"))
}

fn execute_value(command: &str, input: &str) -> Result<Value, String> {
    match command {
        "eval" => parse_and_run::<EvalRequest, _>(input, |request| request.evaluate()),
        "simplify" => parse_and_run::<SimplifyRequest, _>(input, |request| request.simplify()),
        "trace" => parse_and_run::<TraceRequest, _>(input, |request| request.evaluate()),
        "substitute" => {
            parse_and_run::<SubstituteRequest, _>(input, |request| request.substitute())
        }
        "assumptions" => {
            parse_and_run::<AssumptionsRequest, _>(input, |request| request.evaluate())
        }
        "solve" => parse_and_run::<SolveRequest, _>(input, |request| request.evaluate()),
        "calculus" => parse_and_run::<CalculusRequest, _>(input, |request| request.evaluate()),
        "inequality" => parse_and_run::<InequalityRequest, _>(input, |request| request.evaluate()),
        "polynomial" => parse_and_run::<PolynomialRequest, _>(input, |request| request.evaluate()),
        "interval" => parse_and_run::<IntervalRequest, _>(input, |request| request.evaluate()),
        "finance" => parse_and_run::<FinanceRequest, _>(input, |request| request.evaluate()),
        "units" => parse_and_run::<UnitRequest, _>(input, |request| request.evaluate()),
        "matrix" => parse_and_run::<MatrixRequest, _>(input, |request| request.evaluate()),
        "stats" => parse_and_run::<StatsRequest, _>(input, |request| request.evaluate()),
        "optimize" => parse_and_run::<OptimizeRequest, _>(input, |request| request.evaluate()),
        "linear" => parse_and_run::<LinearRequest, _>(input, |request| request.evaluate()),
        "complex" => parse_and_run::<ComplexRequest, _>(input, |request| request.evaluate()),
        "number" => parse_and_run::<NumberRequest, _>(input, |request| request.evaluate()),
        _ => Err(format!("unsupported calculator command `{command}`")),
    }
}

pub fn execute_tool_json(command: &str, input: &str) -> String {
    let response = if input.len() > MAX_TOOL_INPUT_BYTES {
        json!({
            "status": "rejected",
            "contract_version": CONTRACT_VERSION,
            "command": command,
            "error": {
                "code": "input_too_large",
                "reason": format!("tool request must be <= {MAX_TOOL_INPUT_BYTES} bytes")
            }
        })
    } else if !ALLOWED_TOOL_COMMANDS.contains(&command) {
        json!({
            "status": "rejected",
            "contract_version": CONTRACT_VERSION,
            "command": command,
            "error": {
                "code": "unsupported_command",
                "reason": format!("unsupported calculator command `{command}`")
            }
        })
    } else {
        match execute_value(command, input) {
            Ok(output) => json!({
                "status": "executed",
                "contract_version": CONTRACT_VERSION,
                "command": command,
                "output": output
            }),
            Err(reason) => json!({
                "status": "rejected",
                "contract_version": CONTRACT_VERSION,
                "command": command,
                "error": {
                    "code": "invalid_request",
                    "reason": reason
                }
            }),
        }
    };

    serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"status":"failed","error":{"code":"serialization_failure","reason":"calculator bridge could not serialize its response"}}"#.to_string()
    })
}

pub fn tool_schema_json(command: &str) -> String {
    let schema = match command {
        "eval" => Some(schema_json()),
        "simplify" => Some(simplify_schema_json()),
        "trace" => Some(trace_schema_json()),
        "substitute" => Some(substitute_schema_json()),
        "assumptions" => Some(assumptions_schema_json()),
        "solve" => Some(solve_schema_json()),
        "calculus" => Some(calculus_schema_json()),
        "inequality" => Some(inequality_schema_json()),
        "polynomial" => Some(polynomial_schema_json()),
        "interval" => Some(interval_schema_json()),
        "finance" => Some(finance_schema_json()),
        "units" => Some(units_schema_json()),
        "matrix" => Some(matrix_schema_json()),
        "stats" => Some(stats_schema_json()),
        "optimize" => Some(optimize_schema_json()),
        "linear" => Some(linear_schema_json()),
        "complex" => Some(complex_schema_json()),
        "number" => Some(number_schema_json()),
        _ => None,
    };

    match schema {
        Some(schema) => serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string()),
        None => serde_json::to_string(&json!({
            "error": {
                "code": "unsupported_command",
                "reason": format!("unsupported calculator command `{command}`")
            }
        }))
        .unwrap_or_else(|_| "{}".to_string()),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn execute_oddly_exact(command: &str, input: &str) -> String {
    execute_tool_json(command, input)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn schema_oddly_exact(command: &str) -> String {
    tool_schema_json(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_exact_eval_through_the_bridge() {
        let response = execute_tool_json(
            "eval",
            r#"{"expr":{"kind":"add","left":{"kind":"integer","value":"2"},"right":{"kind":"integer","value":"3"}}}"#,
        );
        let response: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(response["status"], "executed");
        assert_eq!(response["output"]["status"], "solved");
        assert_eq!(response["output"]["exact"]["numerator"], "5");
        assert_eq!(response["output"]["exact"]["denominator"], "1");
    }

    #[test]
    fn rejects_commands_outside_the_allowlist() {
        let response = execute_tool_json("shell", r#"{"command":"whoami"}"#);
        let response: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(response["status"], "rejected");
        assert_eq!(response["error"]["code"], "unsupported_command");
    }

    #[test]
    fn preserves_strict_request_deserialization() {
        let response = execute_tool_json(
            "eval",
            r#"{"expr":{"kind":"integer","value":"7"},"unexpected":true}"#,
        );
        let response: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(response["status"], "rejected");
        assert_eq!(response["error"]["code"], "invalid_request");
        assert!(
            response["error"]["reason"]
                .as_str()
                .unwrap()
                .contains("unknown field `unexpected`")
        );
    }

    #[test]
    fn exposes_the_runtime_schema_for_an_allowlisted_command() {
        let schema: Value = serde_json::from_str(&tool_schema_json("solve")).unwrap();

        assert_eq!(schema["additionalProperties"], false);
        assert!(schema["$defs"]["Expr"].is_object());
    }
}
