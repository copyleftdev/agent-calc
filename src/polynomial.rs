use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, ExactRational, classify_error},
};
use num_bigint::BigInt;
use num_traits::Signed;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum PolynomialRequest {
    Normalize {
        polynomial: PolynomialInput,
    },
    Add {
        left: PolynomialInput,
        right: PolynomialInput,
    },
    Mul {
        left: PolynomialInput,
        right: PolynomialInput,
    },
    Evaluate {
        polynomial: PolynomialInput,
        at: Expr,
    },
    Derivative {
        polynomial: PolynomialInput,
    },
    Solve {
        polynomial: PolynomialInput,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolynomialInput {
    pub variable: String,
    pub coefficients: Vec<Expr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PolynomialResponse {
    Polynomial {
        contract_version: String,
        variable: String,
        coefficients: Vec<ExactRational>,
        checks: Vec<PolynomialCheck>,
    },
    Value {
        contract_version: String,
        exact: ExactRational,
        checks: Vec<PolynomialCheck>,
    },
    Roots {
        contract_version: String,
        variable: String,
        roots: Vec<ExactRational>,
        checks: Vec<PolynomialCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolynomialCheck {
    pub name: String,
    pub passed: bool,
}

impl PolynomialRequest {
    pub fn evaluate(&self) -> PolynomialResponse {
        match self.evaluate_inner() {
            Ok(PolynomialOutput::Polynomial {
                variable,
                coefficients,
            }) => PolynomialResponse::Polynomial {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable,
                coefficients: coefficients.iter().map(exact_rational).collect(),
                checks: default_checks(),
            },
            Ok(PolynomialOutput::Value(value)) => PolynomialResponse::Value {
                contract_version: CONTRACT_VERSION.to_owned(),
                exact: exact_rational(&value),
                checks: default_checks(),
            },
            Ok(PolynomialOutput::Roots { variable, roots }) => PolynomialResponse::Roots {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable,
                roots: roots.iter().map(exact_rational).collect(),
                checks: solve_checks(),
            },
            Err(reason) => PolynomialResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<PolynomialOutput, String> {
        match self {
            PolynomialRequest::Normalize { polynomial } => {
                let polynomial = Polynomial::parse(polynomial)?;
                Ok(PolynomialOutput::Polynomial {
                    variable: polynomial.variable,
                    coefficients: polynomial.coefficients,
                })
            }
            PolynomialRequest::Add { left, right } => {
                let left = Polynomial::parse(left)?;
                let right = Polynomial::parse(right)?;
                ensure_same_variable(&left, &right)?;
                Ok(PolynomialOutput::Polynomial {
                    variable: left.variable.clone(),
                    coefficients: add_coefficients(&left.coefficients, &right.coefficients)?,
                })
            }
            PolynomialRequest::Mul { left, right } => {
                let left = Polynomial::parse(left)?;
                let right = Polynomial::parse(right)?;
                ensure_same_variable(&left, &right)?;
                Ok(PolynomialOutput::Polynomial {
                    variable: left.variable.clone(),
                    coefficients: mul_coefficients(&left.coefficients, &right.coefficients)?,
                })
            }
            PolynomialRequest::Evaluate { polynomial, at } => {
                let polynomial = Polynomial::parse(polynomial)?;
                let at = at.evaluate()?;
                Ok(PolynomialOutput::Value(evaluate_coefficients(
                    &polynomial.coefficients,
                    &at,
                )?))
            }
            PolynomialRequest::Derivative { polynomial } => {
                let polynomial = Polynomial::parse(polynomial)?;
                Ok(PolynomialOutput::Polynomial {
                    variable: polynomial.variable,
                    coefficients: derivative_coefficients(&polynomial.coefficients)?,
                })
            }
            PolynomialRequest::Solve { polynomial } => {
                let polynomial = Polynomial::parse(polynomial)?;
                Ok(PolynomialOutput::Roots {
                    variable: polynomial.variable,
                    roots: solve_coefficients(&polynomial.coefficients)?,
                })
            }
        }
    }
}

pub fn polynomial_schema_json() -> Value {
    let expr_defs = crate::schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/polynomial.json"),
        "title": "agent-calc calc1 polynomial request",
        "description": "Typed exact polynomial request over rational coefficients in ascending power order.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/UnaryPolynomial"},
            {"$ref": "#/$defs/BinaryPolynomial"},
            {"$ref": "#/$defs/Evaluate"}
        ],
        "$defs": {
            "Expr": expr_defs["Expr"].clone(),
            "Integer": expr_defs["Integer"].clone(),
            "Rational": expr_defs["Rational"].clone(),
            "Symbol": expr_defs["Symbol"].clone(),
            "Add": expr_defs["Add"].clone(),
            "Sub": expr_defs["Sub"].clone(),
            "Mul": expr_defs["Mul"].clone(),
            "Div": expr_defs["Div"].clone(),
            "Pow": expr_defs["Pow"].clone(),
            "Neg": expr_defs["Neg"].clone(),
            "Polynomial": {
                "type": "object",
                "required": ["variable", "coefficients"],
                "additionalProperties": false,
                "properties": {
                    "variable": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"},
                    "coefficients": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Expr"}
                    }
                }
            },
            "UnaryPolynomial": {
                "type": "object",
                "required": ["intent", "polynomial"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["normalize", "derivative", "solve"]},
                    "polynomial": {"$ref": "#/$defs/Polynomial"}
                }
            },
            "BinaryPolynomial": {
                "type": "object",
                "required": ["intent", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["add", "mul"]},
                    "left": {"$ref": "#/$defs/Polynomial"},
                    "right": {"$ref": "#/$defs/Polynomial"}
                }
            },
            "Evaluate": {
                "type": "object",
                "required": ["intent", "polynomial", "at"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "evaluate"},
                    "polynomial": {"$ref": "#/$defs/Polynomial"},
                    "at": {"$ref": "#/$defs/Expr"}
                }
            }
        }
    })
}

