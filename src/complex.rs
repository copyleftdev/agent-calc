use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum ComplexRequest {
    Add {
        left: ComplexInput,
        right: ComplexInput,
    },
    Sub {
        left: ComplexInput,
        right: ComplexInput,
    },
    Mul {
        left: ComplexInput,
        right: ComplexInput,
    },
    Div {
        left: ComplexInput,
        right: ComplexInput,
    },
    Conjugate {
        value: ComplexInput,
    },
    Abs {
        value: ComplexInput,
    },
    Arg {
        value: ComplexInput,
    },
    FromPolar {
        radius: f64,
        theta: f64,
    },
    ToPolar {
        value: ComplexInput,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComplexInput {
    pub re: f64,
    pub im: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ComplexResponse {
    Complex {
        contract_version: String,
        re: f64,
        im: f64,
        exactness: ComplexExactness,
        checks: Vec<ComplexCheck>,
    },
    Scalar {
        contract_version: String,
        value: f64,
        exactness: ComplexExactness,
        checks: Vec<ComplexCheck>,
    },
    Polar {
        contract_version: String,
        radius: f64,
        theta: f64,
        exactness: ComplexExactness,
        checks: Vec<ComplexCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexExactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComplexCheck {
    pub name: String,
    pub passed: bool,
}

impl ComplexRequest {
    pub fn evaluate(&self) -> ComplexResponse {
        match self.evaluate_inner() {
            Ok(ComplexOutput::Complex(value)) => ComplexResponse::Complex {
                contract_version: CONTRACT_VERSION.to_owned(),
                re: value.re,
                im: value.im,
                exactness: ComplexExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(ComplexOutput::Scalar(value)) => ComplexResponse::Scalar {
                contract_version: CONTRACT_VERSION.to_owned(),
                value,
                exactness: ComplexExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(ComplexOutput::Polar { radius, theta }) => ComplexResponse::Polar {
                contract_version: CONTRACT_VERSION.to_owned(),
                radius,
                theta,
                exactness: ComplexExactness::ApproximateF64,
                checks: default_checks(),
            },
            Err(reason) => ComplexResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<ComplexOutput, String> {
        match self {
            ComplexRequest::Add { left, right } => Ok(ComplexOutput::Complex(
                parse_complex(left)? + parse_complex(right)?,
            )),
            ComplexRequest::Sub { left, right } => Ok(ComplexOutput::Complex(
                parse_complex(left)? - parse_complex(right)?,
            )),
            ComplexRequest::Mul { left, right } => Ok(ComplexOutput::Complex(
                parse_complex(left)? * parse_complex(right)?,
            )),
            ComplexRequest::Div { left, right } => {
                let right = parse_complex(right)?;
                if right.norm_sqr() == 0.0 {
                    return Err("complex division by zero".to_owned());
                }
                Ok(ComplexOutput::Complex(parse_complex(left)? / right))
            }
            ComplexRequest::Conjugate { value } => {
                Ok(ComplexOutput::Complex(parse_complex(value)?.conj()))
            }
            ComplexRequest::Abs { value } => {
                Ok(ComplexOutput::Scalar(parse_complex(value)?.norm()))
            }
            ComplexRequest::Arg { value } => Ok(ComplexOutput::Scalar(parse_complex(value)?.arg())),
            ComplexRequest::FromPolar { radius, theta } => {
                ensure_finite(*radius, "radius")?;
                ensure_finite(*theta, "theta")?;
                if *radius < 0.0 {
                    return Err("radius must be non-negative".to_owned());
                }
                Ok(ComplexOutput::Complex(Complex64::from_polar(
                    *radius, *theta,
                )))
            }
            ComplexRequest::ToPolar { value } => {
                let value = parse_complex(value)?;
                Ok(ComplexOutput::Polar {
                    radius: value.norm(),
                    theta: value.arg(),
                })
            }
        }
    }
}

pub fn complex_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/complex.json"),
        "title": "agent-calc calc1 complex request",
        "description": "Typed complex-number request backed by num-complex.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/BinaryComplex"},
            {"$ref": "#/$defs/UnaryComplex"},
            {"$ref": "#/$defs/FromPolar"}
        ],
        "$defs": {
            "Complex": {
                "type": "object",
                "required": ["re", "im"],
                "additionalProperties": false,
                "properties": {
                    "re": {"type": "number"},
                    "im": {"type": "number"}
                }
            },
            "BinaryComplex": {
                "type": "object",
                "required": ["intent", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["add", "sub", "mul", "div"]},
                    "left": {"$ref": "#/$defs/Complex"},
                    "right": {"$ref": "#/$defs/Complex"}
                }
            },
            "UnaryComplex": {
                "type": "object",
                "required": ["intent", "value"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["conjugate", "abs", "arg", "to_polar"]},
                    "value": {"$ref": "#/$defs/Complex"}
                }
            },
            "FromPolar": {
                "type": "object",
                "required": ["intent", "radius", "theta"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "from_polar"},
                    "radius": {"type": "number", "minimum": 0},
                    "theta": {"type": "number"}
                }
            }
        }
    })
}

enum ComplexOutput {
    Complex(Complex64),
    Scalar(f64),
    Polar { radius: f64, theta: f64 },
}

fn parse_complex(input: &ComplexInput) -> Result<Complex64, String> {
    ensure_finite(input.re, "re")?;
    ensure_finite(input.im, "im")?;
    Ok(Complex64::new(input.re, input.im))
}

fn ensure_finite(value: f64, name: &str) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(format!("{name} must be finite"))
    }
}

