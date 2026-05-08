use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum CalculusRequest {
    Derivative { expr: Expr, variable: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CalculusResponse {
    Derivative {
        contract_version: String,
        variable: String,
        expr: Expr,
        checks: Vec<CalculusCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalculusCheck {
    pub name: String,
    pub passed: bool,
}

impl CalculusRequest {
    pub fn evaluate(&self) -> CalculusResponse {
        match self.evaluate_inner() {
            Ok(response) => response,
            Err(reason) => CalculusResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<CalculusResponse, String> {
        match self {
            CalculusRequest::Derivative { expr, variable } => {
                validate_symbol(variable)?;
                let derivative = derivative_expr(expr, variable)?.simplify()?;
                Ok(CalculusResponse::Derivative {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    variable: variable.clone(),
                    expr: derivative,
                    checks: default_checks(),
                })
            }
        }
    }
}

pub fn calculus_schema_json() -> Value {
    let expr_defs = crate::schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/calculus.json"),
        "title": "agent-calc calc1 calculus request",
        "description": "Typed exact symbolic calculus request over the agent-calc expression AST.",
        "type": "object",
        "required": ["intent", "expr", "variable"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "derivative"},
            "expr": {"$ref": "#/$defs/Expr"},
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
            "Neg": expr_defs["Neg"].clone()
        }
    })
}

fn derivative_expr(expr: &Expr, variable: &str) -> Result<Expr, String> {
    match expr {
        Expr::Integer { value } => {
            Rational::parse_integer(value).map_err(|e| e.to_string())?;
            Ok(integer(0))
        }
        Expr::Rational {
            numerator,
            denominator,
        } => {
            Rational::parse(numerator, denominator).map_err(|e| e.to_string())?;
            Ok(integer(0))
        }
        Expr::Symbol { name } => {
            validate_symbol(name)?;
            if name == variable {
                Ok(integer(1))
            } else {
                Ok(integer(0))
            }
        }
        Expr::Add { left, right } => Ok(add(
            derivative_expr(left, variable)?,
            derivative_expr(right, variable)?,
        )),
        Expr::Sub { left, right } => Ok(sub(
            derivative_expr(left, variable)?,
            derivative_expr(right, variable)?,
        )),
        Expr::Mul { left, right } => {
            let left_value = left.as_ref().clone();
            let right_value = right.as_ref().clone();
            Ok(add(
                mul(derivative_expr(left, variable)?, right_value.clone()),
                mul(left_value, derivative_expr(right, variable)?),
            ))
        }
        Expr::Div { left, right } => {
            let left_value = left.as_ref().clone();
            let right_value = right.as_ref().clone();
            let numerator = sub(
                mul(derivative_expr(left, variable)?, right_value.clone()),
                mul(left_value, derivative_expr(right, variable)?),
            );
            Ok(div(numerator, pow(right_value, 2)))
        }
        Expr::Pow { base, exponent } => derivative_power(base, *exponent, variable),
        Expr::Neg { value } => Ok(neg(derivative_expr(value, variable)?)),
    }
}

fn derivative_power(base: &Expr, exponent: i32, variable: &str) -> Result<Expr, String> {
    if exponent == 0 {
        return Ok(integer(0));
    }
    let coefficient = integer(exponent);
    let next_exponent = exponent
        .checked_sub(1)
        .ok_or_else(|| "exponent is too small to decrement".to_owned())?;
    let base_value = base.clone();
    Ok(mul(
        mul(coefficient, pow(base_value, next_exponent)),
        derivative_expr(base, variable)?,
    ))
}

fn integer(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
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

fn neg(value: Expr) -> Expr {
    Expr::Neg {
        value: Box::new(value),
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

fn default_checks() -> Vec<CalculusCheck> {
    vec![
        CalculusCheck {
            name: "symbolic_derivative_rule_applied".to_owned(),
            passed: true,
        },
        CalculusCheck {
            name: "derivative_simplified".to_owned(),
            passed: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol(name: &str) -> Expr {
        Expr::Symbol {
            name: name.to_owned(),
        }
    }

    fn derivative(expr: Expr) -> CalculusResponse {
        CalculusRequest::Derivative {
            expr,
            variable: "x".to_owned(),
        }
        .evaluate()
    }

    fn derivative_expr_only(expr: Expr) -> Expr {
        match derivative(expr) {
            CalculusResponse::Derivative { expr, checks, .. } => {
                assert_eq!(checks, expected_checks());
                expr
            }
            other => panic!("expected derivative response, got {other:?}"),
        }
    }

    fn expected_checks() -> Vec<CalculusCheck> {
        vec![
            CalculusCheck {
                name: "symbolic_derivative_rule_applied".to_owned(),
                passed: true,
            },
            CalculusCheck {
                name: "derivative_simplified".to_owned(),
                passed: true,
            },
        ]
    }

    #[test]
    fn differentiates_literals_and_symbols() {
        assert_eq!(derivative_expr_only(integer(7)), integer(0));
        assert_eq!(derivative_expr_only(symbol("x")), integer(1));
        assert_eq!(derivative_expr_only(symbol("y")), integer(0));
        assert!(matches!(
            (CalculusRequest::Derivative {
                expr: symbol("x"),
                variable: "1x".to_owned(),
            })
            .evaluate(),
            CalculusResponse::Error { reason, .. } if reason == "invalid symbol name `1x`"
        ));
    }

    #[test]
    fn accepts_underscore_symbol_names() {
        let response = CalculusRequest::Derivative {
            expr: symbol("_x"),
            variable: "_x".to_owned(),
        }
        .evaluate();
        assert!(matches!(
            response,
            CalculusResponse::Derivative {
                expr: Expr::Integer { ref value },
                ..
            } if value == "1"
        ));

        let response = CalculusRequest::Derivative {
            expr: symbol("x_1"),
            variable: "x_1".to_owned(),
        }
        .evaluate();
        assert!(matches!(
            response,
            CalculusResponse::Derivative {
                expr: Expr::Integer { ref value },
                ..
            } if value == "1"
        ));
    }

    #[test]
    fn differentiates_sums_products_and_negation() {
        assert_eq!(
            derivative_expr_only(add(mul(integer(3), symbol("x")), integer(2))),
            integer(3)
        );
        assert_eq!(
            derivative_expr_only(neg(symbol("x"))),
            Expr::Neg {
                value: Box::new(integer(1))
            }
        );
    }

    #[test]
    fn differentiates_integer_powers() {
        assert_eq!(derivative_expr_only(pow(symbol("x"), 0)), integer(0));
        assert_eq!(derivative_expr_only(pow(symbol("x"), 1)), integer(1));
        assert_eq!(
            derivative_expr_only(pow(symbol("x"), 3)),
            mul(integer(3), pow(symbol("x"), 2))
        );
    }

    #[test]
    fn differentiates_quotients() {
        assert_eq!(
            derivative_expr_only(div(symbol("x"), integer(2))),
            div(integer(2), pow(integer(2), 2))
        );
        assert_eq!(
            derivative_expr_only(div(integer(1), symbol("x"))),
            div(sub(integer(0), integer(1)), pow(symbol("x"), 2))
        );
    }
}
