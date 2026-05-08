use crate::{CONTRACT_VERSION, Rational};
use num_bigint::BigInt;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const MAX_DECIMAL_PLACES: usize = 128;
pub const MAX_EXPR_DEPTH: usize = 64;
pub const MAX_EXPR_NODES: usize = 4096;
pub const MAX_INTEGER_DIGITS: usize = 1024;
pub const MAX_SYMBOL_LEN: usize = 128;
pub const MAX_ABS_EXPONENT: i32 = 1024;
pub const MAX_BINDINGS: usize = 1024;

#[derive(Clone, Debug, Serialize)]
pub struct Describe {
    pub name: &'static str,
    pub version: &'static str,
    pub contract_version: &'static str,
    pub purpose: &'static str,
    pub capabilities: Vec<&'static str>,
    pub inputs: Vec<&'static str>,
    pub outputs: Vec<&'static str>,
    pub invariants: Vec<&'static str>,
}

impl Describe {
    pub fn current() -> Self {
        Self {
            name: "agent-calc",
            version: env!("CARGO_PKG_VERSION"),
            contract_version: CONTRACT_VERSION,
            purpose: "AI-native exact computation kernel with typed failures",
            capabilities: vec![
                "describe-contract",
                "emit-json-schema",
                "evaluate-exact-rational-expressions",
                "evaluate-transcendental-expression-nodes",
                "simplify-symbolic-expressions",
                "emit-machine-readable-computation-traces",
                "substitute-symbolic-bindings",
                "validate-symbolic-assumption-contexts",
                "solve-exact-affine-symbolic-equations",
                "differentiate-symbolic-expressions",
                "solve-exact-affine-inequalities",
                "operate-on-exact-rational-polynomials",
                "analyze-exact-rational-intervals",
                "evaluate-exact-rational-time-value-finance",
                "convert-and-compose-dimension-checked-quantities",
                "evaluate-f64-matrix-operations",
                "evaluate-statistical-distributions-and-samples",
                "solve-bounded-one-dimensional-optimization",
                "solve-continuous-linear-programs",
                "evaluate-complex-number-operations",
                "return-explicit-failure-status",
                "return-stable-error-codes",
            ],
            inputs: vec!["stdin-json", "file-json"],
            outputs: vec!["json"],
            invariants: vec![
                "deterministic: same input -> same output",
                "exact-first: rational outputs preserve numerator and denominator",
                "symbolic-safe: simplification validates leaves and applies bounded identities only",
                "trace-safe: trace steps are deterministic and ordered",
                "binding-safe: substitution requires every symbol to have an exact evaluable binding",
                "assumption-safe: domain assumptions derive exact rational bounds or typed contradictions",
                "solve-safe: symbolic solve isolates exact affine equations or returns typed unsupported failures",
                "calculus-safe: symbolic derivatives apply exact AST rewrite rules",
                "inequality-safe: affine inequality solutions preserve exact rational boundary semantics",
                "polynomial-safe: polynomial coefficients are exact rational expressions in ascending power order",
                "interval-safe: interval arithmetic preserves ordered exact rational bounds",
                "finance-safe: time-value calculations use exact rational cash-flow math",
                "dimension-safe: unit operations are checked before arithmetic",
                "shape-safe: matrix operations validate dimensions before calling nalgebra",
                "parameter-safe: distribution parameters are validated before calling statrs",
                "bounded-optimization: optimizer inputs are validated before calling argmin",
                "solver-backed: linear programs are modeled through good_lp and solved by microlp",
                "complex-safe: complex inputs are validated before calling num-complex",
                "typed-failures: unsupported or invalid computation returns an error status",
                "bounded-inputs: request handlers enforce expression size and decimal precision limits",
                "no-prose-contract: machine consumers use JSON schema and command output",
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvalRequest {
    pub expr: Expr,
    #[serde(default = "default_decimal_places")]
    pub decimal_places: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expr {
    Integer {
        value: String,
    },
    Rational {
        numerator: String,
        denominator: String,
    },
    Symbol {
        name: String,
    },
    Add {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Sub {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Mul {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Div {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Pow {
        base: Box<Expr>,
        exponent: i32,
    },
    Neg {
        value: Box<Expr>,
    },
    Sqrt {
        value: Box<Expr>,
    },
    Exp {
        value: Box<Expr>,
    },
    Ln {
        value: Box<Expr>,
    },
    Sin {
        value: Box<Expr>,
    },
    Cos {
        value: Box<Expr>,
    },
    Tan {
        value: Box<Expr>,
    },
    Abs {
        value: Box<Expr>,
    },
    Floor {
        value: Box<Expr>,
    },
    Ceil {
        value: Box<Expr>,
    },
    Round {
        value: Box<Expr>,
    },
    Log {
        base: Box<Expr>,
        value: Box<Expr>,
    },
    Max {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Min {
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exactness {
    ApproximateF64,
}

enum EvalOutput {
    Exact(Rational),
    Approx(f64),
}

impl EvalOutput {
    fn to_f64(&self) -> f64 {
        match self {
            EvalOutput::Exact(r) => r.to_f64(),
            EvalOutput::Approx(f) => *f,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimplifyRequest {
    pub expr: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubstituteRequest {
    pub expr: Expr,
    pub bindings: BTreeMap<String, Expr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SimplifyResponse {
    Simplified {
        contract_version: String,
        expr: Expr,
        checks: Vec<Check>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SubstituteResponse {
    Substituted {
        contract_version: String,
        expr: Expr,
        checks: Vec<Check>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EvalResponse {
    Solved {
        contract_version: String,
        exact: ExactRational,
        decimal: String,
        checks: Vec<Check>,
    },
    Approximate {
        contract_version: String,
        value: f64,
        exactness: Exactness,
        checks: Vec<Check>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    DivisionByZero,
    InvalidInput,
    InvalidInteger,
    InvalidSymbol,
    ResourceLimit,
    UnboundSymbol,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExactRational {
    pub numerator: String,
    pub denominator: String,
    pub display: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub passed: bool,
}

impl EvalRequest {
    pub fn evaluate(&self) -> EvalResponse {
        match validate_decimal_places(self.decimal_places)
            .and_then(|_| validate_expr_limits(&self.expr))
        {
            Ok(()) => {}
            Err(reason) => {
                return EvalResponse::Error {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    code: classify_error(&reason),
                    reason,
                };
            }
        }

        match self.expr.evaluate_output() {
            Ok(EvalOutput::Exact(value)) => EvalResponse::Solved {
                contract_version: CONTRACT_VERSION.to_owned(),
                exact: ExactRational {
                    numerator: value.numerator().to_string(),
                    denominator: value.denominator().to_string(),
                    display: value.to_string(),
                },
                decimal: value.decimal_string(self.decimal_places),
                checks: vec![Check {
                    name: "canonical_rational_form".to_owned(),
                    passed: value.denominator() > &BigInt::zero(),
                }],
            },
            Ok(EvalOutput::Approx(value)) => EvalResponse::Approximate {
                contract_version: CONTRACT_VERSION.to_owned(),
                value,
                exactness: Exactness::ApproximateF64,
                checks: vec![Check {
                    name: "evaluated_as_f64".to_owned(),
                    passed: value.is_finite(),
                }],
            },
            Err(reason) => EvalResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }
}

impl Expr {
    pub fn evaluate(&self) -> Result<Rational, String> {
        match self.evaluate_output()? {
            EvalOutput::Exact(r) => Ok(r),
            EvalOutput::Approx(_) => Err(
                "expression contains transcendental operations and cannot be evaluated to an exact rational".to_owned(),
            ),
        }
    }

    pub fn evaluate_float(&self) -> Result<f64, String> {
        Ok(self.evaluate_output()?.to_f64())
    }

    fn evaluate_output(&self) -> Result<EvalOutput, String> {
        match self {
            Expr::Integer { value } => Ok(EvalOutput::Exact(
                Rational::parse_integer(value).map_err(|e| e.to_string())?,
            )),
            Expr::Rational {
                numerator,
                denominator,
            } => Ok(EvalOutput::Exact(
                Rational::parse(numerator, denominator).map_err(|e| e.to_string())?,
            )),
            Expr::Symbol { name } => Err(format!("unbound symbol `{name}`")),
            Expr::Add { left, right } => {
                let l = left.evaluate_output()?;
                let r = right.evaluate_output()?;
                match (l, r) {
                    (EvalOutput::Exact(lv), EvalOutput::Exact(rv)) => Ok(EvalOutput::Exact(
                        lv.checked_add(&rv).map_err(|e| e.to_string())?,
                    )),
                    (l, r) => Ok(EvalOutput::Approx(l.to_f64() + r.to_f64())),
                }
            }
            Expr::Sub { left, right } => {
                let l = left.evaluate_output()?;
                let r = right.evaluate_output()?;
                match (l, r) {
                    (EvalOutput::Exact(lv), EvalOutput::Exact(rv)) => Ok(EvalOutput::Exact(
                        lv.checked_sub(&rv).map_err(|e| e.to_string())?,
                    )),
                    (l, r) => Ok(EvalOutput::Approx(l.to_f64() - r.to_f64())),
                }
            }
            Expr::Mul { left, right } => {
                let l = left.evaluate_output()?;
                let r = right.evaluate_output()?;
                match (l, r) {
                    (EvalOutput::Exact(lv), EvalOutput::Exact(rv)) => Ok(EvalOutput::Exact(
                        lv.checked_mul(&rv).map_err(|e| e.to_string())?,
                    )),
                    (l, r) => Ok(EvalOutput::Approx(l.to_f64() * r.to_f64())),
                }
            }
            Expr::Div { left, right } => {
                let l = left.evaluate_output()?;
                let r = right.evaluate_output()?;
                match (l, r) {
                    (EvalOutput::Exact(lv), EvalOutput::Exact(rv)) => Ok(EvalOutput::Exact(
                        lv.checked_div(&rv).map_err(|e| e.to_string())?,
                    )),
                    (l, r) => {
                        let rv = r.to_f64();
                        if rv == 0.0 {
                            return Err("division by zero".to_owned());
                        }
                        Ok(EvalOutput::Approx(l.to_f64() / rv))
                    }
                }
            }
            Expr::Pow { base, exponent } => match base.evaluate_output()? {
                EvalOutput::Exact(rv) => Ok(EvalOutput::Exact(
                    rv.checked_pow_i32(*exponent).map_err(|e| e.to_string())?,
                )),
                EvalOutput::Approx(f) => Ok(EvalOutput::Approx(f.powi(*exponent))),
            },
            Expr::Neg { value } => match value.evaluate_output()? {
                EvalOutput::Exact(r) => Ok(EvalOutput::Exact(
                    r.checked_neg().map_err(|e| e.to_string())?,
                )),
                EvalOutput::Approx(f) => Ok(EvalOutput::Approx(-f)),
            },
            Expr::Abs { value } => match value.evaluate_output()? {
                EvalOutput::Exact(r) => Ok(EvalOutput::Exact(r.abs())),
                EvalOutput::Approx(f) => Ok(EvalOutput::Approx(f.abs())),
            },
            Expr::Floor { value } => match value.evaluate_output()? {
                EvalOutput::Exact(r) => Ok(EvalOutput::Exact(r.floor())),
                EvalOutput::Approx(f) => Ok(EvalOutput::Approx(f.floor())),
            },
            Expr::Ceil { value } => match value.evaluate_output()? {
                EvalOutput::Exact(r) => Ok(EvalOutput::Exact(r.ceil())),
                EvalOutput::Approx(f) => Ok(EvalOutput::Approx(f.ceil())),
            },
            Expr::Round { value } => match value.evaluate_output()? {
                EvalOutput::Exact(r) => Ok(EvalOutput::Exact(r.round())),
                EvalOutput::Approx(f) => Ok(EvalOutput::Approx(f.round())),
            },
            Expr::Max { left, right } => {
                let l = left.evaluate_output()?;
                let r = right.evaluate_output()?;
                match (l, r) {
                    (EvalOutput::Exact(lv), EvalOutput::Exact(rv)) => {
                        Ok(EvalOutput::Exact(if lv >= rv { lv } else { rv }))
                    }
                    (l, r) => Ok(EvalOutput::Approx(l.to_f64().max(r.to_f64()))),
                }
            }
            Expr::Min { left, right } => {
                let l = left.evaluate_output()?;
                let r = right.evaluate_output()?;
                match (l, r) {
                    (EvalOutput::Exact(lv), EvalOutput::Exact(rv)) => {
                        Ok(EvalOutput::Exact(if lv <= rv { lv } else { rv }))
                    }
                    (l, r) => Ok(EvalOutput::Approx(l.to_f64().min(r.to_f64()))),
                }
            }
            Expr::Sqrt { value } => match value.evaluate_output()? {
                EvalOutput::Exact(r) => {
                    if r.is_negative() {
                        return Err("sqrt of negative number".to_owned());
                    }
                    match r.exact_sqrt() {
                        Some(s) => Ok(EvalOutput::Exact(s)),
                        None => Ok(EvalOutput::Approx(r.to_f64().sqrt())),
                    }
                }
                EvalOutput::Approx(f) => {
                    if f < 0.0 {
                        return Err("sqrt of negative number".to_owned());
                    }
                    Ok(EvalOutput::Approx(f.sqrt()))
                }
            },
            Expr::Exp { value } => Ok(EvalOutput::Approx(value.evaluate_output()?.to_f64().exp())),
            Expr::Ln { value } => {
                let f = value.evaluate_output()?.to_f64();
                if f <= 0.0 {
                    return Err("ln argument must be positive".to_owned());
                }
                Ok(EvalOutput::Approx(f.ln()))
            }
            Expr::Sin { value } => Ok(EvalOutput::Approx(value.evaluate_output()?.to_f64().sin())),
            Expr::Cos { value } => Ok(EvalOutput::Approx(value.evaluate_output()?.to_f64().cos())),
            Expr::Tan { value } => Ok(EvalOutput::Approx(value.evaluate_output()?.to_f64().tan())),
            Expr::Log { base, value } => {
                let b = base.evaluate_output()?.to_f64();
                let v = value.evaluate_output()?.to_f64();
                if b <= 0.0 || b == 1.0 {
                    return Err("log base must be positive and not equal to 1".to_owned());
                }
                if v <= 0.0 {
                    return Err("log argument must be positive".to_owned());
                }
                Ok(EvalOutput::Approx(v.ln() / b.ln()))
            }
        }
    }

    pub fn simplify(&self) -> Result<Expr, String> {
        match self {
            Expr::Integer { value } => {
                Rational::parse_integer(value).map_err(|e| e.to_string())?;
                Ok(self.clone())
            }
            Expr::Rational {
                numerator,
                denominator,
            } => {
                Rational::parse(numerator, denominator).map_err(|e| e.to_string())?;
                Ok(self.clone())
            }
            Expr::Symbol { name } => {
                if is_valid_symbol_name(name) {
                    Ok(self.clone())
                } else {
                    Err(format!("invalid symbol name `{name}`"))
                }
            }
            Expr::Add { left, right } => {
                let left = left.simplify()?;
                let right = right.simplify()?;
                if is_zero_expr(&left)? {
                    Ok(right)
                } else if is_zero_expr(&right)? {
                    Ok(left)
                } else {
                    Ok(Expr::Add {
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                }
            }
            Expr::Sub { left, right } => {
                let left = left.simplify()?;
                let right = right.simplify()?;
                if is_zero_expr(&right)? {
                    Ok(left)
                } else {
                    Ok(Expr::Sub {
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                }
            }
            Expr::Mul { left, right } => {
                let left = left.simplify()?;
                let right = right.simplify()?;
                if is_zero_expr(&left)? || is_zero_expr(&right)? {
                    Ok(integer_expr(0))
                } else if is_one_expr(&left)? {
                    Ok(right)
                } else if is_one_expr(&right)? {
                    Ok(left)
                } else {
                    Ok(Expr::Mul {
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                }
            }
            Expr::Div { left, right } => {
                let left = left.simplify()?;
                let right = right.simplify()?;
                if is_one_expr(&right)? {
                    Ok(left)
                } else {
                    Ok(Expr::Div {
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                }
            }
            Expr::Pow { base, exponent } => {
                let base = base.simplify()?;
                if *exponent == 0 {
                    Ok(integer_expr(1))
                } else if *exponent == 1 {
                    Ok(base)
                } else {
                    Ok(Expr::Pow {
                        base: Box::new(base),
                        exponent: *exponent,
                    })
                }
            }
            Expr::Neg { value } => {
                let value = value.simplify()?;
                if let Expr::Neg { value } = value {
                    Ok(*value)
                } else {
                    Ok(Expr::Neg {
                        value: Box::new(value),
                    })
                }
            }
            Expr::Sqrt { value } => Ok(Expr::Sqrt {
                value: Box::new(value.simplify()?),
            }),
            Expr::Exp { value } => Ok(Expr::Exp {
                value: Box::new(value.simplify()?),
            }),
            Expr::Ln { value } => Ok(Expr::Ln {
                value: Box::new(value.simplify()?),
            }),
            Expr::Sin { value } => Ok(Expr::Sin {
                value: Box::new(value.simplify()?),
            }),
            Expr::Cos { value } => Ok(Expr::Cos {
                value: Box::new(value.simplify()?),
            }),
            Expr::Tan { value } => Ok(Expr::Tan {
                value: Box::new(value.simplify()?),
            }),
            Expr::Abs { value } => Ok(Expr::Abs {
                value: Box::new(value.simplify()?),
            }),
            Expr::Floor { value } => Ok(Expr::Floor {
                value: Box::new(value.simplify()?),
            }),
            Expr::Ceil { value } => Ok(Expr::Ceil {
                value: Box::new(value.simplify()?),
            }),
            Expr::Round { value } => Ok(Expr::Round {
                value: Box::new(value.simplify()?),
            }),
            Expr::Log { base, value } => Ok(Expr::Log {
                base: Box::new(base.simplify()?),
                value: Box::new(value.simplify()?),
            }),
            Expr::Max { left, right } => Ok(Expr::Max {
                left: Box::new(left.simplify()?),
                right: Box::new(right.simplify()?),
            }),
            Expr::Min { left, right } => Ok(Expr::Min {
                left: Box::new(left.simplify()?),
                right: Box::new(right.simplify()?),
            }),
        }
    }

    pub fn substitute(&self, bindings: &BTreeMap<String, Expr>) -> Result<Expr, String> {
        match self {
            Expr::Integer { value } => {
                Rational::parse_integer(value).map_err(|e| e.to_string())?;
                Ok(self.clone())
            }
            Expr::Rational {
                numerator,
                denominator,
            } => {
                Rational::parse(numerator, denominator).map_err(|e| e.to_string())?;
                Ok(self.clone())
            }
            Expr::Symbol { name } => bindings
                .get(name)
                .cloned()
                .ok_or_else(|| format!("missing binding for symbol `{name}`")),
            Expr::Add { left, right } => Ok(Expr::Add {
                left: Box::new(left.substitute(bindings)?),
                right: Box::new(right.substitute(bindings)?),
            }),
            Expr::Sub { left, right } => Ok(Expr::Sub {
                left: Box::new(left.substitute(bindings)?),
                right: Box::new(right.substitute(bindings)?),
            }),
            Expr::Mul { left, right } => Ok(Expr::Mul {
                left: Box::new(left.substitute(bindings)?),
                right: Box::new(right.substitute(bindings)?),
            }),
            Expr::Div { left, right } => Ok(Expr::Div {
                left: Box::new(left.substitute(bindings)?),
                right: Box::new(right.substitute(bindings)?),
            }),
            Expr::Pow { base, exponent } => Ok(Expr::Pow {
                base: Box::new(base.substitute(bindings)?),
                exponent: *exponent,
            }),
            Expr::Neg { value } => Ok(Expr::Neg {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Sqrt { value } => Ok(Expr::Sqrt {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Exp { value } => Ok(Expr::Exp {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Ln { value } => Ok(Expr::Ln {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Sin { value } => Ok(Expr::Sin {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Cos { value } => Ok(Expr::Cos {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Tan { value } => Ok(Expr::Tan {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Abs { value } => Ok(Expr::Abs {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Floor { value } => Ok(Expr::Floor {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Ceil { value } => Ok(Expr::Ceil {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Round { value } => Ok(Expr::Round {
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Log { base, value } => Ok(Expr::Log {
                base: Box::new(base.substitute(bindings)?),
                value: Box::new(value.substitute(bindings)?),
            }),
            Expr::Max { left, right } => Ok(Expr::Max {
                left: Box::new(left.substitute(bindings)?),
                right: Box::new(right.substitute(bindings)?),
            }),
            Expr::Min { left, right } => Ok(Expr::Min {
                left: Box::new(left.substitute(bindings)?),
                right: Box::new(right.substitute(bindings)?),
            }),
        }
    }
}

impl SimplifyRequest {
    pub fn simplify(&self) -> SimplifyResponse {
        match validate_expr_limits(&self.expr).and_then(|_| self.expr.simplify()) {
            Ok(expr) => SimplifyResponse::Simplified {
                contract_version: CONTRACT_VERSION.to_owned(),
                expr,
                checks: vec![Check {
                    name: "symbolic_identities_applied".to_owned(),
                    passed: true,
                }],
            },
            Err(reason) => SimplifyResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }
}

impl SubstituteRequest {
    pub fn substitute(&self) -> SubstituteResponse {
        match validate_expr_limits(&self.expr)
            .and_then(|_| validate_bindings(&self.bindings))
            .and_then(|_| self.expr.substitute(&self.bindings))
            .and_then(|expr| expr.simplify())
        {
            Ok(expr) => SubstituteResponse::Substituted {
                contract_version: CONTRACT_VERSION.to_owned(),
                expr,
                checks: vec![
                    Check {
                        name: "all_symbols_bound".to_owned(),
                        passed: true,
                    },
                    Check {
                        name: "binding_values_exactly_evaluable".to_owned(),
                        passed: true,
                    },
                    Check {
                        name: "symbolic_identities_applied".to_owned(),
                        passed: true,
                    },
                ],
            },
            Err(reason) => SubstituteResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }
}

pub fn schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}.json"),
        "title": "agent-calc calc1 request",
        "description": "Typed exact rational expression request for the agent-calc executable contract.",
        "type": "object",
        "required": ["expr"],
        "additionalProperties": false,
        "properties": {
            "decimal_places": {
                "type": "integer",
                "minimum": 0,
                "maximum": 128,
                "default": 12
            },
            "expr": { "$ref": "#/$defs/Expr" }
        },
        "$defs": {
            "Expr": {
                "oneOf": [
                    { "$ref": "#/$defs/Integer" },
                    { "$ref": "#/$defs/Rational" },
                    { "$ref": "#/$defs/Symbol" },
                    { "$ref": "#/$defs/Add" },
                    { "$ref": "#/$defs/Sub" },
                    { "$ref": "#/$defs/Mul" },
                    { "$ref": "#/$defs/Div" },
                    { "$ref": "#/$defs/Pow" },
                    { "$ref": "#/$defs/Neg" },
                    { "$ref": "#/$defs/Sqrt" },
                    { "$ref": "#/$defs/Exp" },
                    { "$ref": "#/$defs/Ln" },
                    { "$ref": "#/$defs/Sin" },
                    { "$ref": "#/$defs/Cos" },
                    { "$ref": "#/$defs/Tan" },
                    { "$ref": "#/$defs/Abs" },
                    { "$ref": "#/$defs/Floor" },
                    { "$ref": "#/$defs/Ceil" },
                    { "$ref": "#/$defs/Round" },
                    { "$ref": "#/$defs/Log" },
                    { "$ref": "#/$defs/Max" },
                    { "$ref": "#/$defs/Min" }
                ]
            },
            "Integer": {
                "type": "object",
                "required": ["kind", "value"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "integer" },
                    "value": { "type": "string", "pattern": "^-?[0-9]+$" }
                }
            },
            "Rational": {
                "type": "object",
                "required": ["kind", "numerator", "denominator"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "rational" },
                    "numerator": { "type": "string", "pattern": "^-?[0-9]+$" },
                    "denominator": { "type": "string", "pattern": "^-?[0-9]+$" }
                }
            },
            "Symbol": {
                "type": "object",
                "required": ["kind", "name"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "symbol" },
                    "name": { "type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$" }
                }
            },
            "Add": {
                "type": "object",
                "required": ["kind", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "add" },
                    "left": { "$ref": "#/$defs/Expr" },
                    "right": { "$ref": "#/$defs/Expr" }
                }
            },
            "Sub": {
                "type": "object",
                "required": ["kind", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "sub" },
                    "left": { "$ref": "#/$defs/Expr" },
                    "right": { "$ref": "#/$defs/Expr" }
                }
            },
            "Mul": {
                "type": "object",
                "required": ["kind", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "mul" },
                    "left": { "$ref": "#/$defs/Expr" },
                    "right": { "$ref": "#/$defs/Expr" }
                }
            },
            "Div": {
                "type": "object",
                "required": ["kind", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "div" },
                    "left": { "$ref": "#/$defs/Expr" },
                    "right": { "$ref": "#/$defs/Expr" }
                }
            },
            "Pow": {
                "type": "object",
                "required": ["kind", "base", "exponent"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "pow" },
                    "base": { "$ref": "#/$defs/Expr" },
                    "exponent": {
                        "type": "integer",
                        "minimum": -1024,
                        "maximum": 1024
                    }
                }
            },
            "Neg": unary_value_def("neg"),
            "Sqrt": unary_value_def("sqrt"),
            "Exp":  unary_value_def("exp"),
            "Ln":   unary_value_def("ln"),
            "Sin":  unary_value_def("sin"),
            "Cos":  unary_value_def("cos"),
            "Tan":  unary_value_def("tan"),
            "Abs":  unary_value_def("abs"),
            "Floor": unary_value_def("floor"),
            "Ceil":  unary_value_def("ceil"),
            "Round": unary_value_def("round"),
            "Log": {
                "type": "object",
                "required": ["kind", "base", "value"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "log" },
                    "base": { "$ref": "#/$defs/Expr" },
                    "value": { "$ref": "#/$defs/Expr" }
                }
            },
            "Max": {
                "type": "object",
                "required": ["kind", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "max" },
                    "left": { "$ref": "#/$defs/Expr" },
                    "right": { "$ref": "#/$defs/Expr" }
                }
            },
            "Min": {
                "type": "object",
                "required": ["kind", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "kind": { "const": "min" },
                    "left": { "$ref": "#/$defs/Expr" },
                    "right": { "$ref": "#/$defs/Expr" }
                }
            }
        }
    })
}

fn unary_value_def(kind: &str) -> Value {
    json!({
        "type": "object",
        "required": ["kind", "value"],
        "additionalProperties": false,
        "properties": {
            "kind": { "const": kind },
            "value": { "$ref": "#/$defs/Expr" }
        }
    })
}

fn default_decimal_places() -> usize {
    12
}

pub fn validate_decimal_places(decimal_places: usize) -> Result<(), String> {
    if decimal_places > MAX_DECIMAL_PLACES {
        Err(format!("decimal_places must be <= {MAX_DECIMAL_PLACES}"))
    } else {
        Ok(())
    }
}

pub fn validate_expr_limits(expr: &Expr) -> Result<(), String> {
    let mut nodes = 0usize;
    validate_expr_limits_inner(expr, 1, &mut nodes)
}

fn validate_expr_limits_inner(expr: &Expr, depth: usize, nodes: &mut usize) -> Result<(), String> {
    if depth > MAX_EXPR_DEPTH {
        return Err(format!("expression depth must be <= {MAX_EXPR_DEPTH}"));
    }
    *nodes = nodes
        .checked_add(1)
        .ok_or_else(|| "expression node count overflowed".to_owned())?;
    if *nodes > MAX_EXPR_NODES {
        return Err(format!("expression node count must be <= {MAX_EXPR_NODES}"));
    }

    match expr {
        Expr::Integer { value } => validate_integer_text(value),
        Expr::Rational {
            numerator,
            denominator,
        } => {
            validate_integer_text(numerator)?;
            validate_integer_text(denominator)
        }
        Expr::Symbol { name } => validate_symbol_length(name),
        Expr::Add { left, right }
        | Expr::Sub { left, right }
        | Expr::Mul { left, right }
        | Expr::Div { left, right } => {
            validate_expr_limits_inner(left, depth + 1, nodes)?;
            validate_expr_limits_inner(right, depth + 1, nodes)
        }
        Expr::Pow { base, exponent } => {
            if exponent.unsigned_abs() > MAX_ABS_EXPONENT as u32 {
                return Err(format!("absolute exponent must be <= {MAX_ABS_EXPONENT}"));
            }
            validate_expr_limits_inner(base, depth + 1, nodes)
        }
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
        | Expr::Round { value } => validate_expr_limits_inner(value, depth + 1, nodes),
        Expr::Log { base, value }
        | Expr::Max {
            left: base,
            right: value,
        }
        | Expr::Min {
            left: base,
            right: value,
        } => {
            validate_expr_limits_inner(base, depth + 1, nodes)?;
            validate_expr_limits_inner(value, depth + 1, nodes)
        }
    }
}

fn validate_integer_text(value: &str) -> Result<(), String> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid integer `{value}`"));
    }
    if digits.len() > MAX_INTEGER_DIGITS {
        return Err(format!(
            "integer digit count must be <= {MAX_INTEGER_DIGITS}"
        ));
    }
    Ok(())
}

fn validate_symbol_text(name: &str) -> Result<(), String> {
    validate_symbol_length(name)?;
    if is_valid_symbol_name(name) {
        Ok(())
    } else {
        Err(format!("invalid symbol name `{name}`"))
    }
}

fn validate_symbol_length(name: &str) -> Result<(), String> {
    if name.len() > MAX_SYMBOL_LEN {
        return Err(format!("symbol length must be <= {MAX_SYMBOL_LEN}"));
    }
    Ok(())
}

pub fn classify_error(reason: &str) -> ErrorCode {
    if reason.contains("denominator must not be zero")
        || reason.contains("division by zero")
        || reason.contains("must not contain zero")
        || reason.contains("undefined when 1 + rate is zero")
    {
        ErrorCode::DivisionByZero
    } else if reason.contains("invalid integer") {
        ErrorCode::InvalidInteger
    } else if reason.contains("invalid symbol")
        || reason.contains("invalid binding name")
        || reason.contains("invalid polynomial variable")
    {
        ErrorCode::InvalidSymbol
    } else if reason.contains("unbound symbol") || reason.contains("missing binding") {
        ErrorCode::UnboundSymbol
    } else if reason.contains("must be <=")
        || reason.contains("too many")
        || reason.contains("exceed")
        || reason.contains("overflow")
    {
        ErrorCode::ResourceLimit
    } else if reason.contains("unsupported")
        || reason.contains("not supported")
        || reason.contains("nonlinear")
    {
        ErrorCode::Unsupported
    } else {
        ErrorCode::InvalidInput
    }
}

pub fn simplify_schema_json() -> Value {
    let mut schema = schema_json();
    schema["$id"] = json!(format!(
        "https://agent-calc.local/schema/{CONTRACT_VERSION}/simplify.json"
    ));
    schema["title"] = json!("agent-calc calc1 simplify request");
    schema["description"] =
        json!("Typed symbolic simplification request for the agent-calc executable contract.");
    schema
}

pub fn substitute_schema_json() -> Value {
    let mut schema = schema_json();
    schema["$id"] = json!(format!(
        "https://agent-calc.local/schema/{CONTRACT_VERSION}/substitute.json"
    ));
    schema["title"] = json!("agent-calc calc1 substitute request");
    schema["description"] =
        json!("Typed symbolic substitution request for the agent-calc executable contract.");
    schema["required"] = json!(["expr", "bindings"]);
    schema["properties"]["bindings"] = json!({
        "type": "object",
        "propertyNames": { "pattern": "^[A-Za-z_][A-Za-z0-9_]*$" },
        "additionalProperties": { "$ref": "#/$defs/Expr" },
        "default": {}
    });
    schema
}

fn integer_expr(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
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

fn validate_bindings(bindings: &BTreeMap<String, Expr>) -> Result<(), String> {
    if bindings.len() > MAX_BINDINGS {
        return Err(format!("binding count must be <= {MAX_BINDINGS}"));
    }
    for (name, expr) in bindings {
        validate_symbol_text(name).map_err(|_| format!("invalid binding name `{name}`"))?;
        validate_expr_limits(expr)?;
        expr.evaluate()
            .map_err(|reason| format!("binding `{name}` is not exactly evaluable: {reason}"))?;
    }
    Ok(())
}