struct Polynomial {
    variable: String,
    coefficients: Vec<Rational>,
}

enum PolynomialOutput {
    Polynomial {
        variable: String,
        coefficients: Vec<Rational>,
    },
    Value(Rational),
    Roots {
        variable: String,
        roots: Vec<Rational>,
    },
}

impl Polynomial {
    fn parse(input: &PolynomialInput) -> Result<Self, String> {
        if !is_valid_symbol_name(&input.variable) {
            return Err(format!("invalid polynomial variable `{}`", input.variable));
        }

        let mut coefficients = Vec::with_capacity(input.coefficients.len());
        for coefficient in &input.coefficients {
            coefficients.push(coefficient.evaluate()?);
        }
        normalize_coefficients(&mut coefficients);

        Ok(Self {
            variable: input.variable.clone(),
            coefficients,
        })
    }
}

fn ensure_same_variable(left: &Polynomial, right: &Polynomial) -> Result<(), String> {
    if left.variable == right.variable {
        Ok(())
    } else {
        Err(format!(
            "variable mismatch `{}` != `{}`",
            left.variable, right.variable
        ))
    }
}

fn add_coefficients(left: &[Rational], right: &[Rational]) -> Result<Vec<Rational>, String> {
    let len = left.len().max(right.len());
    let mut result = Vec::with_capacity(len);
    for index in 0..len {
        let left = left.get(index).cloned().unwrap_or_else(Rational::zero);
        let right = right.get(index).cloned().unwrap_or_else(Rational::zero);
        result.push(left.checked_add(&right).map_err(|e| e.to_string())?);
    }
    normalize_coefficients(&mut result);
    Ok(result)
}

fn mul_coefficients(left: &[Rational], right: &[Rational]) -> Result<Vec<Rational>, String> {
    let mut result = vec![Rational::zero(); left.len() + right.len() - 1];
    for (left_index, left_coefficient) in left.iter().enumerate() {
        for (right_index, right_coefficient) in right.iter().enumerate() {
            let product = left_coefficient
                .checked_mul(right_coefficient)
                .map_err(|e| e.to_string())?;
            let current = result[left_index + right_index].clone();
            result[left_index + right_index] =
                current.checked_add(&product).map_err(|e| e.to_string())?;
        }
    }
    normalize_coefficients(&mut result);
    Ok(result)
}

