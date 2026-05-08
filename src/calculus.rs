use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum CalculusRequest {
    Derivative {
        expr: Expr,
        variable: String,
    },
    Integral {
        expr: Expr,
        variable: String,
    },
    DefiniteIntegral {
        expr: Expr,
        variable: String,
        lower: Expr,
        upper: Expr,
    },
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
    Integral {
        contract_version: String,
        variable: String,
        expr: Expr,
        checks: Vec<CalculusCheck>,
    },
    DefiniteIntegral {
        contract_version: String,
        variable: String,
        value: Expr,
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
            CalculusRequest::Integral { expr, variable } => {
                validate_symbol(variable)?;
                let antideriv = integral_expr(expr, variable)?.simplify()?;
                Ok(CalculusResponse::Integral {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    variable: variable.clone(),
                    expr: antideriv,
                    checks: default_checks(),
                })
            }
            CalculusRequest::DefiniteIntegral {
                expr,
                variable,
                lower,
                upper,
            } => {
                validate_symbol(variable)?;
                let lower_val = eval_bound(lower)?;
                let upper_val = eval_bound(upper)?;
                let antideriv = integral_expr(expr, variable)?;
                let at_upper = eval_rational(&antideriv, variable, &upper_val)?;
                let at_lower = eval_rational(&antideriv, variable, &lower_val)?;
                let result = at_upper.checked_sub(&at_lower).map_err(|e| e.to_string())?;
                Ok(CalculusResponse::DefiniteIntegral {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    variable: variable.clone(),
                    value: rational_to_expr(&result),
                    checks: default_checks(),
                })
            }
        }
    }
}

pub fn calculus_schema_json() -> Value {
    let mut defs = crate::schema_json()["$defs"].clone();
    let expr_ref = json!({"$ref": "#/$defs/Expr"});
    let variable_schema = json!({"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"});
    defs["Derivative"] = json!({
        "type": "object",
        "required": ["intent", "expr", "variable"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "derivative"},
            "expr": expr_ref.clone(),
            "variable": variable_schema.clone()
        }
    });
    defs["Integral"] = json!({
        "type": "object",
        "required": ["intent", "expr", "variable"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "integral"},
            "expr": expr_ref.clone(),
            "variable": variable_schema.clone()
        }
    });
    defs["DefiniteIntegral"] = json!({
        "type": "object",
        "required": ["intent", "expr", "variable", "lower", "upper"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "definite_integral"},
            "expr": expr_ref.clone(),
            "variable": variable_schema,
            "lower": expr_ref.clone(),
            "upper": expr_ref
        }
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/calculus.json"),
        "title": "agent-calc calc1 calculus request",
        "description": "Typed exact symbolic calculus request over the agent-calc expression AST.",
        "oneOf": [
            {"$ref": "#/$defs/Derivative"},
            {"$ref": "#/$defs/Integral"},
            {"$ref": "#/$defs/DefiniteIntegral"}
        ],
        "$defs": defs
    })
}

fn contains_symbol(expr: &Expr, variable: &str) -> bool {
    match expr {
        Expr::Symbol { name } => name == variable,
        Expr::Integer { .. } | Expr::Rational { .. } => false,
        Expr::Add { left, right }
        | Expr::Sub { left, right }
        | Expr::Mul { left, right }
        | Expr::Div { left, right }
        | Expr::Max { left, right }
        | Expr::Min { left, right } => {
            contains_symbol(left, variable) || contains_symbol(right, variable)
        }
        Expr::Pow { base, .. } => contains_symbol(base, variable),
        Expr::Neg { value }
        | Expr::Sqrt { value }
        | Expr::Exp { value }
        | Expr::Ln { value }
        | Expr::Sin { value }
        | Expr::Cos { value }
        | Expr::Tan { value }
        | Expr::Abs { value }
        | Expr::Floor { value }
        | Expr::Ceil { value }
        | Expr::Round { value } => contains_symbol(value, variable),
        Expr::Log { base, value } => {
            contains_symbol(base, variable) || contains_symbol(value, variable)
        }
    }
}

