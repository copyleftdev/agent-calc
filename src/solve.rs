use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, ExactRational, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum SolveRequest {
    Solve {
        equation: EquationInput,
        variable: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EquationInput {
    pub left: Expr,
    pub right: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SolveResponse {
    Solutions {
        contract_version: String,
        variable: String,
        solutions: Vec<ExactRational>,
        checks: Vec<SolveCheck>,
    },
    NoSolution {
        contract_version: String,
        checks: Vec<SolveCheck>,
    },
    InfiniteSolutions {
        contract_version: String,
        checks: Vec<SolveCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolveCheck {
    pub name: String,
    pub passed: bool,
}

impl SolveRequest {
    pub fn evaluate(&self) -> SolveResponse {
        match self.evaluate_inner() {
            Ok(response) => response,
            Err(reason) => SolveResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<SolveResponse, String> {
        match self {
            SolveRequest::Solve { equation, variable } => solve_equation(equation, variable),
        }
    }
}

pub fn solve_schema_json() -> Value {
    let expr_defs = crate::schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/solve.json"),
        "title": "agent-calc calc1 solve request",
        "description": "Typed exact single-variable symbolic equation solve request over affine rational expressions.",
        "type": "object",
        "required": ["intent", "equation", "variable"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "solve"},
            "equation": {"$ref": "#/$defs/Equation"},
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
            "Equation": {
                "type": "object",
                "required": ["left", "right"],
                "additionalProperties": false,
                "properties": {
                    "left": {"$ref": "#/$defs/Expr"},
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

fn solve_equation(equation: &EquationInput, variable: &str) -> Result<SolveResponse, String> {
    validate_symbol(variable)?;
    let left = affine_expr(&equation.left, variable)?;
    let right = affine_expr(&equation.right, variable)?;
    let coefficient = left
        .coefficient
        .checked_sub(&right.coefficient)
        .map_err(|e| e.to_string())?;
    let constant = left
        .constant
        .checked_sub(&right.constant)
        .map_err(|e| e.to_string())?;

    if coefficient.is_zero() {
        if constant.is_zero() {
            return Ok(SolveResponse::InfiniteSolutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                checks: default_checks(),
            });
        }
        return Ok(SolveResponse::NoSolution {
            contract_version: CONTRACT_VERSION.to_owned(),
            checks: default_checks(),
        });
    }

    let solution = constant
        .checked_neg()
        .and_then(|value| value.checked_div(&coefficient))
        .map_err(|e| e.to_string())?;
    Ok(SolveResponse::Solutions {
        contract_version: CONTRACT_VERSION.to_owned(),
        variable: variable.to_owned(),
        solutions: vec![exact_rational(&solution)],
        checks: default_checks(),
    })
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
                    "unsupported symbol `{name}` in solve for `{variable}`"
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
        _ => Err("unsupported: nonlinear or transcendental expression in solve".to_owned()),
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

fn default_checks() -> Vec<SolveCheck> {
    vec![
        SolveCheck {
            name: "affine_expression_isolated".to_owned(),
            passed: true,
        },
        SolveCheck {
            name: "solutions_are_exact_rationals".to_owned(),
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

    fn sub(left: Expr, right: Expr) -> Expr {
        Expr::Sub {
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

    fn solve(left: Expr, right: Expr) -> SolveResponse {
        SolveRequest::Solve {
            equation: EquationInput { left, right },
            variable: "x".to_owned(),
        }
        .evaluate()
    }

    fn expected_checks() -> Vec<SolveCheck> {
        vec![
            SolveCheck {
                name: "affine_expression_isolated".to_owned(),
                passed: true,
            },
            SolveCheck {
                name: "solutions_are_exact_rationals".to_owned(),
                passed: true,
            },
        ]
    }

    #[test]
    fn solves_exact_affine_equations() {
        assert_eq!(
            solve(add(mul(int(2), symbol("x")), int(3)), int(7)),
            SolveResponse::Solutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "x".to_owned(),
                solutions: vec![exact_rational(&Rational::integer(2))],
                checks: expected_checks(),
            }
        );
        assert_eq!(
            solve(div(sub(symbol("x"), int(1)), int(2)), int(3)),
            SolveResponse::Solutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "x".to_owned(),
                solutions: vec![exact_rational(&Rational::integer(7))],
                checks: expected_checks(),
            }
        );
    }

    #[test]
    fn classifies_degenerate_affine_equations() {
        assert_eq!(
            solve(add(symbol("x"), int(1)), add(symbol("x"), int(2))),
            SolveResponse::NoSolution {
                contract_version: CONTRACT_VERSION.to_owned(),
                checks: expected_checks(),
            }
        );
        assert_eq!(
            solve(add(symbol("x"), int(1)), add(symbol("x"), int(1))),
            SolveResponse::InfiniteSolutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                checks: expected_checks(),
            }
        );
    }

    #[test]
    fn supports_constant_powers_but_rejects_nonlinear_variable_terms() {
        assert_eq!(
            solve(add(symbol("x"), pow(int(2), 3)), int(10)),
            SolveResponse::Solutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "x".to_owned(),
                solutions: vec![exact_rational(&Rational::integer(2))],
                checks: expected_checks(),
            }
        );
        assert!(matches!(
            solve(pow(symbol("x"), 2), int(4)),
            SolveResponse::Error { reason, .. } if reason == "nonlinear power involving `x`"
        ));
        assert!(matches!(
            solve(mul(symbol("x"), symbol("x")), int(4)),
            SolveResponse::Error { reason, .. } if reason == "nonlinear term involving `x`"
        ));
    }

    #[test]
    fn rejects_invalid_symbols_and_variable_denominators() {
        assert!(matches!(
            (SolveRequest::Solve {
                equation: EquationInput {
                    left: symbol("x"),
                    right: int(1),
                },
                variable: "1x".to_owned(),
            })
            .evaluate(),
            SolveResponse::Error { reason, .. } if reason == "invalid symbol name `1x`"
        ));
        assert!(matches!(
            solve(symbol("y"), int(1)),
            SolveResponse::Error { reason, .. } if reason == "unsupported symbol `y` in solve for `x`"
        ));
        assert!(matches!(
            solve(div(int(1), symbol("x")), int(1)),
            SolveResponse::Error { reason, .. } if reason == "division by expression containing `x` is nonlinear"
        ));
        assert!(matches!(
            solve(div(symbol("x"), int(0)), int(1)),
            SolveResponse::Error { reason, .. } if reason == "denominator must not be zero"
        ));
    }

    #[test]
    fn accepts_variables_starting_with_underscore() {
        let response = SolveRequest::Solve {
            equation: EquationInput {
                left: add(symbol("_x"), int(1)),
                right: int(3),
            },
            variable: "_x".to_owned(),
        }
        .evaluate();

        assert_eq!(
            response,
            SolveResponse::Solutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "_x".to_owned(),
                solutions: vec![exact_rational(&Rational::integer(2))],
                checks: expected_checks(),
            }
        );
    }

    #[test]
    fn accepts_variables_containing_underscore() {
        let response = SolveRequest::Solve {
            equation: EquationInput {
                left: add(symbol("x_1"), int(1)),
                right: int(3),
            },
            variable: "x_1".to_owned(),
        }
        .evaluate();

        assert_eq!(
            response,
            SolveResponse::Solutions {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable: "x_1".to_owned(),
                solutions: vec![exact_rational(&Rational::integer(2))],
                checks: expected_checks(),
            }
        );
    }
}