fn evaluate_coefficients(coefficients: &[Rational], at: &Rational) -> Result<Rational, String> {
    let mut acc = Rational::zero();
    for coefficient in coefficients.iter().rev() {
        acc = acc.checked_mul(at).map_err(|e| e.to_string())?;
        acc = acc.checked_add(coefficient).map_err(|e| e.to_string())?;
    }
    Ok(acc)
}

fn derivative_coefficients(coefficients: &[Rational]) -> Result<Vec<Rational>, String> {
    if coefficients.len() <= 1 {
        return Ok(vec![Rational::zero()]);
    }

    let mut result = Vec::with_capacity(coefficients.len() - 1);
    for (index, coefficient) in coefficients.iter().enumerate().skip(1) {
        let multiplier = Rational::integer(index);
        result.push(
            coefficient
                .checked_mul(&multiplier)
                .map_err(|e| e.to_string())?,
        );
    }
    normalize_coefficients(&mut result);
    Ok(result)
}

fn solve_coefficients(coefficients: &[Rational]) -> Result<Vec<Rational>, String> {
    match coefficients.len() - 1 {
        0 => solve_constant(coefficients),
        1 => solve_linear(coefficients),
        2 => solve_quadratic(coefficients),
        degree => Err(format!("polynomial degree {degree} is not supported")),
    }
}

fn solve_constant(coefficients: &[Rational]) -> Result<Vec<Rational>, String> {
    if coefficients[0].is_zero() {
        Err("zero polynomial has infinitely many roots".to_owned())
    } else {
        Ok(vec![])
    }
}

fn solve_linear(coefficients: &[Rational]) -> Result<Vec<Rational>, String> {
    let constant = &coefficients[0];
    let linear = &coefficients[1];
    let numerator = constant.checked_neg().map_err(|e| e.to_string())?;
    Ok(vec![
        numerator.checked_div(linear).map_err(|e| e.to_string())?,
    ])
}

fn solve_quadratic(coefficients: &[Rational]) -> Result<Vec<Rational>, String> {
    let c = &coefficients[0];
    let b = &coefficients[1];
    let a = &coefficients[2];

    let four_ac = a
        .checked_mul(c)
        .and_then(|value| value.checked_mul(&Rational::integer(4)))
        .map_err(|e| e.to_string())?;
    let discriminant = b
        .checked_mul(b)
        .and_then(|value| value.checked_sub(&four_ac))
        .map_err(|e| e.to_string())?;
    if discriminant < Rational::zero() {
        return Err("quadratic has no real rational roots".to_owned());
    }
    let sqrt_discriminant =
        rational_sqrt(&discriminant).ok_or_else(|| "quadratic roots are irrational".to_owned())?;
    let neg_b = b.checked_neg().map_err(|e| e.to_string())?;
    let denominator = a
        .checked_mul(&Rational::integer(2))
        .map_err(|e| e.to_string())?;

    let first = neg_b
        .checked_sub(&sqrt_discriminant)
        .and_then(|value| value.checked_div(&denominator))
        .map_err(|e| e.to_string())?;
    let second = neg_b
        .checked_add(&sqrt_discriminant)
        .and_then(|value| value.checked_div(&denominator))
        .map_err(|e| e.to_string())?;

    let mut roots = vec![first, second];
    roots.sort();
    roots.dedup();
    Ok(roots)
}

fn rational_sqrt(value: &Rational) -> Option<Rational> {
    if value < &Rational::zero() {
        return None;
    }

    let numerator = value.numerator();
    let denominator = value.denominator();
    if !is_perfect_square(numerator) || !is_perfect_square(denominator) {
        return None;
    }

    Rational::new(numerator.sqrt(), denominator.sqrt()).ok()
}

