use crate::{
    CONTRACT_VERSION, Expr, PolynomialInput, PolynomialRequest, PolynomialResponse, Rational,
    protocol::{ErrorCode, ExactRational, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum IntervalRequest {
    Add {
        left: IntervalInput,
        right: IntervalInput,
    },
    Sub {
        left: IntervalInput,
        right: IntervalInput,
    },
    Mul {
        left: IntervalInput,
        right: IntervalInput,
    },
    Div {
        left: IntervalInput,
        right: IntervalInput,
    },
    Pow {
        interval: IntervalInput,
        exponent: i32,
    },
    PolynomialRange {
        polynomial: PolynomialInput,
        domain: IntervalInput,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntervalInput {
    pub lower: Expr,
    pub upper: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum IntervalResponse {
    Interval {
        contract_version: String,
        lower: ExactRational,
        upper: ExactRational,
        checks: Vec<IntervalCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntervalCheck {
    pub name: String,
    pub passed: bool,
}

impl IntervalRequest {
    pub fn evaluate(&self) -> IntervalResponse {
        match self.evaluate_inner() {
            Ok(interval) => IntervalResponse::Interval {
                contract_version: CONTRACT_VERSION.to_owned(),
                lower: exact_rational(&interval.lower),
                upper: exact_rational(&interval.upper),
                checks: default_checks(),
            },
            Err(reason) => IntervalResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<Interval, String> {
        match self {
            IntervalRequest::Add { left, right } => {
                let left = Interval::parse(left)?;
                let right = Interval::parse(right)?;
                Interval::new(
                    left.lower
                        .checked_add(&right.lower)
                        .map_err(|e| e.to_string())?,
                    left.upper
                        .checked_add(&right.upper)
                        .map_err(|e| e.to_string())?,
                )
            }
            IntervalRequest::Sub { left, right } => {
                let left = Interval::parse(left)?;
                let right = Interval::parse(right)?;
                Interval::new(
                    left.lower
                        .checked_sub(&right.upper)
                        .map_err(|e| e.to_string())?,
                    left.upper
                        .checked_sub(&right.lower)
                        .map_err(|e| e.to_string())?,
                )
            }
            IntervalRequest::Mul { left, right } => {
                multiply_intervals(&Interval::parse(left)?, &Interval::parse(right)?)
            }
            IntervalRequest::Div { left, right } => {
                divide_intervals(&Interval::parse(left)?, &Interval::parse(right)?)
            }
            IntervalRequest::Pow { interval, exponent } => {
                pow_interval(&Interval::parse(interval)?, *exponent)
            }
            IntervalRequest::PolynomialRange { polynomial, domain } => {
                polynomial_range(polynomial, &Interval::parse(domain)?)
            }
        }
    }
}

pub fn interval_schema_json() -> Value {
    let polynomial_schema = crate::polynomial_schema_json();
    let defs = polynomial_schema["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/interval.json"),
        "title": "agent-calc calc1 interval request",
        "description": "Typed exact closed-interval arithmetic and polynomial range request.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/BinaryInterval"},
            {"$ref": "#/$defs/PowInterval"},
            {"$ref": "#/$defs/PolynomialRange"}
        ],
        "$defs": {
            "Expr": defs["Expr"].clone(),
            "Integer": defs["Integer"].clone(),
            "Rational": defs["Rational"].clone(),
            "Symbol": defs["Symbol"].clone(),
            "Add": defs["Add"].clone(),
            "Sub": defs["Sub"].clone(),
            "Mul": defs["Mul"].clone(),
            "Div": defs["Div"].clone(),
            "Pow": defs["Pow"].clone(),
            "Neg": defs["Neg"].clone(),
            "Polynomial": defs["Polynomial"].clone(),
            "Interval": {
                "type": "object",
                "required": ["lower", "upper"],
                "additionalProperties": false,
                "properties": {
                    "lower": {"$ref": "#/$defs/Expr"},
                    "upper": {"$ref": "#/$defs/Expr"}
                }
            },
            "BinaryInterval": {
                "type": "object",
                "required": ["intent", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["add", "sub", "mul", "div"]},
                    "left": {"$ref": "#/$defs/Interval"},
                    "right": {"$ref": "#/$defs/Interval"}
                }
            },
            "PowInterval": {
                "type": "object",
                "required": ["intent", "interval", "exponent"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "pow"},
                    "interval": {"$ref": "#/$defs/Interval"},
                    "exponent": {
                        "type": "integer",
                        "minimum": -2147483648,
                        "maximum": 2147483647
                    }
                }
            },
            "PolynomialRange": {
                "type": "object",
                "required": ["intent", "polynomial", "domain"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "polynomial_range"},
                    "polynomial": {"$ref": "#/$defs/Polynomial"},
                    "domain": {"$ref": "#/$defs/Interval"}
                }
            }
        }
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Interval {
    lower: Rational,
    upper: Rational,
}

impl Interval {
    fn parse(input: &IntervalInput) -> Result<Self, String> {
        Self::new(input.lower.evaluate()?, input.upper.evaluate()?)
    }

    fn new(lower: Rational, upper: Rational) -> Result<Self, String> {
        if lower <= upper {
            Ok(Self { lower, upper })
        } else {
            Err("interval lower bound must be <= upper bound".to_owned())
        }
    }

    fn contains_zero(&self) -> bool {
        self.lower <= Rational::zero() && Rational::zero() <= self.upper
    }
}

fn multiply_intervals(left: &Interval, right: &Interval) -> Result<Interval, String> {
    let candidates = [
        left.lower
            .checked_mul(&right.lower)
            .map_err(|e| e.to_string())?,
        left.lower
            .checked_mul(&right.upper)
            .map_err(|e| e.to_string())?,
        left.upper
            .checked_mul(&right.lower)
            .map_err(|e| e.to_string())?,
        left.upper
            .checked_mul(&right.upper)
            .map_err(|e| e.to_string())?,
    ];
    interval_from_candidates(candidates)
}

fn divide_intervals(left: &Interval, right: &Interval) -> Result<Interval, String> {
    if right.contains_zero() {
        return Err("division interval must not contain zero".to_owned());
    }
    let reciprocal = Interval::new(
        Rational::one()
            .checked_div(&right.upper)
            .map_err(|e| e.to_string())?,
        Rational::one()
            .checked_div(&right.lower)
            .map_err(|e| e.to_string())?,
    )?;
    multiply_intervals(left, &reciprocal)
}

fn pow_interval(interval: &Interval, exponent: i32) -> Result<Interval, String> {
    if exponent <= 0 {
        if exponent == 0 {
            return Interval::new(Rational::one(), Rational::one());
        }
        if interval.contains_zero() {
            return Err("negative exponent interval must not contain zero".to_owned());
        }
        let positive_exponent = exponent
            .checked_neg()
            .ok_or_else(|| "exponent is too small to negate".to_owned())?;
        let positive = pow_interval(interval, positive_exponent)?;
        return divide_intervals(&Interval::new(Rational::one(), Rational::one())?, &positive);
    }

    let lower = interval
        .lower
        .checked_pow_i32(exponent)
        .map_err(|e| e.to_string())?;
    let upper = interval
        .upper
        .checked_pow_i32(exponent)
        .map_err(|e| e.to_string())?;
    if exponent % 2 == 0 && interval.contains_zero() {
        let max = lower.max(upper);
        Interval::new(Rational::zero(), max)
    } else if lower <= upper {
        Interval::new(lower, upper)
    } else {
        Interval::new(upper, lower)
    }
}

fn polynomial_range(polynomial: &PolynomialInput, domain: &Interval) -> Result<Interval, String> {
    let mut points = vec![domain.lower.clone(), domain.upper.clone()];
    let derivative = match (PolynomialRequest::Derivative {
        polynomial: polynomial.clone(),
    })
    .evaluate()
    {
        PolynomialResponse::Polynomial { coefficients, .. } => PolynomialInput {
            variable: polynomial.variable.clone(),
            coefficients: coefficients
                .into_iter()
                .map(|value| Expr::Rational {
                    numerator: value.numerator,
                    denominator: value.denominator,
                })
                .collect(),
        },
        PolynomialResponse::Error { reason, .. } => return Err(reason),
        other => return Err(format!("unexpected derivative response {other:?}")),
    };
    match (PolynomialRequest::Solve {
        polynomial: derivative,
    })
    .evaluate()
    {
        PolynomialResponse::Roots { roots, .. } => {
            for root in roots {
                let root = Rational::parse(&root.numerator, &root.denominator)
                    .map_err(|e| e.to_string())?;
                if domain.lower <= root && root <= domain.upper {
                    points.push(root);
                }
            }
        }
        PolynomialResponse::Error { reason, .. }
            if reason == "zero polynomial has infinitely many roots" => {}
        PolynomialResponse::Error { reason, .. } => return Err(reason),
        other => return Err(format!("unexpected solve response {other:?}")),
    }

    let mut values = Vec::with_capacity(points.len());
    for point in points {
        values.push(evaluate_polynomial_at(polynomial, &point)?);
    }
    interval_from_candidates(values)
}

fn evaluate_polynomial_at(polynomial: &PolynomialInput, at: &Rational) -> Result<Rational, String> {
    match (PolynomialRequest::Evaluate {
        polynomial: polynomial.clone(),
        at: Expr::Rational {
            numerator: at.numerator().to_string(),
            denominator: at.denominator().to_string(),
        },
    })
    .evaluate()
    {
        PolynomialResponse::Value { exact, .. } => {
            Rational::parse(&exact.numerator, &exact.denominator).map_err(|e| e.to_string())
        }
        PolynomialResponse::Error { reason, .. } => Err(reason),
        other => Err(format!("unexpected evaluate response {other:?}")),
    }
}

fn interval_from_candidates<I>(candidates: I) -> Result<Interval, String>
where
    I: IntoIterator<Item = Rational>,
{
    let mut iter = candidates.into_iter();
    let Some(first) = iter.next() else {
        return Err("interval candidate set must not be empty".to_owned());
    };
    let mut lower = first.clone();
    let mut upper = first;
    for candidate in iter {
        lower = lower.min(candidate.clone());
        upper = upper.max(candidate);
    }
    Interval::new(lower, upper)
}

fn exact_rational(value: &Rational) -> ExactRational {
    ExactRational {
        numerator: value.numerator().to_string(),
        denominator: value.denominator().to_string(),
        display: value.to_string(),
    }
}

fn default_checks() -> Vec<IntervalCheck> {
    vec![
        IntervalCheck {
            name: "closed_interval_bounds_ordered".to_owned(),
            passed: true,
        },
        IntervalCheck {
            name: "bounds_are_exact_rationals".to_owned(),
            passed: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i32) -> Expr {
        Expr::Integer {
            value: value.to_string(),
        }
    }

    fn interval(lower: i32, upper: i32) -> IntervalInput {
        IntervalInput {
            lower: int(lower),
            upper: int(upper),
        }
    }

    fn poly(coefficients: Vec<i32>) -> PolynomialInput {
        PolynomialInput {
            variable: "x".to_owned(),
            coefficients: coefficients.into_iter().map(int).collect(),
        }
    }

    fn bounds(response: IntervalResponse) -> (String, String) {
        match response {
            IntervalResponse::Interval {
                lower,
                upper,
                checks,
                ..
            } => {
                assert_eq!(
                    checks,
                    vec![
                        IntervalCheck {
                            name: "closed_interval_bounds_ordered".to_owned(),
                            passed: true,
                        },
                        IntervalCheck {
                            name: "bounds_are_exact_rationals".to_owned(),
                            passed: true,
                        },
                    ]
                );
                (lower.display, upper.display)
            }
            other => panic!("expected interval response, got {other:?}"),
        }
    }

    #[test]
    fn performs_basic_interval_arithmetic() {
        assert_eq!(
            bounds(
                IntervalRequest::Add {
                    left: interval(1, 3),
                    right: interval(2, 4),
                }
                .evaluate()
            ),
            ("3".to_owned(), "7".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Mul {
                    left: interval(-2, 3),
                    right: interval(4, 5),
                }
                .evaluate()
            ),
            ("-10".to_owned(), "15".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Pow {
                    interval: interval(-2, 3),
                    exponent: 2,
                }
                .evaluate()
            ),
            ("0".to_owned(), "9".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Pow {
                    interval: interval(-2, 3),
                    exponent: 3,
                }
                .evaluate()
            ),
            ("-8".to_owned(), "27".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Pow {
                    interval: interval(2, 3),
                    exponent: 3,
                }
                .evaluate()
            ),
            ("8".to_owned(), "27".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Pow {
                    interval: interval(2, 4),
                    exponent: -1,
                }
                .evaluate()
            ),
            ("1/4".to_owned(), "1/2".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Div {
                    left: interval(1, 2),
                    right: interval(1, 2),
                }
                .evaluate()
            ),
            ("1/2".to_owned(), "2".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::Div {
                    left: interval(1, 2),
                    right: interval(-2, -1),
                }
                .evaluate()
            ),
            ("-2".to_owned(), "-1/2".to_owned())
        );
    }

    #[test]
    fn rejects_invalid_interval_operations() {
        assert!(matches!(
            (IntervalRequest::Sub {
                left: interval(3, 1),
                right: interval(0, 1),
            })
            .evaluate(),
            IntervalResponse::Error { reason, .. } if reason == "interval lower bound must be <= upper bound"
        ));
        assert!(matches!(
            (IntervalRequest::Div {
                left: interval(1, 2),
                right: interval(-1, 1),
            })
            .evaluate(),
            IntervalResponse::Error { reason, .. } if reason == "division interval must not contain zero"
        ));
        assert!(matches!(
            (IntervalRequest::Pow {
                interval: interval(-1, 1),
                exponent: -1,
            })
            .evaluate(),
            IntervalResponse::Error { reason, .. } if reason == "negative exponent interval must not contain zero"
        ));
        assert!(matches!(
            (IntervalRequest::Pow {
                interval: interval(1, 2),
                exponent: i32::MIN,
            })
            .evaluate(),
            IntervalResponse::Error { reason, .. } if reason == "exponent is too small to negate"
        ));
    }

    #[test]
    fn computes_polynomial_range_with_critical_point() {
        assert_eq!(
            bounds(
                IntervalRequest::PolynomialRange {
                    polynomial: poly(vec![1, 0, 1]),
                    domain: interval(-2, 3),
                }
                .evaluate()
            ),
            ("1".to_owned(), "10".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::PolynomialRange {
                    polynomial: poly(vec![0, -2, 1]),
                    domain: interval(0, 3),
                }
                .evaluate()
            ),
            ("-1".to_owned(), "3".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::PolynomialRange {
                    polynomial: poly(vec![5]),
                    domain: interval(-3, 7),
                }
                .evaluate()
            ),
            ("5".to_owned(), "5".to_owned())
        );
        assert_eq!(
            bounds(
                IntervalRequest::PolynomialRange {
                    polynomial: poly(vec![0, 0, 1]),
                    domain: interval(1, 3),
                }
                .evaluate()
            ),
            ("1".to_owned(), "9".to_owned())
        );
        assert!(matches!(
            (IntervalRequest::PolynomialRange {
                polynomial: poly(vec![0, 1, 0, 1]),
                domain: interval(-2, 2),
            })
            .evaluate(),
            IntervalResponse::Error { reason, .. } if reason == "quadratic has no real rational roots"
        ));
    }
}