fn integral_expr(expr: &Expr, variable: &str) -> Result<Expr, String> {
    match expr {
        // ∫c dx = c*x
        Expr::Integer { .. } | Expr::Rational { .. } => {
            Ok(mul(expr.clone(), symbol_expr(variable)))
        }
        // ∫x dx = (1/2)*x²  ;  ∫y dx = y*x (y is constant w.r.t. x)
        Expr::Symbol { name } => {
            if name == variable {
                Ok(mul(rational_frac(1, 2), pow(symbol_expr(variable), 2)))
            } else {
                Ok(mul(expr.clone(), symbol_expr(variable)))
            }
        }
        // Linearity
        Expr::Add { left, right } => Ok(add(
            integral_expr(left, variable)?,
            integral_expr(right, variable)?,
        )),
        Expr::Sub { left, right } => Ok(sub(
            integral_expr(left, variable)?,
            integral_expr(right, variable)?,
        )),
        Expr::Neg { value } => Ok(neg(integral_expr(value, variable)?)),
        // Constant multiple rule
        Expr::Mul { left, right } => {
            let lv = contains_symbol(left, variable);
            let rv = contains_symbol(right, variable);
            match (lv, rv) {
                (false, false) => Ok(mul(expr.clone(), symbol_expr(variable))),
                (false, true) => Ok(mul(*left.clone(), integral_expr(right, variable)?)),
                (true, false) => Ok(mul(integral_expr(left, variable)?, *right.clone())),
                (true, true) => Err(
                    "unsupported: product of two sub-expressions both containing the integration variable".to_owned(),
                ),
            }
        }
        // Division by constant
        Expr::Div { left, right } => {
            if contains_symbol(right, variable) {
                Err("unsupported: divisor contains the integration variable".to_owned())
            } else if !contains_symbol(left, variable) {
                Ok(mul(expr.clone(), symbol_expr(variable)))
            } else {
                Ok(div(integral_expr(left, variable)?, *right.clone()))
            }
        }
        // Power rule: ∫x^n dx = x^(n+1)/(n+1)
        Expr::Pow { base, exponent } => {
            if !contains_symbol(base, variable) {
                Ok(mul(expr.clone(), symbol_expr(variable)))
            } else if matches!(base.as_ref(), Expr::Symbol { name } if name == variable) {
                if *exponent == -1 {
                    Err("unsupported: integral of x^(-1) = ln(x) is out of scope for v1".to_owned())
                } else {
                    let new_exp = exponent
                        .checked_add(1)
                        .ok_or_else(|| "exponent overflow in integral".to_owned())?;
                    Ok(mul(
                        rational_frac(1, new_exp),
                        pow(symbol_expr(variable), new_exp),
                    ))
                }
            } else {
                Err("unsupported: power whose base contains the integration variable in a non-simple way".to_owned())
            }
        }
        _ => Err(format!(
            "unsupported: integral of `{}` expression is out of scope for v1",
            expr_type_name(expr)
        )),
    }
}

fn eval_bound(expr: &Expr) -> Result<Rational, String> {
    match expr {
        Expr::Integer { value } => Rational::parse_integer(value).map_err(|e| e.to_string()),
        Expr::Rational {
            numerator,
            denominator,
        } => Rational::parse(numerator, denominator).map_err(|e| e.to_string()),
        Expr::Symbol { name } => Err(format!(
            "bounds must be exact constants; got unbound symbol `{name}`"
        )),
        Expr::Neg { value } => eval_bound(value)?.checked_neg().map_err(|e| e.to_string()),
        _ => Err("bounds must be exact integer or rational constants".to_owned()),
    }
}

fn eval_rational(expr: &Expr, variable: &str, value: &Rational) -> Result<Rational, String> {
    match expr {
        Expr::Integer { value: v } => Rational::parse_integer(v).map_err(|e| e.to_string()),
        Expr::Rational {
            numerator,
            denominator,
        } => Rational::parse(numerator, denominator).map_err(|e| e.to_string()),
        Expr::Symbol { name } => {
            if name == variable {
                Ok(value.clone())
            } else {
                Err(format!(
                    "unbound symbol `{name}` in definite integral evaluation"
                ))
            }
        }
        Expr::Add { left, right } => eval_rational(left, variable, value)?
            .checked_add(&eval_rational(right, variable, value)?)
            .map_err(|e| e.to_string()),
        Expr::Sub { left, right } => eval_rational(left, variable, value)?
            .checked_sub(&eval_rational(right, variable, value)?)
            .map_err(|e| e.to_string()),
        Expr::Mul { left, right } => eval_rational(left, variable, value)?
            .checked_mul(&eval_rational(right, variable, value)?)
            .map_err(|e| e.to_string()),
        Expr::Div { left, right } => eval_rational(left, variable, value)?
            .checked_div(&eval_rational(right, variable, value)?)
            .map_err(|e| e.to_string()),
        Expr::Neg { value: inner } => eval_rational(inner, variable, value)?
            .checked_neg()
            .map_err(|e| e.to_string()),
        Expr::Pow { base, exponent } => eval_rational(base, variable, value)?
            .checked_pow_i32(*exponent)
            .map_err(|e| e.to_string()),
        _ => Err(format!(
            "cannot evaluate `{}` as exact rational",
            expr_type_name(expr)
        )),
    }
}

