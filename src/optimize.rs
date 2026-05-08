use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::brent::BrentOpt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum OptimizeRequest {
    #[serde(rename = "minimize_1d")]
    Minimize1d {
        objective: Objective,
        lower: f64,
        upper: f64,
        #[serde(default = "default_max_iters")]
        max_iters: u64,
        #[serde(default = "default_abs_tolerance")]
        abs_tolerance: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Objective {
    Quadratic { a: f64, b: f64, c: f64 },
    Polynomial { coefficients: Vec<f64> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OptimizeResponse {
    Optimum {
        contract_version: String,
        minimizer: f64,
        minimum: f64,
        iterations: u64,
        exactness: OptimizeExactness,
        checks: Vec<OptimizeCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizeExactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OptimizeCheck {
    pub name: String,
    pub passed: bool,
}

impl OptimizeRequest {
    pub fn evaluate(&self) -> OptimizeResponse {
        match self.evaluate_inner() {
            Ok((minimizer, minimum, iterations)) => OptimizeResponse::Optimum {
                contract_version: CONTRACT_VERSION.to_owned(),
                minimizer,
                minimum,
                iterations,
                exactness: OptimizeExactness::ApproximateF64,
                checks: vec![OptimizeCheck {
                    name: "bounded_minimum_computed_by_argmin_brent".to_owned(),
                    passed: true,
                }],
            },
            Err(reason) => OptimizeResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<(f64, f64, u64), String> {
        match self {
            OptimizeRequest::Minimize1d {
                objective,
                lower,
                upper,
                max_iters,
                abs_tolerance,
            } => {
                objective.validate()?;
                ensure_finite(*lower, "lower")?;
                ensure_finite(*upper, "upper")?;
                ensure_positive_finite(*abs_tolerance, "abs_tolerance")?;
                if lower >= upper {
                    return Err("lower must be less than upper".to_owned());
                }
                if *max_iters == 0 {
                    return Err("max_iters must be greater than zero".to_owned());
                }

                let problem = ObjectiveProblem {
                    objective: objective.clone(),
                };
                let solver = BrentOpt::new(*lower, *upper)
                    .set_tolerance(f64::EPSILON.sqrt(), *abs_tolerance);
                let result = Executor::new(problem, solver)
                    .configure(|state| state.counting(true).max_iters(*max_iters))
                    .run()
                    .map_err(|e| e.to_string())?;
                let state = result.state();
                let minimizer = state
                    .best_param
                    .ok_or_else(|| "optimizer returned no minimizer".to_owned())?;
                let minimum = state.best_cost;
                if !minimizer.is_finite() || !minimum.is_finite() {
                    return Err("optimizer returned a non-finite result".to_owned());
                }
                Ok((minimizer, minimum, state.iter))
            }
        }
    }
}

impl Objective {
    pub fn evaluate_at(&self, x: f64) -> f64 {
        match self {
            Objective::Quadratic { a, b, c } => a * x * x + b * x + c,
            Objective::Polynomial { coefficients } => coefficients
                .iter()
                .rev()
                .fold(0.0, |acc, coeff| acc * x + coeff),
        }
    }

    fn validate(&self) -> Result<(), String> {
        match self {
            Objective::Quadratic { a, b, c } => {
                ensure_finite(*a, "a")?;
                ensure_finite(*b, "b")?;
                ensure_finite(*c, "c")?;
                if *a <= 0.0 {
                    return Err("quadratic coefficient a must be greater than zero".to_owned());
                }
                Ok(())
            }
            Objective::Polynomial { coefficients } => {
                if coefficients.is_empty() {
                    return Err("polynomial must contain at least one coefficient".to_owned());
                }
                if !coefficients.iter().all(|value| value.is_finite()) {
                    return Err("polynomial coefficients must be finite".to_owned());
                }
                Ok(())
            }
        }
    }
}

pub fn optimize_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/optimize.json"),
        "title": "agent-calc calc1 optimize request",
        "description": "Typed one-dimensional bounded optimization request backed by argmin.",
        "type": "object",
        "required": ["intent", "objective", "lower", "upper"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "minimize_1d"},
            "objective": {"$ref": "#/$defs/Objective"},
            "lower": {"type": "number"},
            "upper": {"type": "number"},
            "max_iters": {"type": "integer", "minimum": 1, "default": 100},
            "abs_tolerance": {"type": "number", "exclusiveMinimum": 0, "default": 1e-8}
        },
        "$defs": {
            "Objective": {
                "oneOf": [
                    {"$ref": "#/$defs/Quadratic"},
                    {"$ref": "#/$defs/Polynomial"}
                ]
            },
            "Quadratic": {
                "type": "object",
                "required": ["kind", "a", "b", "c"],
                "additionalProperties": false,
                "properties": {
                    "kind": {"const": "quadratic"},
                    "a": {"type": "number", "exclusiveMinimum": 0},
                    "b": {"type": "number"},
                    "c": {"type": "number"}
                }
            },
            "Polynomial": {
                "type": "object",
                "required": ["kind", "coefficients"],
                "additionalProperties": false,
                "properties": {
                    "kind": {"const": "polynomial"},
                    "coefficients": {
                        "type": "array",
                        "items": {"type": "number"},
                        "minItems": 1
                    }
                }
            }
        }
    })
}

#[derive(Clone, Debug)]
struct ObjectiveProblem {
    objective: Objective,
}

impl CostFunction for ObjectiveProblem {
    type Param = f64;
    type Output = f64;

    fn cost(&self, x: &Self::Param) -> Result<Self::Output, Error> {
        Ok(self.objective.evaluate_at(*x))
    }
}

fn ensure_finite(value: f64, name: &str) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(format!("{name} must be finite"))
    }
}

fn ensure_positive_finite(value: f64, name: &str) -> Result<(), String> {
    ensure_finite(value, name)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(format!("{name} must be greater than zero"))
    }
}

fn default_max_iters() -> u64 {
    100
}

fn default_abs_tolerance() -> f64 {
    1e-8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimizes_quadratic() {
        match (OptimizeRequest::Minimize1d {
            objective: Objective::Quadratic {
                a: 2.0,
                b: -8.0,
                c: 3.0,
            },
            lower: -10.0,
            upper: 10.0,
            max_iters: 100,
            abs_tolerance: 1e-10,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                minimum,
                checks,
                ..
            } => {
                assert!((minimizer - 2.0).abs() < 1e-6);
                assert!((minimum + 5.0).abs() < 1e-6);
                assert_eq!(checks[0].name, "bounded_minimum_computed_by_argmin_brent");
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn minimizes_polynomial() {
        match (OptimizeRequest::Minimize1d {
            objective: Objective::Polynomial {
                coefficients: vec![9.0, -6.0, 1.0],
            },
            lower: -10.0,
            upper: 10.0,
            max_iters: 100,
            abs_tolerance: 1e-10,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!((minimizer - 3.0).abs() < 1e-6);
                assert!(minimum.abs() < 1e-6);
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Quadratic { a: 0.0, b: 1.0, c: 1.0 },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "quadratic coefficient a must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "polynomial must contain at least one coefficient"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: 1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "lower must be less than upper"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 0,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "max_iters must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Quadratic { a: f64::NAN, b: 1.0, c: 1.0 },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "a must be finite"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: f64::NAN,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "lower must be finite"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 0.0,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "abs_tolerance must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![f64::INFINITY] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "polynomial coefficients must be finite"
        ));
    }

    #[test]
    fn default_tolerance_remains_precise_enough() {
        let request_json = r#"{
            "intent": "minimize_1d",
            "objective": { "kind": "quadratic", "a": 1.0, "b": -6.0, "c": 9.0 },
            "lower": -10.0,
            "upper": 10.0
        }"#;
        let request: OptimizeRequest = serde_json::from_str(request_json).unwrap();
        match request.evaluate() {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!((minimizer - 3.0).abs() < 1e-4);
                assert!(minimum.abs() < 1e-6);
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_non_finite_optimizer_results() {
        match (OptimizeRequest::Minimize1d {
            objective: Objective::Polynomial {
                coefficients: vec![0.0, 0.0, f64::MAX],
            },
            lower: -10.0,
            upper: 10.0,
            max_iters: 100,
            abs_tolerance: 1e-8,
        })
        .evaluate()
        {
            OptimizeResponse::Error { reason, .. } => {
                assert_eq!(reason, "optimizer returned a non-finite result");
            }
            other => panic!("expected error response, got {other:?}"),
        }
    }
}