fn default_checks() -> Vec<ComplexCheck> {
    vec![ComplexCheck {
        name: "computed_by_num_complex".to_owned(),
        passed: true,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(re: f64, im: f64) -> ComplexInput {
        ComplexInput { re, im }
    }

    fn complex(response: ComplexResponse) -> (f64, f64) {
        match response {
            ComplexResponse::Complex { re, im, checks, .. } => {
                assert_computed_by_num_complex(checks);
                (re, im)
            }
            other => panic!("expected complex response, got {other:?}"),
        }
    }

    fn scalar(response: ComplexResponse) -> f64 {
        match response {
            ComplexResponse::Scalar { value, checks, .. } => {
                assert_computed_by_num_complex(checks);
                value
            }
            other => panic!("expected scalar response, got {other:?}"),
        }
    }

    fn assert_computed_by_num_complex(checks: Vec<ComplexCheck>) {
        assert_eq!(
            checks,
            vec![ComplexCheck {
                name: "computed_by_num_complex".to_owned(),
                passed: true,
            }]
        );
    }

    #[test]
    fn performs_complex_arithmetic() {
        assert_eq!(
            complex(
                ComplexRequest::Add {
                    left: z(1.0, 2.0),
                    right: z(3.0, -4.0),
                }
                .evaluate()
            ),
            (4.0, -2.0)
        );
        assert_eq!(
            complex(
                ComplexRequest::Sub {
                    left: z(1.0, 2.0),
                    right: z(3.0, -4.0),
                }
                .evaluate()
            ),
            (-2.0, 6.0)
        );
        assert_eq!(
            complex(
                ComplexRequest::Mul {
                    left: z(1.0, 2.0),
                    right: z(3.0, 4.0),
                }
                .evaluate()
            ),
            (-5.0, 10.0)
        );
        let (re, im) = complex(
            ComplexRequest::Div {
                left: z(1.0, 2.0),
                right: z(3.0, 4.0),
            }
            .evaluate(),
        );
        assert!((re - 0.44).abs() < 1e-12);
        assert!((im - 0.08).abs() < 1e-12);
    }

    #[test]
    fn computes_conjugate_abs_arg_and_polar() {
        assert_eq!(
            complex(ComplexRequest::Conjugate { value: z(3.0, 4.0) }.evaluate()),
            (3.0, -4.0)
        );
        assert_eq!(
            scalar(ComplexRequest::Abs { value: z(3.0, 4.0) }.evaluate()),
            5.0
        );
        assert!(
            (scalar(ComplexRequest::Arg { value: z(0.0, 1.0) }.evaluate())
                - std::f64::consts::FRAC_PI_2)
                .abs()
                < 1e-12
        );

        let (re, im) = complex(
            ComplexRequest::FromPolar {
                radius: 2.0,
                theta: std::f64::consts::FRAC_PI_2,
            }
            .evaluate(),
        );
        assert!(re.abs() < 1e-12);
        assert!((im - 2.0).abs() < 1e-12);

        assert_eq!(
            complex(
                ComplexRequest::FromPolar {
                    radius: 0.0,
                    theta: 7.0,
                }
                .evaluate()
            ),
            (0.0, 0.0)
        );

        match (ComplexRequest::ToPolar { value: z(0.0, 2.0) }).evaluate() {
            ComplexResponse::Polar {
                radius,
                theta,
                checks,
                ..
            } => {
                assert_computed_by_num_complex(checks);
                assert_eq!(radius, 2.0);
                assert!((theta - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
            }
            other => panic!("expected polar response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(matches!(
            (ComplexRequest::Div { left: z(1.0, 0.0), right: z(0.0, 0.0) }).evaluate(),
            ComplexResponse::Error { reason, .. } if reason == "complex division by zero"
        ));
        assert!(matches!(
            (ComplexRequest::FromPolar { radius: -1.0, theta: 0.0 }).evaluate(),
            ComplexResponse::Error { reason, .. } if reason == "radius must be non-negative"
        ));
        assert!(matches!(
            (ComplexRequest::Abs { value: z(f64::NAN, 0.0) }).evaluate(),
            ComplexResponse::Error { reason, .. } if reason == "re must be finite"
        ));
    }
}