fn is_perfect_square(value: &BigInt) -> bool {
    if value.is_negative() {
        return false;
    }
    let root = value.sqrt();
    &root * &root == *value
}

fn normalize_coefficients(coefficients: &mut Vec<Rational>) {
    if coefficients.is_empty() {
        coefficients.push(Rational::zero());
        return;
    }

    loop {
        if coefficients.len() == 1 {
            break;
        }
        match coefficients.last() {
            Some(value) if value.is_zero() => {
                coefficients.pop();
            }
            _ => break,
        }
    }
}

fn exact_rational(value: &Rational) -> ExactRational {
    ExactRational {
        numerator: value.numerator().to_string(),
        denominator: value.denominator().to_string(),
        display: value.to_string(),
    }
}

fn default_checks() -> Vec<PolynomialCheck> {
    vec![
        PolynomialCheck {
            name: "coefficients_are_exact_rationals".to_owned(),
            passed: true,
        },
        PolynomialCheck {
            name: "trailing_zero_coefficients_removed".to_owned(),
            passed: true,
        },
    ]
}

fn solve_checks() -> Vec<PolynomialCheck> {
    vec![
        PolynomialCheck {
            name: "coefficients_are_exact_rationals".to_owned(),
            passed: true,
        },
        PolynomialCheck {
            name: "degree_at_most_two".to_owned(),
            passed: true,
        },
        PolynomialCheck {
            name: "roots_are_exact_rationals".to_owned(),
            passed: true,
        },
    ]
}

