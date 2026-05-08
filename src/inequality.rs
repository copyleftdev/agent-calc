use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, ExactRational, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum InequalityRequest {
    Solve {
        inequality: InequalityInput,
        variable: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InequalityInput {
    pub left: Expr,
    pub relation: InequalityRelation,
    pub right: Expr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InequalityRelation {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum InequalityResponse {
    SolutionSet {
        contract_version: String,
        variable: String,
        set: SolutionSet,
        checks: Vec<InequalityCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SolutionSet {
    Empty,
    AllReals,
    Point {
        value: ExactRational,
    },
    NotPoint {
        value: ExactRational,
    },
    Interval {
        lower: Option<Bound>,
        upper: Option<Bound>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bound {
    pub value: ExactRational,
    pub inclusive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InequalityCheck {
    pub name: String,
    pub passed: bool,
}

impl InequalityRequest {
    pub fn evaluate(&self) -> InequalityResponse {
        match self.evaluate_inner() {
            Ok(response) => response,
            Err(reason) => InequalityResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<InequalityResponse, String> {
        match self {
            InequalityRequest::Solve {
                inequality,
                variable,
            } => solve_inequality(inequality, variable),
        }
    }
}

pub fn inequality_schema_json() -> Value {
    let expr_defs = crate::schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/inequality.json"),
        "title": "agent-calc calc1 inequality request",
        "description": "Typed exact single-variable affine inequality solve request over rational expressions.",
        "type": "object",
        "required": ["intent", "inequality", "variable"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "solve"},
            "inequality": {"$ref": "#/$defs/Inequality"},
            "variable": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"}
        },
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
            "Relation": {"enum": ["lt", "lte", "gt", "gte", "eq", "neq"]},
            "Inequality": {
                "type": "object",
                "required": ["left", "relation", "right"],
                "additionalProperties": false,
                "properties": {
                    "left": {"$ref": "#/$defs/Expr"},
                    "relation": {"$ref": "#/$defs/Relation"},
                    "right": {"$ref": "#/$defs/Expr"}
                }
            }
        }
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Affine {
    coefficient: Rational,
    constant: Rational,
}

fn solve_inequality(
    inequality: &InequalityInput,
    variable: &str,
) -> Result<InequalityResponse, String> {
    validate_symbol(variable)?;
    let left = affine_expr(&inequality.left, variable)?;
    let right = affine_expr(&inequality.right, variable)?;
    let coefficient = left
        .coefficient
        .checked_sub(&right.coefficient)
        .map_err(|e| e.to_string())?;
    let constant = left
        .constant
        .checked_sub(&right.constant)
        .map_err(|e| e.to_string())?;
    let set = solve_normalized(coefficient, constant, inequality.relation)?;
    Ok(InequalityResponse::SolutionSet {
        contract_version: CONTRACT_VERSION.to_owned(),
        variable: variable.to_owned(),
        set,
        checks: default_checks(),
    })
}

fn solve_normalized(
    coefficient: Rational,
    constant: Rational,
    relation: InequalityRelation,
) -> Result<SolutionSet, String> {
    let relation = match coefficient.cmp(&Rational::zero()) {
        Ordering::Equal => {
            return Ok(if constant_satisfies(&constant, relation) {
                SolutionSet::AllReals
            } else {
                SolutionSet::Empty
            });
        }
        Ordering::Less => flip_relation(relation),
        Ordering::Greater => relation,
    };
    let root = constant
        .checked_neg()
        .and_then(|value| value.checked_div(&coefficient))
        .map_err(|e| e.to_string())?;

    Ok(match relation {
        InequalityRelation::Lt => SolutionSet::Interval {
            lower: None,
            upper: Some(bound(root, false)),
        },
        InequalityRelation::Lte => SolutionSet::Interval {
            lower: None,
            upper: Some(bound(root, true)),
        },
        InequalityRelation::Gt => SolutionSet::Interval {
            lower: Some(bound(root, false)),
            upper: None,
        },
        InequalityRelation::Gte => SolutionSet::Interval {
            lower: Some(bound(root, true)),
            upper: None,
        },
        InequalityRelation::Eq => SolutionSet::Point {
            value: exact_rational(&root),
        },
        InequalityRelation::Neq => SolutionSet::NotPoint {
            value: exact_rational(&root),
        },
    })
}

fn constant_satisfies(value: &Rational, relation: InequalityRelation) -> bool {
    match relation {
        InequalityRelation::Lt => value < &Rational::zero(),
        InequalityRelation::Lte => value <= &Rational::zero(),
        InequalityRelation::Gt => value > &Rational::zero(),
        InequalityRelation::Gte => value >= &Rational::zero(),
        InequalityRelation::Eq => value == &Rational::zero(),
        InequalityRelation::Neq => value != &Rational::zero(),
    }
}

fn flip_relation(relation: InequalityRelation) -> InequalityRelation {
    match relation {
        InequalityRelation::Lt => InequalityRelation::Gt,
        InequalityRelation::Lte => InequalityRelation::Gte,
        InequalityRelation::Gt => InequalityRelation::Lt,
        InequalityRelation::Gte => InequalityRelation::Lte,
        InequalityRelation::Eq => InequalityRelation::Eq,
        InequalityRelation::Neq => InequalityRelation::Neq,
    }
}

fn affine_expr(expr: &Expr, variable: &str) -> Result<Affine, String> {
    match expr {
        Expr::Integer { .. } | Expr::Rational { .. } => Ok(constant(expr.evaluate()?)),
        Expr::Symbol { name } => {
            validate_symbol(name)?;
            if name == variable {
                Ok(Affine {
                    coefficient: Rational::one(),
                    constant: Rational::zero(),
                })
            } else {
                Err(format!(
                    "unsupported symbol `{name}` in inequality for `{variable}`"
                ))
            }
        }
        Expr::Add { left, right } => {
            add_affine(affine_expr(left, variable)?, affine_expr(right, variable)?)
        }
        Expr::Sub { left, right } => {
            sub_affine(affine_expr(left, variable)?, affine_expr(right, variable)?)
        }
        Expr::Mul { left, right } => mul_affine(
            affine_expr(left, variable)?,
            affine_expr(right, variable)?,
            variable,
        ),
        Expr::Div { left, right } => div_affine(
            affine_expr(left, variable)?,
            affine_expr(right, variable)?,
            variable,
        ),
        Expr::Pow { base, exponent } => pow_affine(base, *exponent, variable),
        Expr::Neg { value } => neg_affine(affine_expr(value, variable)?),
    }
}

fn constant(value: Rational) -> Affine {
    Affine {
        coefficient: Rational::zero(),
        constant: value,
    }
}

fn add_affine(left: Affine, right: Affine) -> Result<Affine, String> {
    Ok(Affine {
        coefficient: left
            .coefficient
            .checked_add(&right.coefficient)
            .map_err(|e| e.to_string())?,
        constant: left
            .constant
            .checked_add(&right.constant)
            .map_err(|e| e.to_string())?,
    })
}

fn sub_affine(left: Affine, right: Affine) -> Result<Affine, String> {
    Ok(Affine {
        coefficient: left
            .coefficient
            .checked_sub(&right.coefficient)
            .map_err(|e| e.to_string())?,
        constant: left
            .constant
            .checked_sub(&right.constant)
            .map_err(|e| e.to_string())?,
    })
}

fn neg_affine(value: Affine) -> Result<Affine, String> {
    Ok(Affine {
        coefficient: value.coefficient.checked_neg().map_err(|e| e.to_string())?,
        constant: value.constant.checked_neg().map_err(|e| e.to_string())?,
    })
}

fn mul_affine(left: Affine, right: Affine, variable: &str) -> Result<Affine, String> {
    if !left.coefficient.is_zero() && !right.coefficient.is_zero() {
        return Err(format!("nonlinear term involving `{variable}`"));
    }
    if left.coefficient.is_zero() {
        mul_by_constant(right, &left.constant)
    } else {
        mul_by_constant(left, &right.constant)
    }
}

fn div_affine(left: Affine, right: Affine, variable: &str) -> Result<Affine, String> {
    if !right.coefficient.is_zero() {
        return Err(format!(
            "division by expression containing `{variable}` is nonlinear"
        ));
    }
    if right.constant.is_zero() {
        return Err("denominator must not be zero".to_owned());
    }
    Ok(Affine {
        coefficient: left
            .coefficient
            .checked_div(&right.constant)
            .map_err(|e| e.to_string())?,
        constant: left
            .constant
            .checked_div(&right.constant)
            .map_err(|e| e.to_string())?,
    })
}

fn pow_affine(base: &Expr, exponent: i32, variable: &str) -> Result<Affine, String> {
    if exponent == 0 {
        return Ok(constant(Rational::one()));
    }
    if exponent == 1 {
        return affine_expr(base, variable);
    }

    let base = affine_expr(base, variable)?;
    if !base.coefficient.is_zero() {
        return Err(format!("nonlinear power involving `{variable}`"));
    }
    Ok(constant(
        base.constant
            .checked_pow_i32(exponent)
            .map_err(|e| e.to_string())?,
    ))
}

fn mul_by_constant(value: Affine, scalar: &Rational) -> Result<Affine, String> {
    Ok(Affine {
        coefficient: value
            .coefficient
            .checked_mul(scalar)
            .map_err(|e| e.to_string())?,
        constant: value
            .constant
            .checked_mul(scalar)
            .map_err(|e| e.to_string())?,
    })
}

fn bound(value: Rational, inclusive: bool) -> Bound {
    Bound {
        value: exact_rational(&value),
        inclusive,
    }
}

fn exact_rational(value: &Rational) -> ExactRational {
    ExactRational {
        numerator: value.numerator().to_string(),
        denominator: value.denominator().to_string(),
        display: value.to_string(),
    }
}

fn validate_symbol(symbol: &str) -> Result<(), String> {
    if is_valid_symbol_name(symbol) {
        Ok(())
    } else {
        Err(format!("invalid symbol name `{symbol}`"))
    }
}

fn is_valid_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn default_checks() -> Vec<InequalityCheck> {
    vec![
        InequalityCheck {
            name: "affine_inequality_isolated".to_owned(),
            passed: true,
        },
        InequalityCheck {
            name: "solution_set_bounds_are_exact".to_owned(),
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

    fn symbol(name: &str) -> Expr {
        Expr::Symbol {
            name: name.to_owned(),
        }
    }

    fn add(left: Expr, right: Expr) -> Expr {
        Expr::Add {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn mul(left: Expr, right: Expr) -> Expr {
        Expr::Mul {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn div(left: Expr, right: Expr) -> Expr {
        Expr::Div {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn pow(base: Expr, exponent: i32) -> Expr {
        Expr::Pow {
            base: Box::new(base),
            exponent,
        }
    }

    fn solve(left: Expr, relation: InequalityRelation, right: Expr) -> InequalityResponse {
        InequalityRequest::Solve {
            inequality: InequalityInput {
                left,
                relation,
                right,
            },
            variable: "x".to_owned(),
        }
        .evaluate()
    }

    fn expected_checks() -> Vec<InequalityCheck> {
        vec![
            InequalityCheck {
                name: "affine_inequality_isolated".to_owned(),
                passed: true,
            },
            InequalityCheck {
                name: "solution_set_bounds_are_exact".to_owned(),
                passed: true,
            },
        ]
    }

    #[test]
    fn solves_positive_and_negative_coefficient_bounds() {
        assert_eq!(
            solve(
                add(mul(int(2), symbol("x")), int(3)),
                InequalityRelation::Lt,
                int(7)
            ),
            InequalityResponse::SolutionSet {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "x".to_owned(),
                set: SolutionSet::Interval {
                    lower: None,
                    upper: Some(bound(Rational::integer(2), false)),
                },
                checks: expected_checks(),
            }
        );
        assert_eq!(
            solve(
                add(mul(int(-2), symbol("x")), int(3)),
                InequalityRelation::Lte,
                int(7)
            ),
            InequalityResponse::SolutionSet {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "x".to_owned(),
                set: SolutionSet::Interval {
                    lower: Some(bound(Rational::integer(-2), true)),
                    upper: None,
                },
                checks: expected_checks(),
            }
        );
    }

    #[test]
    fn handles_equality_neq_and_constant_truth_values() {
        assert!(matches!(
            solve(add(symbol("x"), int(1)), InequalityRelation::Eq, int(3)),
            InequalityResponse::SolutionSet { set: SolutionSet::Point { value }, .. } if value.display == "2"
        ));
        assert!(matches!(
            solve(add(symbol("x"), int(1)), InequalityRelation::Neq, int(3)),
            InequalityResponse::SolutionSet { set: SolutionSet::NotPoint { value }, .. } if value.display == "2"
        ));
        assert!(matches!(
            solve(int(1), InequalityRelation::Lt, int(2)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(int(1), InequalityRelation::Gt, int(2)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
    }

    #[test]
    fn covers_constant_relation_truth_table_at_zero() {
        assert!(matches!(
            solve(int(0), InequalityRelation::Lt, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
        assert!(matches!(
            solve(int(0), InequalityRelation::Lte, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(int(0), InequalityRelation::Gt, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
        assert!(matches!(
            solve(int(0), InequalityRelation::Gte, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(int(0), InequalityRelation::Eq, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(int(0), InequalityRelation::Neq, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
    }

    #[test]
    fn covers_zero_coefficient_nonconstant_truth_values() {
        assert!(matches!(
            solve(int(-1), InequalityRelation::Lt, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(int(1), InequalityRelation::Lte, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
        assert!(matches!(
            solve(int(1), InequalityRelation::Gt, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(int(-1), InequalityRelation::Gte, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
        assert!(matches!(
            solve(int(1), InequalityRelation::Eq, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
        assert!(matches!(
            solve(int(1), InequalityRelation::Neq, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
    }

    #[test]
    fn rejects_invalid_or_nonlinear_inputs() {
        assert!(matches!(
            (InequalityRequest::Solve {
                inequality: InequalityInput {
                    left: symbol("x"),
                    relation: InequalityRelation::Lt,
                    right: int(1),
                },
                variable: "1x".to_owned(),
            })
            .evaluate(),
            InequalityResponse::Error { reason, .. } if reason == "invalid symbol name `1x`"
        ));
        assert!(matches!(
            solve(symbol("y"), InequalityRelation::Lt, int(1)),
            InequalityResponse::Error { reason, .. } if reason == "unsupported symbol `y` in inequality for `x`"
        ));
        assert!(matches!(
            solve(mul(symbol("x"), symbol("x")), InequalityRelation::Lt, int(1)),
            InequalityResponse::Error { reason, .. } if reason == "nonlinear term involving `x`"
        ));
        assert!(matches!(
            solve(div(int(1), symbol("x")), InequalityRelation::Lt, int(1)),
            InequalityResponse::Error { reason, .. } if reason == "division by expression containing `x` is nonlinear"
        ));
        assert!(matches!(
            solve(pow(symbol("x"), 2), InequalityRelation::Lt, int(1)),
            InequalityResponse::Error { reason, .. } if reason == "nonlinear power involving `x`"
        ));
    }

    #[test]
    fn supports_constant_powers_and_power_identity_cases() {
        assert!(matches!(
            solve(pow(symbol("x"), 0), InequalityRelation::Eq, int(1)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(pow(symbol("x"), 1), InequalityRelation::Gte, int(0)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Interval {
                    lower: Some(Bound {
                        inclusive: true,
                        ..
                    }),
                    upper: None
                },
                ..
            }
        ));
        assert!(matches!(
            solve(pow(int(2), 3), InequalityRelation::Eq, int(8)),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
    }

    #[test]
    fn equality_with_identical_affine_terms_is_all_reals() {
        assert!(matches!(
            solve(symbol("x"), InequalityRelation::Eq, symbol("x")),
            InequalityResponse::SolutionSet {
                set: SolutionSet::AllReals,
                ..
            }
        ));
        assert!(matches!(
            solve(symbol("x"), InequalityRelation::Neq, symbol("x")),
            InequalityResponse::SolutionSet {
                set: SolutionSet::Empty,
                ..
            }
        ));
    }

    #[test]
    fn accepts_underscore_symbol_names() {
        assert!(matches!(
            (InequalityRequest::Solve {
                inequality: InequalityInput {
                    left: symbol("_x"),
                    relation: InequalityRelation::Gte,
                    right: int(0),
                },
                variable: "_x".to_owned(),
            })
            .evaluate(),
            InequalityResponse::SolutionSet { .. }
        ));
        assert!(matches!(
            (InequalityRequest::Solve {
                inequality: InequalityInput {
                    left: symbol("x_1"),
                    relation: InequalityRelation::Gte,
                    right: int(0),
                },
                variable: "x_1".to_owned(),
            })
            .evaluate(),
            InequalityResponse::SolutionSet { .. }
        ));
    }
}