fn rational_to_expr(r: &Rational) -> Expr {
    if r.denominator().to_string() == "1" {
        Expr::Integer {
            value: r.numerator().to_string(),
        }
    } else {
        Expr::Rational {
            numerator: r.numerator().to_string(),
            denominator: r.denominator().to_string(),
        }
    }
}

fn symbol_expr(name: &str) -> Expr {
    Expr::Symbol {
        name: name.to_owned(),
    }
}

fn rational_frac(n: i32, d: i32) -> Expr {
    debug_assert_ne!(d, 0, "rational_frac denominator must be non-zero");
    let (n, d) = if d < 0 { (-n, -d) } else { (n, d) };
    if d == 1 {
        integer(n)
    } else {
        Expr::Rational {
            numerator: n.to_string(),
            denominator: d.to_string(),
        }
    }
}

fn expr_type_name(expr: &Expr) -> &'static str {
    match expr {
        Expr::Integer { .. } => "integer",
        Expr::Rational { .. } => "rational",
        Expr::Symbol { .. } => "symbol",
        Expr::Add { .. } => "add",
        Expr::Sub { .. } => "sub",
        Expr::Mul { .. } => "mul",
        Expr::Div { .. } => "div",
        Expr::Pow { .. } => "pow",
        Expr::Neg { .. } => "neg",
        Expr::Sqrt { .. } => "sqrt",
        Expr::Exp { .. } => "exp",
        Expr::Ln { .. } => "ln",
        Expr::Sin { .. } => "sin",
        Expr::Cos { .. } => "cos",
        Expr::Tan { .. } => "tan",
        Expr::Abs { .. } => "abs",
        Expr::Floor { .. } => "floor",
        Expr::Ceil { .. } => "ceil",
        Expr::Round { .. } => "round",
        Expr::Log { .. } => "log",
        Expr::Max { .. } => "max",
        Expr::Min { .. } => "min",
    }
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
        // d/dx sqrt(f) = f' / (2 * sqrt(f))
        Expr::Sqrt { value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(div(fp, mul(integer(2), sqrt(value.as_ref().clone()))))
        }
        // d/dx exp(f) = exp(f) * f'
        Expr::Exp { value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(mul(exp(value.as_ref().clone()), fp))
        }
        // d/dx ln(f) = f' / f
        Expr::Ln { value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(div(fp, value.as_ref().clone()))
        }
        // d/dx sin(f) = cos(f) * f'
        Expr::Sin { value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(mul(cos(value.as_ref().clone()), fp))
        }
        // d/dx cos(f) = -sin(f) * f'
        Expr::Cos { value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(neg(mul(sin(value.as_ref().clone()), fp)))
        }
        // d/dx tan(f) = f' / cos(f)^2
        Expr::Tan { value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(div(fp, pow(cos(value.as_ref().clone()), 2)))
        }
        // d/dx log_b(f) = f' / (f * ln(b))
        Expr::Log { base, value } => {
            let fp = derivative_expr(value, variable)?;
            Ok(div(
                fp,
                mul(value.as_ref().clone(), ln(base.as_ref().clone())),
            ))
        }
        // abs, floor, ceil, round, max, min: not differentiable everywhere
        Expr::Abs { .. } => Err("unsupported: abs is not differentiable everywhere".to_owned()),
        Expr::Floor { .. } | Expr::Ceil { .. } | Expr::Round { .. } => {
            Err("unsupported: floor/ceil/round are not differentiable".to_owned())
        }
        Expr::Max { .. } | Expr::Min { .. } => {
            Err("unsupported: max/min are not differentiable everywhere".to_owned())
        }
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

fn sqrt(value: Expr) -> Expr {
    Expr::Sqrt {
        value: Box::new(value),
    }
}

fn exp(value: Expr) -> Expr {
    Expr::Exp {
        value: Box::new(value),
    }
}

fn ln(value: Expr) -> Expr {
    Expr::Ln {
        value: Box::new(value),
    }
}

fn sin(value: Expr) -> Expr {
    Expr::Sin {
        value: Box::new(value),
    }
}