fn is_valid_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i32) -> Expr {
        Expr::Integer {
            value: value.to_string(),
        }
    }

    fn poly(coefficients: Vec<Expr>) -> PolynomialInput {
        PolynomialInput {
            variable: "x".to_owned(),
            coefficients,
        }
    }

    fn displays(response: PolynomialResponse) -> Vec<String> {
        match response {
            PolynomialResponse::Polynomial {
                coefficients,
                checks,
                ..
            } => {
                assert_default_checks(checks);
                coefficients
                    .into_iter()
                    .map(|value| value.display)
                    .collect()
            }
            other => panic!("expected polynomial response, got {other:?}"),
        }
    }

    fn value(response: PolynomialResponse) -> String {
        match response {
            PolynomialResponse::Value { exact, checks, .. } => {
                assert_default_checks(checks);
                exact.display
            }
            other => panic!("expected value response, got {other:?}"),
        }
    }

    fn roots(response: PolynomialResponse) -> Vec<String> {
        match response {
            PolynomialResponse::Roots { roots, checks, .. } => {
                assert_eq!(
                    checks,
                    vec![
                        PolynomialCheck {
                            name: "coefficients_are_exact_rationals".to_owned(),
                            passed: true,
                        },
                        PolynomialCheck {
                            name: "degree_at_most_two".to_owned(),
                            passed: true,
                        },
                        PolynomialCheck {
                            name: "roots_are_exact_rationals".to_owned(),
                            passed: true,
                        },
                    ]
                );
                roots.into_iter().map(|root| root.display).collect()
            }
            other => panic!("expected roots response, got {other:?}"),
        }
    }

    fn assert_default_checks(checks: Vec<PolynomialCheck>) {
        assert_eq!(
            checks,
            vec![
                PolynomialCheck {
                    name: "coefficients_are_exact_rationals".to_owned(),
                    passed: true,
                },
                PolynomialCheck {
                    name: "trailing_zero_coefficients_removed".to_owned(),
                    passed: true,
                },
            ]
        );
    }

    #[test]
    fn normalizes_and_differentiates_polynomials() {
        assert_eq!(
            displays(
                PolynomialRequest::Normalize {
                    polynomial: poly(vec![int(1), int(0), int(0)]),
                }
                .evaluate()
            ),
            vec!["1"]
        );
        assert_eq!(
            displays(
                PolynomialRequest::Normalize {
                    polynomial: poly(vec![]),
                }
                .evaluate()
            ),
            vec!["0"]
        );

        assert_eq!(
            displays(
                PolynomialRequest::Derivative {
                    polynomial: poly(vec![int(3), int(2), int(5)]),
                }
                .evaluate()
            ),
            vec!["2", "10"]
        );
    }

    #[test]
    fn adds_multiplies_and_evaluates_polynomials() {
        assert_eq!(
            displays(
                PolynomialRequest::Add {
                    left: poly(vec![int(1), int(2)]),
                    right: poly(vec![int(3), int(-2), int(4)]),
                }
                .evaluate()
            ),
            vec!["4", "0", "4"]
        );

        assert_eq!(
            displays(
                PolynomialRequest::Mul {
                    left: poly(vec![int(1), int(1)]),
                    right: poly(vec![int(1), int(-1)]),
                }
                .evaluate()
            ),
            vec!["1", "0", "-1"]
        );
        assert_eq!(
            displays(
                PolynomialRequest::Mul {
                    left: poly(vec![int(0)]),
                    right: poly(vec![int(2), int(3)]),
                }
                .evaluate()
            ),
            vec!["0"]
        );
        assert_eq!(
            displays(
                PolynomialRequest::Mul {
                    left: poly(vec![int(2), int(3)]),
                    right: poly(vec![int(0)]),
                }
                .evaluate()
            ),
            vec!["0"]
        );

        assert_eq!(
            value(
                PolynomialRequest::Evaluate {
                    polynomial: poly(vec![int(1), int(2), int(3)]),
                    at: int(2),
                }
                .evaluate()
            ),
            "17"
        );
    }

    #[test]
    fn rejects_invalid_polynomial_inputs() {
        assert!(matches!(
            (PolynomialRequest::Normalize {
                polynomial: PolynomialInput {
                    variable: "1x".to_owned(),
                    coefficients: vec![int(1)],
                },
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "invalid polynomial variable `1x`"
        ));
        assert!(matches!(
            (PolynomialRequest::Normalize {
                polynomial: PolynomialInput {
                    variable: "_x".to_owned(),
                    coefficients: vec![int(1)],
                },
            })
            .evaluate(),
            PolynomialResponse::Polynomial { variable, .. } if variable == "_x"
        ));
        assert!(matches!(
            (PolynomialRequest::Normalize {
                polynomial: PolynomialInput {
                    variable: "x_y".to_owned(),
                    coefficients: vec![int(1)],
                },
            })
            .evaluate(),
            PolynomialResponse::Polynomial { variable, .. } if variable == "x_y"
        ));

        assert!(matches!(
            (PolynomialRequest::Add {
                left: poly(vec![int(1)]),
                right: PolynomialInput {
                    variable: "y".to_owned(),
                    coefficients: vec![int(1)],
                },
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "variable mismatch `x` != `y`"
        ));
    }

    #[test]
    fn solves_linear_and_quadratic_equations_exactly() {
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-4), int(2)]),
                }
                .evaluate()
            ),
            vec!["2"]
        );
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(6), int(-5), int(1)]),
                }
                .evaluate()
            ),
            vec!["2", "3"]
        );
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(1), int(-2), int(1)]),
                }
                .evaluate()
            ),
            vec!["1"]
        );
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(1)]),
                }
                .evaluate()
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn solve_reports_unsupported_or_non_rational_cases() {
        assert!(matches!(
            (PolynomialRequest::Solve {
                polynomial: poly(vec![int(0)]),
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "zero polynomial has infinitely many roots"
        ));
        assert!(matches!(
            (PolynomialRequest::Solve {
                polynomial: poly(vec![int(-2), int(0), int(1)]),
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "quadratic roots are irrational"
        ));
        assert!(matches!(
            (PolynomialRequest::Solve {
                polynomial: poly(vec![int(1), int(0), int(1)]),
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "quadratic has no real rational roots"
        ));
        assert!(matches!(
            (PolynomialRequest::Solve {
                polynomial: poly(vec![int(1), int(0), int(0), int(1)]),
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "polynomial degree 3 is not supported"
        ));
    }
}
