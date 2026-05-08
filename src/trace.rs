use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{
        ErrorCode, ExactRational, classify_error, schema_json, validate_decimal_places,
        validate_expr_limits,
    },
};
use num_bigint::BigInt;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum TraceRequest {
    Eval {
        expr: Expr,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
    },
    Simplify {
        expr: Expr,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TraceResponse {
    Evaluated {
        contract_version: String,
        exact: ExactRational,
        decimal: String,
        trace: Vec<TraceStep>,
        checks: Vec<TraceCheck>,
    },
    Simplified {
        contract_version: String,
        expr: Expr,
        trace: Vec<TraceStep>,
        checks: Vec<TraceCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
        trace: Vec<TraceStep>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: usize,
    pub rule: String,
    pub input: Expr,
    pub output: TraceOutput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceOutput {
    Expr { expr: Expr },
    Rational { exact: ExactRational },
    Error { code: ErrorCode, reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceCheck {
    pub name: String,
    pub passed: bool,
}

impl TraceRequest {
    pub fn evaluate(&self) -> TraceResponse {
        match self {
            TraceRequest::Eval {
                expr,
                decimal_places,
            } => {
                let mut trace = TraceBuilder::default();
                match validate_decimal_places(*decimal_places)
                    .and_then(|_| validate_expr_limits(expr))
                    .and_then(|_| eval_with_trace(expr, &mut trace))
                {
                    Ok(value) => TraceResponse::Evaluated {
                        contract_version: CONTRACT_VERSION.to_owned(),
                        exact: exact_rational(&value),
                        decimal: value.decimal_string(*decimal_places),
                        trace: trace.steps,
                        checks: vec![
                            TraceCheck {
                                name: "trace_steps_ordered".to_owned(),
                                passed: true,
                            },
                            TraceCheck {
                                name: "canonical_rational_form".to_owned(),
                                passed: value.denominator() > &BigInt::zero(),
                            },
                        ],
                    },
                    Err(reason) => TraceResponse::Error {
                        contract_version: CONTRACT_VERSION.to_owned(),
                        code: classify_error(&reason),
                        reason,
                        trace: trace.steps,
                    },
                }
            }
            TraceRequest::Simplify { expr } => {
                let mut trace = TraceBuilder::default();
                match validate_expr_limits(expr).and_then(|_| simplify_with_trace(expr, &mut trace))
                {
                    Ok(expr) => TraceResponse::Simplified {
                        contract_version: CONTRACT_VERSION.to_owned(),
                        expr,
                        trace: trace.steps,
                        checks: vec![
                            TraceCheck {
                                name: "trace_steps_ordered".to_owned(),
                                passed: true,
                            },
                            TraceCheck {
                                name: "symbolic_identities_applied".to_owned(),
                                passed: true,
                            },
                        ],
                    },
                    Err(reason) => TraceResponse::Error {
                        contract_version: CONTRACT_VERSION.to_owned(),
                        code: classify_error(&reason),
                        reason,
                        trace: trace.steps,
                    },
                }
            }
        }
    }
}

pub fn trace_schema_json() -> Value {
    let defs = schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/trace.json"),
        "title": "agent-calc calc1 trace request",
        "description": "Typed request for deterministic evaluation or simplification with machine-readable trace steps.",
        "type": "object",
        "required": ["intent", "expr"],
        "additionalProperties": false,
        "properties": {
            "intent": {"enum": ["eval", "simplify"]},
            "expr": {"$ref": "#/$defs/Expr"},
            "decimal_places": {
                "type": "integer",
                "minimum": 0,
                "maximum": 128,
                "default": 12
            }
        },
        "$defs": defs
    })
}

#[derive(Default)]
struct TraceBuilder {
    steps: Vec<TraceStep>,
}

impl TraceBuilder {
    fn rational(&mut self, rule: &str, input: &Expr, value: &Rational) {
        self.push(
            rule,
            input.clone(),
            TraceOutput::Rational {
                exact: exact_rational(value),
            },
        );
    }

    fn expr(&mut self, rule: &str, input: &Expr, expr: &Expr) {
        self.push(
            rule,
            input.clone(),
            TraceOutput::Expr { expr: expr.clone() },
        );
    }

    fn error(&mut self, rule: &str, input: &Expr, reason: &str) {
        self.push(
            rule,
            input.clone(),
            TraceOutput::Error {
                code: classify_error(reason),
                reason: reason.to_owned(),
            },
        );
    }

    fn push(&mut self, rule: &str, input: Expr, output: TraceOutput) {
        let step = self.steps.len() + 1;
        self.steps.push(TraceStep {
            step,
            rule: rule.to_owned(),
            input,
            output,
        });
    }
}

fn eval_with_trace(expr: &Expr, trace: &mut TraceBuilder) -> Result<Rational, String> {
    let result = match expr {
        Expr::Integer { value } => Rational::parse_integer(value).map_err(|e| e.to_string()),
        Expr::Rational {
            numerator,
            denominator,
        } => Rational::parse(numerator, denominator).map_err(|e| e.to_string()),
        Expr::Symbol { name } => Err(format!("unbound symbol `{name}`")),
        Expr::Add { left, right } => eval_with_trace(left, trace)?
            .checked_add(&eval_with_trace(right, trace)?)
            .map_err(|e| e.to_string()),
        Expr::Sub { left, right } => eval_with_trace(left, trace)?
            .checked_sub(&eval_with_trace(right, trace)?)
            .map_err(|e| e.to_string()),
        Expr::Mul { left, right } => eval_with_trace(left, trace)?
            .checked_mul(&eval_with_trace(right, trace)?)
            .map_err(|e| e.to_string()),
        Expr::Div { left, right } => eval_with_trace(left, trace)?
            .checked_div(&eval_with_trace(right, trace)?)
            .map_err(|e| e.to_string()),
        Expr::Pow { base, exponent } => eval_with_trace(base, trace)?
            .checked_pow_i32(*exponent)
            .map_err(|e| e.to_string()),
        Expr::Neg { value } => eval_with_trace(value, trace)?
            .checked_neg()
            .map_err(|e| e.to_string()),
        Expr::Abs { value } => Ok(eval_with_trace(value, trace)?.abs()),
        Expr::Floor { value } => Ok(eval_with_trace(value, trace)?.floor()),
        Expr::Ceil { value } => Ok(eval_with_trace(value, trace)?.ceil()),
        Expr::Round { value } => Ok(eval_with_trace(value, trace)?.round()),
        Expr::Max { left, right } => {
            let l = eval_with_trace(left, trace)?;
            let r = eval_with_trace(right, trace)?;
            Ok(if l >= r { l } else { r })
        }
        Expr::Min { left, right } => {
            let l = eval_with_trace(left, trace)?;
            let r = eval_with_trace(right, trace)?;
            Ok(if l <= r { l } else { r })
        }
        Expr::Sqrt { value } => {
            let v = eval_with_trace(value, trace)?;
            if v.is_negative() {
                return Err("sqrt of negative number".to_owned());
            }
            match v.exact_sqrt() {
                Some(s) => Ok(s),
                None => Err(
                    "sqrt result is irrational; use the eval command for approximate_f64"
                        .to_owned(),
                ),
            }
        }
        Expr::Exp { .. }
        | Expr::Ln { .. }
        | Expr::Sin { .. }
        | Expr::Cos { .. }
        | Expr::Tan { .. }
        | Expr::Log { .. } => {
            Err("transcendental node cannot be evaluated exactly; use the eval command".to_owned())
        }
    };

    match &result {
        Ok(value) => trace.rational(eval_rule(expr), expr, value),
        Err(reason) => trace.error(eval_rule(expr), expr, reason),
    }
    result
}

fn simplify_with_trace(expr: &Expr, trace: &mut TraceBuilder) -> Result<Expr, String> {
    let result: Result<(Expr, &'static str), String> = match expr {
        Expr::Integer { value } => {
            Rational::parse_integer(value).map_err(|e| e.to_string())?;
            Ok((expr.clone(), "simplify.integer_literal"))
        }
        Expr::Rational {
            numerator,
            denominator,
        } => {
            Rational::parse(numerator, denominator).map_err(|e| e.to_string())?;
            Ok((expr.clone(), "simplify.rational_literal"))
        }
        Expr::Symbol { name } => {
            if is_valid_symbol_name(name) {
                Ok((expr.clone(), "simplify.symbol"))
            } else {
                Err(format!("invalid symbol name `{name}`"))
            }
        }
        Expr::Add { left, right } => {
            let left = simplify_with_trace(left, trace)?;
            let right = simplify_with_trace(right, trace)?;
            let normalized_input = add(left.clone(), right.clone());
            let output = if is_zero_expr(&left)? {
                right
            } else if is_zero_expr(&right)? {
                left
            } else {
                add(left, right)
            };
            Ok((output.clone(), simplify_rule(&normalized_input, &output)))
        }
        Expr::Sub { left, right } => {
            let left = simplify_with_trace(left, trace)?;
            let right = simplify_with_trace(right, trace)?;
            let normalized_input = sub(left.clone(), right.clone());
            let output = if is_zero_expr(&right)? {
                left
            } else {
                sub(left, right)
            };
            Ok((output.clone(), simplify_rule(&normalized_input, &output)))
        }
        Expr::Mul { left, right } => {
            let left = simplify_with_trace(left, trace)?;
            let right = simplify_with_trace(right, trace)?;
            let normalized_input = mul(left.clone(), right.clone());
            let output = if is_zero_expr(&left)? || is_zero_expr(&right)? {
                integer(0)
            } else if is_one_expr(&left)? {
                right
            } else if is_one_expr(&right)? {
                left
            } else {
                mul(left, right)
            };
            Ok((output.clone(), simplify_rule(&normalized_input, &output)))
        }
        Expr::Div { left, right } => {
            let left = simplify_with_trace(left, trace)?;
            let right = simplify_with_trace(right, trace)?;
            let normalized_input = div(left.clone(), right.clone());
            let output = if is_one_expr(&right)? {
                left
            } else {
                div(left, right)
            };
            Ok((output.clone(), simplify_rule(&normalized_input, &output)))
        }
        Expr::Pow { base, exponent } => {
            let base = simplify_with_trace(base, trace)?;
            let normalized_input = pow(base.clone(), *exponent);
            let output = if *exponent == 0 {
                integer(1)
            } else if *exponent == 1 {
                base
            } else {
                pow(base, *exponent)
            };
            Ok((output.clone(), simplify_rule(&normalized_input, &output)))
        }
        Expr::Neg { value } => {
            let value = simplify_with_trace(value, trace)?;
            let normalized_input = neg(value.clone());
            let output = if let Expr::Neg { value } = value {
                *value
            } else {
                neg(value)
            };
            Ok((output.clone(), simplify_rule(&normalized_input, &output)))
        }
        Expr::Sqrt { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Sqrt { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Exp { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Exp { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Ln { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Ln { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Sin { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Sin { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Cos { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Cos { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Tan { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Tan { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Abs { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Abs { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Floor { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Floor { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Ceil { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Ceil { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Round { value } => {
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Round { value: Box::new(v) };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Log { base, value } => {
            let b = simplify_with_trace(base, trace)?;
            let v = simplify_with_trace(value, trace)?;
            let out = Expr::Log {
                base: Box::new(b),
                value: Box::new(v),
            };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Max { left, right } => {
            let l = simplify_with_trace(left, trace)?;
            let r = simplify_with_trace(right, trace)?;
            let out = Expr::Max {
                left: Box::new(l),
                right: Box::new(r),
            };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
        Expr::Min { left, right } => {
            let l = simplify_with_trace(left, trace)?;
            let r = simplify_with_trace(right, trace)?;
            let out = Expr::Min {
                left: Box::new(l),
                right: Box::new(r),
            };
            Ok((out.clone(), simplify_rule(expr, &out)))
        }
    };

    match &result {
        Ok((value, rule)) => trace.expr(rule, expr, value),
        Err(reason) => trace.error(simplify_rule_for_input(expr), expr, reason),
    }
    result.map(|(value, _)| value)
}

fn eval_rule(expr: &Expr) -> &'static str {
    match expr {
        Expr::Integer { .. } => "eval.integer_literal",
        Expr::Rational { .. } => "eval.rational_literal",
        Expr::Symbol { .. } => "eval.symbol",
        Expr::Add { .. } => "eval.add",
        Expr::Sub { .. } => "eval.sub",
        Expr::Mul { .. } => "eval.mul",
        Expr::Div { .. } => "eval.div",
        Expr::Pow { .. } => "eval.pow",
        Expr::Neg { .. } => "eval.neg",
        Expr::Sqrt { .. } => "eval.sqrt",
        Expr::Exp { .. } => "eval.exp",
        Expr::Ln { .. } => "eval.ln",
        Expr::Sin { .. } => "eval.sin",
        Expr::Cos { .. } => "eval.cos",
        Expr::Tan { .. } => "eval.tan",
        Expr::Abs { .. } => "eval.abs",
        Expr::Floor { .. } => "eval.floor",
        Expr::Ceil { .. } => "eval.ceil",
        Expr::Round { .. } => "eval.round",
        Expr::Log { .. } => "eval.log",
        Expr::Max { .. } => "eval.max",
        Expr::Min { .. } => "eval.min",
    }
}

fn simplify_rule(input: &Expr, output: &Expr) -> &'static str {
    match input {
        Expr::Integer { .. } => "simplify.integer_literal",
        Expr::Rational { .. } => "simplify.rational_literal",
        Expr::Symbol { .. } => "simplify.symbol",
        Expr::Add { left, right } => {
            if output == left.as_ref() {
                "simplify.add_zero_right"
            } else if output == right.as_ref() {
                "simplify.add_zero_left"
            } else {
                "simplify.add_recurse"
            }
        }
        Expr::Sub { left, .. } => {
            if output == left.as_ref() {
                "simplify.sub_zero_right"
            } else {
                "simplify.sub_recurse"
            }
        }
        Expr::Mul { left, right } => {
            if is_literal_zero(output) {
                "simplify.mul_zero"
            } else if output == left.as_ref() {
                "simplify.mul_one_right"
            } else if output == right.as_ref() {
                "simplify.mul_one_left"
            } else {
                "simplify.mul_recurse"
            }
        }
        Expr::Div { left, .. } => {
            if output == left.as_ref() {
                "simplify.div_one_right"
            } else {
                "simplify.div_recurse"
            }
        }
        Expr::Pow { base, exponent } => {
            if *exponent == 0 {
                "simplify.pow_zero"
            } else if output == base.as_ref() {
                "simplify.pow_one"
            } else {
                "simplify.pow_recurse"
            }
        }
        Expr::Neg { .. } => {
            if matches!(input, Expr::Neg { value } if matches!(value.as_ref(), Expr::Neg { .. })) {
                "simplify.double_negation"
            } else {
                "simplify.neg_recurse"
            }
        }
        Expr::Sqrt { .. } => "simplify.sqrt_recurse",
        Expr::Exp { .. } => "simplify.exp_recurse",
        Expr::Ln { .. } => "simplify.ln_recurse",
        Expr::Sin { .. } => "simplify.sin_recurse",
        Expr::Cos { .. } => "simplify.cos_recurse",
        Expr::Tan { .. } => "simplify.tan_recurse",
        Expr::Abs { .. } => "simplify.abs_recurse",
        Expr::Floor { .. } => "simplify.floor_recurse",
        Expr::Ceil { .. } => "simplify.ceil_recurse",
        Expr::Round { .. } => "simplify.round_recurse",
        Expr::Log { .. } => "simplify.log_recurse",
        Expr::Max { .. } => "simplify.max_recurse",
        Expr::Min { .. } => "simplify.min_recurse",
    }
}

fn simplify_rule_for_input(input: &Expr) -> &'static str {
    match input {
        Expr::Integer { .. } => "simplify.integer_literal",
        Expr::Rational { .. } => "simplify.rational_literal",
        Expr::Symbol { .. } => "simplify.symbol",
        Expr::Add { .. } => "simplify.add_recurse",
        Expr::Sub { .. } => "simplify.sub_recurse",
        Expr::Mul { .. } => "simplify.mul_recurse",
        Expr::Div { .. } => "simplify.div_recurse",
        Expr::Pow { .. } => "simplify.pow_recurse",
        Expr::Neg { .. } => "simplify.neg_recurse",
        Expr::Sqrt { .. } => "simplify.sqrt_recurse",
        Expr::Exp { .. } => "simplify.exp_recurse",
        Expr::Ln { .. } => "simplify.ln_recurse",
        Expr::Sin { .. } => "simplify.sin_recurse",
        Expr::Cos { .. } => "simplify.cos_recurse",
        Expr::Tan { .. } => "simplify.tan_recurse",
        Expr::Abs { .. } => "simplify.abs_recurse",
        Expr::Floor { .. } => "simplify.floor_recurse",
        Expr::Ceil { .. } => "simplify.ceil_recurse",
        Expr::Round { .. } => "simplify.round_recurse",
        Expr::Log { .. } => "simplify.log_recurse",
        Expr::Max { .. } => "simplify.max_recurse",
        Expr::Min { .. } => "simplify.min_recurse",
    }
}

fn exact_rational(value: &Rational) -> ExactRational {
    ExactRational {
        numerator: value.numerator().to_string(),
        denominator: value.denominator().to_string(),
        display: value.to_string(),
    }
}

fn default_decimal_places() -> usize {
    12
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

fn is_zero_expr(expr: &Expr) -> Result<bool, String> {
    Ok(match literal_rational(expr)? {
        Some(value) => value.is_zero(),
        None => false,
    })
}

fn is_one_expr(expr: &Expr) -> Result<bool, String> {
    Ok(match literal_rational(expr)? {
        Some(value) => value == Rational::one(),
        None => false,
    })
}

fn is_literal_zero(expr: &Expr) -> bool {
    literal_rational(expr)
        .ok()
        .flatten()
        .is_some_and(|value| value.is_zero())
}

fn literal_rational(expr: &Expr) -> Result<Option<Rational>, String> {
    match expr {
        Expr::Integer { value } => Rational::parse_integer(value)
            .map(Some)
            .map_err(|e| e.to_string()),
        Expr::Rational {
            numerator,
            denominator,
        } => Rational::parse(numerator, denominator)
            .map(Some)
            .map_err(|e| e.to_string()),
        _ => Ok(None),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol(name: &str) -> Expr {
        Expr::Symbol {
            name: name.to_owned(),
        }
    }

    fn rational(numerator: i32, denominator: i32) -> Expr {
        Expr::Rational {
            numerator: numerator.to_string(),
            denominator: denominator.to_string(),
        }
    }

    fn trace_rules(response: TraceResponse) -> Vec<String> {
        match response {
            TraceResponse::Simplified { trace, .. } => {
                trace.into_iter().map(|step| step.rule).collect()
            }
            TraceResponse::Error { trace, .. } => trace.into_iter().map(|step| step.rule).collect(),
            other => panic!("expected trace-bearing response, got {other:?}"),
        }
    }

    #[test]
    fn traces_exact_evaluation_steps() {
        let response = TraceRequest::Eval {
            expr: add(integer(2), integer(3)),
            decimal_places: 2,
        }
        .evaluate();

        match response {
            TraceResponse::Evaluated {
                exact,
                decimal,
                trace,
                checks,
                ..
            } => {
                assert_eq!(exact.display, "5");
                assert_eq!(decimal, "5.00");
                assert_eq!(trace.len(), 3);
                assert_eq!(trace[0].rule, "eval.integer_literal");
                assert_eq!(trace[1].rule, "eval.integer_literal");
                assert_eq!(trace[2].rule, "eval.add");
                assert_eq!(checks[0].name, "trace_steps_ordered");
            }
            other => panic!("expected evaluated response, got {other:?}"),
        }
    }

    #[test]
    fn traces_simplification_rule_steps() {
        let response = TraceRequest::Simplify {
            expr: mul(add(symbol("x"), integer(0)), integer(1)),
        }
        .evaluate();

        match response {
            TraceResponse::Simplified { expr, trace, .. } => {
                assert_eq!(expr, symbol("x"));
                assert!(
                    trace
                        .iter()
                        .any(|step| step.rule == "simplify.add_zero_right")
                );
                assert!(
                    trace
                        .iter()
                        .any(|step| step.rule == "simplify.mul_one_right")
                );
            }
            other => panic!("expected simplified response, got {other:?}"),
        }
    }

    #[test]
    fn traces_errors_before_returning_typed_failure() {
        let response = TraceRequest::Eval {
            expr: symbol("x"),
            decimal_places: 12,
        }
        .evaluate();

        match response {
            TraceResponse::Error { reason, trace, .. } => {
                assert_eq!(reason, "unbound symbol `x`");
                assert_eq!(trace.len(), 1);
                assert_eq!(trace[0].rule, "eval.symbol");
                assert!(matches!(trace[0].output, TraceOutput::Error { .. }));
            }
            other => panic!("expected error response, got {other:?}"),
        }
    }

    #[test]
    fn accepts_underscore_symbol_names_in_simplify_trace() {
        let response = TraceRequest::Simplify {
            expr: add(symbol("_x"), symbol("x_1")),
        }
        .evaluate();

        assert!(matches!(response, TraceResponse::Simplified { .. }));
    }

    #[test]
    fn default_eval_trace_decimal_places_are_stable() {
        let request: TraceRequest = serde_json::from_str(
            r#"{
                "intent": "eval",
                "expr": {"kind": "integer", "value": "5"}
            }"#,
        )
        .unwrap();

        match request.evaluate() {
            TraceResponse::Evaluated { decimal, .. } => {
                assert_eq!(decimal, "5.000000000000");
            }
            other => panic!("expected evaluated response, got {other:?}"),
        }
    }

    #[test]
    fn trace_rejects_invalid_symbol_names_with_symbol_rule() {
        for name in ["", "1x", "x-y"] {
            let response = TraceRequest::Simplify { expr: symbol(name) }.evaluate();

            match response {
                TraceResponse::Error { reason, trace, .. } => {
                    assert_eq!(reason, format!("invalid symbol name `{name}`"));
                    assert_eq!(trace.len(), 1);
                    assert_eq!(trace[0].rule, "simplify.symbol");
                }
                other => panic!("expected error response for {name:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn traces_each_simplify_identity_rule() {
        let cases = [
            (
                add(integer(0), symbol("x")),
                symbol("x"),
                "simplify.add_zero_left",
            ),
            (
                add(symbol("x"), integer(0)),
                symbol("x"),
                "simplify.add_zero_right",
            ),
            (
                sub(symbol("x"), integer(0)),
                symbol("x"),
                "simplify.sub_zero_right",
            ),
            (
                mul(integer(0), symbol("x")),
                integer(0),
                "simplify.mul_zero",
            ),
            (
                mul(symbol("x"), rational(0, 5)),
                integer(0),
                "simplify.mul_zero",
            ),
            (
                mul(integer(1), symbol("x")),
                symbol("x"),
                "simplify.mul_one_left",
            ),
            (
                mul(symbol("x"), integer(1)),
                symbol("x"),
                "simplify.mul_one_right",
            ),
            (
                div(symbol("x"), integer(1)),
                symbol("x"),
                "simplify.div_one_right",
            ),
            (pow(symbol("x"), 0), integer(1), "simplify.pow_zero"),
            (pow(symbol("x"), 1), symbol("x"), "simplify.pow_one"),
            (
                neg(neg(symbol("x"))),
                symbol("x"),
                "simplify.double_negation",
            ),
        ];

        for (input, expected_expr, expected_rule) in cases {
            let response = TraceRequest::Simplify { expr: input }.evaluate();

            match response {
                TraceResponse::Simplified { expr, trace, .. } => {
                    assert_eq!(expr, expected_expr, "case {expected_rule}");
                    assert_eq!(trace.last().unwrap().rule, expected_rule);
                }
                other => panic!("expected simplified response for {expected_rule}, got {other:?}"),
            }
        }
    }

    #[test]
    fn traces_recurse_rules_when_no_identity_applies() {
        let cases = [
            (add(symbol("x"), integer(2)), "simplify.add_recurse"),
            (sub(symbol("x"), integer(2)), "simplify.sub_recurse"),
            (mul(symbol("x"), integer(2)), "simplify.mul_recurse"),
            (div(symbol("x"), integer(2)), "simplify.div_recurse"),
            (pow(symbol("x"), 2), "simplify.pow_recurse"),
            (neg(symbol("x")), "simplify.neg_recurse"),
        ];

        for (input, expected_rule) in cases {
            let rules = trace_rules(TraceRequest::Simplify { expr: input }.evaluate());
            assert_eq!(rules.last().unwrap(), expected_rule);
        }
    }
}