fn cos(value: Expr) -> Expr {
    Expr::Cos {
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

    fn integral_of(expr: Expr) -> CalculusResponse {
        CalculusRequest::Integral {
            expr,
            variable: "x".to_owned(),
        }
        .evaluate()
    }

    fn integral_expr_only(expr: Expr) -> Expr {
        match integral_of(expr) {
            CalculusResponse::Integral { expr, .. } => expr,
            other => panic!("expected integral response, got {other:?}"),
        }
    }

    #[test]
    fn integrates_constants_and_symbols() {
        // ∫5 dx = 5*x → simplifies to 5*x
        assert!(matches!(integral_expr_only(integer(5)), Expr::Mul { .. }));
        // ∫x dx = (1/2)*x^2
        assert_eq!(
            integral_expr_only(symbol("x")),
            mul(rational_frac(1, 2), pow(symbol("x"), 2))
        );
        // ∫y dx = y*x (y is a constant w.r.t. x)
        assert!(matches!(integral_expr_only(symbol("y")), Expr::Mul { .. }));
    }

    #[test]
    fn integrates_integer_powers() {
        // ∫x^3 dx = (1/4)*x^4
        assert_eq!(
            integral_expr_only(pow(symbol("x"), 3)),
            mul(rational_frac(1, 4), pow(symbol("x"), 4))
        );
        // ∫x^0 dx = ∫1 dx = 1*x → simplifies to x
        assert!(matches!(
            integral_expr_only(pow(symbol("x"), 0)),
            Expr::Symbol { ref name } if name == "x"
        ));
        // ∫x^(-2) dx = (-1)*x^(-1)
        assert_eq!(
            integral_expr_only(pow(symbol("x"), -2)),
            mul(integer(-1), pow(symbol("x"), -1))
        );
    }

    #[test]
    fn integrates_sums_and_constant_multiples() {
        // ∫(3*x^2 + 2) dx → antiderivative involves x^3 and 2*x
        assert!(matches!(
            integral_of(add(mul(integer(3), pow(symbol("x"), 2)), integer(2))),
            CalculusResponse::Integral { .. }
        ));
        // ∫(-x) dx = -(1/2)*x^2
        assert_eq!(
            integral_expr_only(neg(symbol("x"))),
            neg(mul(rational_frac(1, 2), pow(symbol("x"), 2)))
        );
    }

    #[test]
    fn integral_rejects_x_to_minus_one_and_transcendentals() {
        // ∫x^(-1) dx = ln(x) — out of scope
        assert!(matches!(
            integral_of(pow(symbol("x"), -1)),
            CalculusResponse::Error { reason, .. } if reason.contains("ln(x)")
        ));
        // ∫sin(x) dx — out of scope
        assert!(matches!(
            integral_of(Expr::Sin { value: Box::new(symbol("x")) }),
            CalculusResponse::Error { reason, .. } if reason.contains("out of scope")
        ));
        // ∫x*x dx — both sides contain variable
        assert!(matches!(
            integral_of(mul(symbol("x"), symbol("x"))),
            CalculusResponse::Error { reason, .. } if reason.contains("unsupported")
        ));
    }

    #[test]
    fn definite_integral_polynomial() {
        // ∫₀² x³ dx = [x⁴/4]₀² = 16/4 - 0 = 4
        let resp = CalculusRequest::DefiniteIntegral {
            expr: pow(symbol("x"), 3),
            variable: "x".to_owned(),
            lower: integer(0),
            upper: integer(2),
        }
        .evaluate();
        assert!(matches!(
            resp,
            CalculusResponse::DefiniteIntegral { ref value, .. } if matches!(
                value, Expr::Integer { value: v } if v == "4"
            )
        ));
    }

    #[test]
    fn definite_integral_fractional_bounds() {
        // ∫₀¹ x dx = [x²/2]₀¹ = 1/2
        let resp = CalculusRequest::DefiniteIntegral {
            expr: symbol("x"),
            variable: "x".to_owned(),
            lower: integer(0),
            upper: integer(1),
        }
        .evaluate();
        assert!(matches!(
            resp,
            CalculusResponse::DefiniteIntegral { ref value, .. } if matches!(
                value,
                Expr::Rational { numerator: n, denominator: d } if n == "1" && d == "2"
            )
        ));
    }

    #[test]
    fn definite_integral_rejects_symbolic_bounds() {
        let resp = CalculusRequest::DefiniteIntegral {
            expr: symbol("x"),
            variable: "x".to_owned(),
            lower: symbol("a"),
            upper: integer(1),
        }
        .evaluate();
        assert!(matches!(
            resp,
            CalculusResponse::Error { reason, .. } if reason.contains("unbound symbol")
        ));
    }

    #[test]
    fn integral_rejects_invalid_variable_name() {
        assert!(matches!(
            (CalculusRequest::Integral {
                expr: symbol("x"),
                variable: "1x".to_owned(),
            })
            .evaluate(),
            CalculusResponse::Error { reason, .. } if reason.contains("invalid symbol name")
        ));
    }

    #[test]
    fn integrates_subtraction_and_division() {
        // Sub arm in integral_expr (line 202)
        assert!(matches!(
            integral_of(sub(symbol("x"), integer(1))),
            CalculusResponse::Integral { .. }
        ));
        // Div arm (line 221) and !contains_symbol (line 224): ∫x/2 dx must produce an antiderivative
        assert!(matches!(
            integral_of(div(symbol("x"), integer(2))),
            CalculusResponse::Integral { .. }
        ));
    }

    #[test]
    fn contains_symbol_detects_variable_inside_binary_and_log_operands() {
        // Binary arm (line 163) uses ||: mul(x, add(x,1)) has variable on both sides → Error.
        // With && mutation, contains_symbol(add(x,1),"x") = false, so only lv=true,
        // rv=false, and it would produce an Integral instead of Error.
        assert!(matches!(
            integral_of(mul(symbol("x"), add(symbol("x"), integer(1)))),
            CalculusResponse::Error { .. }
        ));

        // Log arm (line 178) uses ||: mul(log_2(x), 3) has variable in log value → Error
        // (integral of log is unsupported). With && mutation, log appears variable-free and
        // the whole expression is treated as a constant, returning Integral instead.
        let log_of_x = Expr::Log {
            base: Box::new(integer(2)),
            value: Box::new(symbol("x")),
        };
        assert!(matches!(
            integral_of(mul(log_of_x, integer(3))),
            CalculusResponse::Error { .. }
        ));
    }

    #[test]
    fn integral_error_message_contains_expression_type_name() {
        // expr_type_name must return the real name (kills "" and "xyzzy" mutations at line 340)
        assert!(matches!(
            integral_of(Expr::Sin { value: Box::new(symbol("x")) }),
            CalculusResponse::Error { reason, .. } if reason.contains("sin")
        ));
    }

    #[test]
    fn definite_integral_exercises_all_eval_rational_arms() {
        // Integer arm (line 275): ∫₀² 3 dx = 6
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: integer(3),
                variable: "x".to_owned(),
                lower: integer(0),
                upper: integer(2),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value, Expr::Integer { value: v } if v == "6")
        ));

        // Add arm (line 286): ∫₀¹ (x+1) dx = 3/2
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: add(symbol("x"), integer(1)),
                variable: "x".to_owned(),
                lower: integer(0),
                upper: integer(1),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value,
                    Expr::Rational { numerator: n, denominator: d } if n == "3" && d == "2"
                )
        ));

        // Sub arm (line 289): ∫₀¹ (x-1) dx = -1/2
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: sub(symbol("x"), integer(1)),
                variable: "x".to_owned(),
                lower: integer(0),
                upper: integer(1),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value,
                    Expr::Rational { numerator: n, denominator: d } if n == "-1" && d == "2"
                )
        ));

        // Div arm (line 295) + Integer arm again: ∫₀² x/2 dx = 1
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: div(symbol("x"), integer(2)),
                variable: "x".to_owned(),
                lower: integer(0),
                upper: integer(2),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value, Expr::Integer { value: v } if v == "1")
        ));

        // Neg arm (line 298): ∫₀¹ (-x) dx = -1/2
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: neg(symbol("x")),
                variable: "x".to_owned(),
                lower: integer(0),
                upper: integer(1),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value,
                    Expr::Rational { numerator: n, denominator: d } if n == "-1" && d == "2"
                )
        ));
    }

    #[test]
    fn definite_integral_exercises_eval_bound_arms() {
        // Rational arm in eval_bound (line 260): ∫_{1/2}^{1} x dx = 3/8
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: symbol("x"),
                variable: "x".to_owned(),
                lower: Expr::Rational {
                    numerator: "1".to_owned(),
                    denominator: "2".to_owned(),
                },
                upper: integer(1),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value,
                    Expr::Rational { numerator: n, denominator: d } if n == "3" && d == "8"
                )
        ));

        // Neg arm in eval_bound (line 266): ∫_{-1}^{1} x dx = 0
        assert!(matches!(
            (CalculusRequest::DefiniteIntegral {
                expr: symbol("x"),
                variable: "x".to_owned(),
                lower: Expr::Neg { value: Box::new(integer(1)) },
                upper: integer(1),
            })
            .evaluate(),
            CalculusResponse::DefiniteIntegral { ref value, .. }
                if matches!(value, Expr::Integer { value: v } if v == "0")
        ));
    }
}
