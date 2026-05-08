use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, ExactRational, classify_error},
};
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
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
    Gcd {
        left: PolynomialInput,
        right: PolynomialInput,
    },
    Lcm {
        left: PolynomialInput,
        right: PolynomialInput,
    },
    Factor {
        polynomial: PolynomialInput,
    },
    IsolateRoots {
        polynomial: PolynomialInput,
        tolerance: Expr,
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
    Factors {
        contract_version: String,
        variable: String,
        factors: Vec<Vec<ExactRational>>,
        checks: Vec<PolynomialCheck>,
    },
    RootIntervals {
        contract_version: String,
        variable: String,
        intervals: Vec<RootInterval>,
        checks: Vec<PolynomialCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RootInterval {
    pub lower: ExactRational,
    pub upper: ExactRational,
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
            Ok(PolynomialOutput::Factors { variable, factors }) => PolynomialResponse::Factors {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable,
                factors: factors
                    .iter()
                    .map(|f| f.iter().map(exact_rational).collect())
                    .collect(),
                checks: default_checks(),
            },
            Ok(PolynomialOutput::RootIntervals {
                variable,
                intervals,
            }) => PolynomialResponse::RootIntervals {
                contract_version: CONTRACT_VERSION.to_owned(),
                variable,
                intervals: intervals
                    .into_iter()
                    .map(|(lo, hi)| RootInterval {
                        lower: exact_rational(&lo),
                        upper: exact_rational(&hi),
                    })
                    .collect(),
                checks: default_checks(),
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
            PolynomialRequest::Gcd { left, right } => {
                let left = Polynomial::parse(left)?;
                let right = Polynomial::parse(right)?;
                ensure_same_variable(&left, &right)?;
                Ok(PolynomialOutput::Polynomial {
                    variable: left.variable.clone(),
                    coefficients: poly_gcd(&left.coefficients, &right.coefficients)?,
                })
            }
            PolynomialRequest::Lcm { left, right } => {
                let left = Polynomial::parse(left)?;
                let right = Polynomial::parse(right)?;
                ensure_same_variable(&left, &right)?;
                Ok(PolynomialOutput::Polynomial {
                    variable: left.variable.clone(),
                    coefficients: poly_lcm(&left.coefficients, &right.coefficients)?,
                })
            }
            PolynomialRequest::Factor { polynomial } => {
                let polynomial = Polynomial::parse(polynomial)?;
                Ok(PolynomialOutput::Factors {
                    variable: polynomial.variable,
                    factors: factor_coefficients(&polynomial.coefficients)?,
                })
            }
            PolynomialRequest::IsolateRoots {
                polynomial,
                tolerance,
            } => {
                let polynomial = Polynomial::parse(polynomial)?;
                let tol = tolerance.evaluate()?;
                if tol <= Rational::zero() {
                    return Err("tolerance must be positive".to_owned());
                }
                Ok(PolynomialOutput::RootIntervals {
                    variable: polynomial.variable,
                    intervals: isolate_roots_coefficients(&polynomial.coefficients, &tol)?,
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
            {"$ref": "#/$defs/Evaluate"},
            {"$ref": "#/$defs/IsolateRoots"}
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
                    "intent": {"enum": ["normalize", "derivative", "solve", "factor"]},
                    "polynomial": {"$ref": "#/$defs/Polynomial"}
                }
            },
            "BinaryPolynomial": {
                "type": "object",
                "required": ["intent", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["add", "mul", "gcd", "lcm"]},
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
            },
            "IsolateRoots": {
                "type": "object",
                "required": ["intent", "polynomial", "tolerance"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "isolate_roots"},
                    "polynomial": {"$ref": "#/$defs/Polynomial"},
                    "tolerance": {"$ref": "#/$defs/Expr"}
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
    Factors {
        variable: String,
        factors: Vec<Vec<Rational>>,
    },
    RootIntervals {
        variable: String,
        intervals: Vec<(Rational, Rational)>,
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
        3 | 4 => solve_by_rational_roots(coefficients),
        degree => Err(format!("polynomial degree {degree} is not supported")),
    }
}

fn solve_by_rational_roots(coefficients: &[Rational]) -> Result<Vec<Rational>, String> {
    let scaled = scale_to_integers(coefficients)?;
    let const_term = scaled[0].abs();
    let lead_term = scaled.last().unwrap().abs();
    let p_divs = positive_divisors(&const_term);
    let q_divs = positive_divisors(&lead_term);

    let mut candidates: Vec<Rational> = Vec::new();
    for p in &p_divs {
        for q in &q_divs {
            for &sign in &[1i64, -1i64] {
                let num = BigInt::from(sign) * p;
                if let Ok(r) = Rational::new(num, q.clone()) {
                    candidates.push(r);
                }
            }
        }
    }
    candidates.sort();
    candidates.dedup();

    let mut found: Vec<Rational> = Vec::new();
    let mut remaining = coefficients.to_vec();

    'outer: while remaining.len() >= 4 {
        for cand in &candidates {
            if evaluate_coefficients(&remaining, cand)?.is_zero() {
                found.push(cand.clone());
                remaining = synthetic_div(&remaining, cand)?;
                normalize_coefficients(&mut remaining);
                continue 'outer;
            }
        }
        break;
    }

    match remaining.len() - 1 {
        2 => {
            if let Ok(more) = solve_quadratic(&remaining) {
                found.extend(more);
            }
        }
        _ if found.is_empty() => {
            return Err(format!(
                "degree {} polynomial has no rational roots",
                coefficients.len() - 1
            ));
        }
        _ => {}
    }

    found.sort();
    found.dedup();
    Ok(found)
}

fn scale_to_integers(coefficients: &[Rational]) -> Result<Vec<BigInt>, String> {
    let mut lcm = BigInt::one();
    for c in coefficients {
        let d = c.denominator().clone();
        lcm = bigint_lcm(lcm, d);
    }
    coefficients
        .iter()
        .map(|c| {
            let scaled = c.numerator() * (&lcm / c.denominator());
            Ok(scaled)
        })
        .collect()
}

fn bigint_lcm(a: BigInt, b: BigInt) -> BigInt {
    let g = bigint_gcd(a.abs(), b.abs());
    if g.is_zero() {
        BigInt::one()
    } else {
        a.abs() / g * b.abs()
    }
}

fn bigint_gcd(mut a: BigInt, mut b: BigInt) -> BigInt {
    while !b.is_zero() {
        let t = b.clone();
        b = a % &t;
        a = t;
    }
    a
}

fn positive_divisors(n: &BigInt) -> Vec<BigInt> {
    if n.is_zero() {
        return vec![BigInt::one()];
    }
    let n_abs = n.abs();
    let mut divs = Vec::new();
    let mut i = BigInt::one();
    while &i * &i <= n_abs {
        if (&n_abs % &i).is_zero() {
            divs.push(i.clone());
            let other = &n_abs / &i;
            if other != i {
                divs.push(other);
            }
        }
        i += BigInt::one();
    }
    divs
}

fn synthetic_div(coefficients: &[Rational], root: &Rational) -> Result<Vec<Rational>, String> {
    let n = coefficients.len() - 1;
    let mut result = vec![Rational::zero(); n];
    result[n - 1] = coefficients[n].clone();
    for i in (0..n - 1).rev() {
        result[i] = result[i + 1]
            .checked_mul(root)
            .and_then(|v| v.checked_add(&coefficients[i + 1]))
            .map_err(|e| e.to_string())?;
    }
    Ok(result)
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
            name: "degree_at_most_four".to_owned(),
            passed: true,
        },
        PolynomialCheck {
            name: "roots_are_exact_rationals".to_owned(),
            passed: true,
        },
    ]
}

fn poly_div(
    dividend: &[Rational],
    divisor: &[Rational],
) -> Result<(Vec<Rational>, Vec<Rational>), String> {
    if divisor.iter().all(|c| c.is_zero()) {
        return Err("polynomial division by zero".to_owned());
    }
    let mut rem = dividend.to_vec();
    let mut div_norm = divisor.to_vec();
    normalize_coefficients(&mut rem);
    normalize_coefficients(&mut div_norm);

    if rem.len() < div_norm.len() {
        return Ok((vec![Rational::zero()], rem));
    }

    let quot_len = rem.len() - div_norm.len() + 1;
    let mut quot = vec![Rational::zero(); quot_len];
    let lead_d = div_norm.last().unwrap().clone();

    for i in (0..quot_len).rev() {
        let lead_r = rem[i + div_norm.len() - 1].clone();
        if lead_r.is_zero() {
            continue;
        }
        let c = lead_r.checked_div(&lead_d).map_err(|e| e.to_string())?;
        quot[i] = c.clone();
        for (j, dj) in div_norm.iter().enumerate() {
            let sub = c.checked_mul(dj).map_err(|e| e.to_string())?;
            rem[i + j] = rem[i + j].checked_sub(&sub).map_err(|e| e.to_string())?;
        }
    }

    normalize_coefficients(&mut quot);
    normalize_coefficients(&mut rem);
    Ok((quot, rem))
}

fn poly_rem(p: &[Rational], q: &[Rational]) -> Result<Vec<Rational>, String> {
    Ok(poly_div(p, q)?.1)
}

fn poly_gcd(p: &[Rational], q: &[Rational]) -> Result<Vec<Rational>, String> {
    let mut a = p.to_vec();
    let mut b = q.to_vec();
    normalize_coefficients(&mut a);
    normalize_coefficients(&mut b);

    loop {
        if b.len() == 1 && b[0].is_zero() {
            break;
        }
        let r = poly_rem(&a, &b)?;
        a = b;
        b = r;
        normalize_coefficients(&mut b);
    }

    if let Some(lead) = a.last().cloned()
        && !lead.is_zero()
    {
        for c in &mut a {
            *c = c.checked_div(&lead).map_err(|e| e.to_string())?;
        }
    }

    Ok(a)
}

fn poly_lcm(p: &[Rational], q: &[Rational]) -> Result<Vec<Rational>, String> {
    let g = poly_gcd(p, q)?;
    let (p_div_g, _rem) = poly_div(p, &g)?;
    let mut result = mul_coefficients(&p_div_g, q)?;

    if let Some(lead) = result.last().cloned()
        && !lead.is_zero()
    {
        for c in &mut result {
            *c = c.checked_div(&lead).map_err(|e| e.to_string())?;
        }
    }

    Ok(result)
}

fn factor_coefficients(coefficients: &[Rational]) -> Result<Vec<Vec<Rational>>, String> {
    let mut remaining = coefficients.to_vec();
    let mut factors: Vec<Vec<Rational>> = Vec::new();

    let lead = remaining.last().unwrap().clone();
    if lead != Rational::one() {
        for c in &mut remaining {
            *c = c.checked_div(&lead).map_err(|e| e.to_string())?;
        }
        factors.push(vec![lead]);
    }

    loop {
        let degree = remaining.len() - 1;
        if degree == 0 {
            break;
        }
        if degree == 1 {
            factors.push(remaining);
            break;
        }
        if degree == 2 {
            match solve_quadratic(&remaining) {
                Ok(roots) => {
                    for root in &roots {
                        let neg = root.checked_neg().map_err(|e| e.to_string())?;
                        factors.push(vec![neg, Rational::one()]);
                    }
                }
                _ => {
                    factors.push(remaining);
                }
            }
            break;
        }

        let scaled = scale_to_integers(&remaining)?;
        let const_term = scaled[0].abs();
        let lead_term = scaled.last().unwrap().abs();
        let p_divs = positive_divisors(&const_term);
        let q_divs = positive_divisors(&lead_term);

        let mut found = false;
        'root_search: for p_val in &p_divs {
            for q_val in &q_divs {
                for &sign in &[1i64, -1i64] {
                    let num = BigInt::from(sign) * p_val;
                    if let Ok(r) = Rational::new(num, q_val.clone())
                        && evaluate_coefficients(&remaining, &r)?.is_zero()
                    {
                        let neg = r.checked_neg().map_err(|e| e.to_string())?;
                        factors.push(vec![neg, Rational::one()]);
                        remaining = synthetic_div(&remaining, &r)?;
                        normalize_coefficients(&mut remaining);
                        found = true;
                        break 'root_search;
                    }
                }
            }
        }

        if !found {
            factors.push(remaining);
            break;
        }
    }

    Ok(factors)
}

fn sturm_sequence(p: &[Rational]) -> Result<Vec<Vec<Rational>>, String> {
    let p0 = p.to_vec();
    let p1 = derivative_coefficients(&p0)?;
    if p1.len() == 1 && p1[0].is_zero() {
        return Ok(vec![p0]);
    }
    let mut seq = vec![p0, p1];

    loop {
        let last = seq.last().unwrap().clone();
        if last.len() == 1 {
            break;
        }
        let prev = seq[seq.len() - 2].clone();
        let r = poly_rem(&prev, &last)?;
        let neg_r: Vec<Rational> = r
            .iter()
            .map(|c| c.checked_neg().map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let mut neg_r = neg_r;
        normalize_coefficients(&mut neg_r);
        if neg_r.len() == 1 && neg_r[0].is_zero() {
            break;
        }
        seq.push(neg_r);
    }

    Ok(seq)
}

fn sign_changes_at(seq: &[Vec<Rational>], x: &Rational) -> Result<usize, String> {
    let mut changes = 0usize;
    let mut last: Option<Rational> = None;
    for poly in seq {
        let v = evaluate_coefficients(poly, x)?;
        if v.is_zero() {
            continue;
        }
        if let Some(ref lv) = last
            && lv.checked_mul(&v).map_err(|e| e.to_string())?.is_negative()
        {
            changes += 1;
        }
        last = Some(v);
    }
    Ok(changes)
}

fn cauchy_bound(coefficients: &[Rational]) -> Result<Rational, String> {
    let lead_abs = coefficients.last().unwrap().abs();
    let mut max_ratio = Rational::zero();
    for c in &coefficients[..coefficients.len() - 1] {
        let ratio = c.abs().checked_div(&lead_abs).map_err(|e| e.to_string())?;
        if ratio.cmp(&max_ratio) == std::cmp::Ordering::Greater {
            max_ratio = ratio;
        }
    }
    Rational::one()
        .checked_add(&max_ratio)
        .map_err(|e| e.to_string())
}

fn isolate_interval(
    sturm: &[Vec<Rational>],
    lo: Rational,
    hi: Rational,
    tolerance: &Rational,
    depth: usize,
    results: &mut Vec<(Rational, Rational)>,
) -> Result<(), String> {
    let sc_lo = sign_changes_at(sturm, &lo)? as isize;
    let sc_hi = sign_changes_at(sturm, &hi)? as isize;
    let count = sc_lo - sc_hi;
    if count <= 0 {
        return Ok(());
    }
    if depth > 200 {
        return Err("root isolation exceeded maximum recursion depth".to_owned());
    }
    let width = hi.checked_sub(&lo).map_err(|e| e.to_string())?;
    if count == 1 && &width <= tolerance {
        results.push((lo, hi));
        return Ok(());
    }
    let two = Rational::integer(2);
    let mid = lo
        .checked_add(&hi)
        .and_then(|s| s.checked_div(&two))
        .map_err(|e| e.to_string())?;
    let next_depth = depth + 1;
    isolate_interval(sturm, lo, mid.clone(), tolerance, next_depth, results)?;
    isolate_interval(sturm, mid, hi, tolerance, next_depth, results)?;
    Ok(())
}

fn isolate_roots_coefficients(
    coefficients: &[Rational],
    tolerance: &Rational,
) -> Result<Vec<(Rational, Rational)>, String> {
    let sturm = sturm_sequence(coefficients)?;
    let bound = cauchy_bound(coefficients)?;
    let neg_bound = bound.checked_neg().map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    isolate_interval(&sturm, neg_bound, bound, tolerance, 0, &mut results)?;
    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
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
                            name: "degree_at_most_four".to_owned(),
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
        // degree 5 is still unsupported
        assert!(matches!(
            (PolynomialRequest::Solve {
                polynomial: poly(vec![int(1), int(0), int(0), int(0), int(0), int(1)]),
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason == "polynomial degree 5 is not supported"
        ));
        // x^3 + 1 has no rational roots (x = -1 is a root actually... wait)
        // x^3 - 2 = 0: only irrational roots
        assert!(matches!(
            (PolynomialRequest::Solve {
                polynomial: poly(vec![int(-2), int(0), int(0), int(1)]),
            })
            .evaluate(),
            PolynomialResponse::Error { reason, .. } if reason.contains("no rational roots")
        ));
    }

    #[test]
    fn solves_cubic_equations_with_rational_roots() {
        // x^3 - 6x^2 + 11x - 6 = 0 → roots: 1, 2, 3
        // coefficients ascending: [-6, 11, -6, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-6), int(11), int(-6), int(1)]),
                }
                .evaluate()
            ),
            vec!["1", "2", "3"]
        );
        // x^3 - x = x(x-1)(x+1) → roots: -1, 0, 1
        // ascending: [0, -1, 0, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(0), int(-1), int(0), int(1)]),
                }
                .evaluate()
            ),
            vec!["-1", "0", "1"]
        );
        // (x-2)^3 = x^3 - 6x^2 + 12x - 8 → triple root at 2
        // ascending: [-8, 12, -6, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-8), int(12), int(-6), int(1)]),
                }
                .evaluate()
            ),
            vec!["2"]
        );
    }

    #[test]
    fn solves_quartic_equations_with_rational_roots() {
        // (x-1)(x-2)(x-3)(x-4) = x^4 - 10x^3 + 35x^2 - 50x + 24
        // ascending: [24, -50, 35, -10, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(24), int(-50), int(35), int(-10), int(1)]),
                }
                .evaluate()
            ),
            vec!["1", "2", "3", "4"]
        );
        // x^4 - 1 = (x-1)(x+1)(x^2+1) → rational roots: -1, 1
        // ascending: [-1, 0, 0, 0, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-1), int(0), int(0), int(0), int(1)]),
                }
                .evaluate()
            ),
            vec!["-1", "1"]
        );
    }

    #[test]
    fn solves_cubic_with_one_rational_and_irrational_pair() {
        // (x-2)(x^2+1) = x^3 - 2x^2 + x - 2 → one rational root: 2
        // ascending: [-2, 1, -2, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-2), int(1), int(-2), int(1)]),
                }
                .evaluate()
            ),
            vec!["2"]
        );
        // (x-3)(x^2+2): root 3 requires other=6/2=3 from positive_divisors
        // (kills positive_divisors / → * mutation which would miss root 3)
        // ascending: [-6, 2, -3, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-6), int(2), int(-3), int(1)]),
                }
                .evaluate()
            ),
            vec!["3"]
        );
    }

    #[test]
    fn solves_quartic_with_one_rational_and_irrational_cubic_factor() {
        // (x-1)(x^3+2): quartic with 1 rational root; cubic factor has no rational roots
        // kills "replace match guard found.is_empty() with true" mutation
        // ascending: [-2, 2, 0, -1, 1]
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly(vec![int(-2), int(2), int(0), int(-1), int(1)]),
                }
                .evaluate()
            ),
            vec!["1"]
        );
    }

    #[test]
    fn solves_cubic_with_rational_coefficients() {
        // (1/2)x^3 - 3x^2 + (11/2)x - 3 = 0 → multiply by 2 → roots 1, 2, 3
        // kills scale_to_integers, bigint_lcm, bigint_gcd mutations (lcm = 2 ≠ 1)
        let half = Expr::Rational {
            numerator: "1".to_owned(),
            denominator: "2".to_owned(),
        };
        let eleven_halves = Expr::Rational {
            numerator: "11".to_owned(),
            denominator: "2".to_owned(),
        };
        let poly_input = PolynomialInput {
            variable: "x".to_owned(),
            coefficients: vec![int(-3), eleven_halves, int(-3), half],
        };
        assert_eq!(
            roots(
                PolynomialRequest::Solve {
                    polynomial: poly_input
                }
                .evaluate()
            ),
            vec!["1", "2", "3"]
        );
    }

    #[test]
    fn bigint_gcd_computes_correctly() {
        assert_eq!(
            bigint_gcd(BigInt::from(12), BigInt::from(8)),
            BigInt::from(4)
        );
        assert_eq!(
            bigint_gcd(BigInt::from(7), BigInt::from(3)),
            BigInt::from(1)
        );
        assert_eq!(
            bigint_gcd(BigInt::from(0), BigInt::from(5)),
            BigInt::from(5)
        );
    }

    #[test]
    fn bigint_lcm_computes_correctly() {
        assert_eq!(
            bigint_lcm(BigInt::from(4), BigInt::from(6)),
            BigInt::from(12)
        );
        assert_eq!(
            bigint_lcm(BigInt::from(3), BigInt::from(7)),
            BigInt::from(21)
        );
        assert_eq!(
            bigint_lcm(BigInt::from(2), BigInt::from(2)),
            BigInt::from(2)
        );
    }

    #[test]
    fn scale_to_integers_uses_lcm_of_denominators() {
        // lcm(3, 2) = 6: coeff 1/3 → 2, coeff 1/2 → 3
        let coeffs = vec![
            Rational::new(1i64, 3i64).unwrap(),
            Rational::new(1i64, 2i64).unwrap(),
        ];
        assert_eq!(
            scale_to_integers(&coeffs).unwrap(),
            vec![BigInt::from(2), BigInt::from(3)]
        );
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn rat(n: i64, d: i64) -> Expr {
        Expr::Rational {
            numerator: n.to_string(),
            denominator: d.to_string(),
        }
    }

    fn r(n: i64, d: i64) -> Rational {
        Rational::new(n, d).unwrap()
    }

    fn factor_displays(response: PolynomialResponse) -> Vec<Vec<String>> {
        match response {
            PolynomialResponse::Factors { factors, .. } => factors
                .into_iter()
                .map(|f| f.into_iter().map(|c| c.display).collect())
                .collect(),
            other => panic!("expected factors, got {other:?}"),
        }
    }

    fn interval_displays(response: PolynomialResponse) -> Vec<(String, String)> {
        match response {
            PolynomialResponse::RootIntervals { intervals, .. } => intervals
                .into_iter()
                .map(|i| (i.lower.display, i.upper.display))
                .collect(),
            other => panic!("expected root_intervals, got {other:?}"),
        }
    }

    fn parse_display_f64(s: &str) -> f64 {
        if let Some((n, d)) = s.split_once('/') {
            n.parse::<f64>().unwrap() / d.parse::<f64>().unwrap()
        } else {
            s.parse().unwrap()
        }
    }

    // ── poly_div ──────────────────────────────────────────────────────────────

    #[test]
    fn poly_div_divides_exactly() {
        // (x^2 + 5x + 6) / (x + 2) = x + 3, rem 0
        let p = vec![r(6, 1), r(5, 1), r(1, 1)];
        let q = vec![r(2, 1), r(1, 1)];
        let (quot, rem) = poly_div(&p, &q).unwrap();
        assert_eq!(quot, vec![r(3, 1), r(1, 1)]);
        assert_eq!(rem, vec![r(0, 1)]);
    }

    #[test]
    fn poly_div_returns_remainder() {
        // (x^2 + 1) / (x + 1) = x - 1, rem 2
        let p = vec![r(1, 1), r(0, 1), r(1, 1)];
        let q = vec![r(1, 1), r(1, 1)];
        let (quot, rem) = poly_div(&p, &q).unwrap();
        assert_eq!(quot, vec![r(-1, 1), r(1, 1)]);
        assert_eq!(rem, vec![r(2, 1)]);
    }

    #[test]
    fn poly_div_degree_less_than_divisor() {
        // (x + 1) / (x^2 + 1) = 0, rem (x + 1)
        let p = vec![r(1, 1), r(1, 1)];
        let q = vec![r(1, 1), r(0, 1), r(1, 1)];
        let (quot, rem) = poly_div(&p, &q).unwrap();
        assert_eq!(quot, vec![r(0, 1)]);
        assert_eq!(rem, vec![r(1, 1), r(1, 1)]);
    }

    // ── poly_gcd ─────────────────────────────────────────────────────────────

    #[test]
    fn gcd_of_coprime_polynomials_is_one() {
        // gcd(x + 1, x + 2) = 1
        let p = vec![r(1, 1), r(1, 1)];
        let q = vec![r(2, 1), r(1, 1)];
        assert_eq!(poly_gcd(&p, &q).unwrap(), vec![r(1, 1)]);
    }

    #[test]
    fn gcd_finds_common_factor() {
        // gcd(x^2 + 5x + 6, x + 2) = x + 2
        let p = vec![r(6, 1), r(5, 1), r(1, 1)];
        let q = vec![r(2, 1), r(1, 1)];
        assert_eq!(poly_gcd(&p, &q).unwrap(), vec![r(2, 1), r(1, 1)]);
    }

    #[test]
    fn gcd_is_monic() {
        // gcd(x^2 - 1, 2x + 2) = x + 1 (monic)
        let p = vec![r(-1, 1), r(0, 1), r(1, 1)];
        let q = vec![r(2, 1), r(2, 1)];
        assert_eq!(poly_gcd(&p, &q).unwrap(), vec![r(1, 1), r(1, 1)]);
    }

    #[test]
    fn gcd_same_polynomial_is_itself() {
        // gcd(p, p) = p (monic)
        let p = vec![r(6, 1), r(5, 1), r(1, 1)];
        assert_eq!(
            poly_gcd(&p, &p.clone()).unwrap(),
            vec![r(6, 1), r(5, 1), r(1, 1)]
        );
    }

    // ── poly_lcm ─────────────────────────────────────────────────────────────

    #[test]
    fn lcm_of_coprime_is_product_monic() {
        // lcm(x + 1, x + 2) = (x+1)(x+2) = x^2 + 3x + 2 (monic)
        let p = vec![r(1, 1), r(1, 1)];
        let q = vec![r(2, 1), r(1, 1)];
        assert_eq!(poly_lcm(&p, &q).unwrap(), vec![r(2, 1), r(3, 1), r(1, 1)]);
    }

    #[test]
    fn lcm_with_common_factor() {
        // lcm(x^2+5x+6, x+2) = x^2+5x+6 (since x+2 | x^2+5x+6)
        let p = vec![r(6, 1), r(5, 1), r(1, 1)];
        let q = vec![r(2, 1), r(1, 1)];
        assert_eq!(poly_lcm(&p, &q).unwrap(), vec![r(6, 1), r(5, 1), r(1, 1)]);
    }

    // ── factor ───────────────────────────────────────────────────────────────

    #[test]
    fn factors_monic_quadratic_with_rational_roots() {
        // x^2 + 5x + 6 = (x+2)(x+3)
        let resp = PolynomialRequest::Factor {
            polynomial: poly(vec![int(6), int(5), int(1)]),
        }
        .evaluate();
        let mut f = factor_displays(resp);
        f.sort();
        assert_eq!(f, vec![vec!["2", "1"], vec!["3", "1"]]);
    }

    #[test]
    fn factors_extracts_leading_coefficient() {
        // 2x^2 + 10x + 12 = 2(x+2)(x+3)
        let resp = PolynomialRequest::Factor {
            polynomial: poly(vec![int(12), int(10), int(2)]),
        }
        .evaluate();
        let mut f = factor_displays(resp);
        f.sort();
        assert_eq!(f, vec![vec!["2"], vec!["2", "1"], vec!["3", "1"]]);
    }

    #[test]
    fn factors_irreducible_quadratic_returns_as_single_factor() {
        // x^2 + 1 is irreducible over Q
        let resp = PolynomialRequest::Factor {
            polynomial: poly(vec![int(1), int(0), int(1)]),
        }
        .evaluate();
        let f = factor_displays(resp);
        assert_eq!(f, vec![vec!["1", "0", "1"]]);
    }

    #[test]
    fn factors_cubic_completely() {
        // x^3 - 6x^2 + 11x - 6 = (x-1)(x-2)(x-3)
        let resp = PolynomialRequest::Factor {
            polynomial: poly(vec![int(-6), int(11), int(-6), int(1)]),
        }
        .evaluate();
        let mut f = factor_displays(resp);
        f.sort();
        assert_eq!(f, vec![vec!["-1", "1"], vec!["-2", "1"], vec!["-3", "1"]]);
    }

    #[test]
    fn factors_cubic_with_irreducible_quadratic() {
        // (x-2)(x^2+1) = x^3 - 2x^2 + x - 2
        // ascending: [-2, 1, -2, 1]
        let resp = PolynomialRequest::Factor {
            polynomial: poly(vec![int(-2), int(1), int(-2), int(1)]),
        }
        .evaluate();
        let mut f = factor_displays(resp);
        f.sort();
        assert_eq!(f, vec![vec!["-2", "1"], vec!["1", "0", "1"]]);
    }

    // ── gcd / lcm via CLI ────────────────────────────────────────────────────

    #[test]
    fn gcd_via_request_returns_polynomial() {
        // gcd(x^2+5x+6, x+2) → polynomial [2, 1] = x+2
        let resp = PolynomialRequest::Gcd {
            left: poly(vec![int(6), int(5), int(1)]),
            right: poly(vec![int(2), int(1)]),
        }
        .evaluate();
        match resp {
            PolynomialResponse::Polynomial { coefficients, .. } => {
                let d: Vec<String> = coefficients.into_iter().map(|c| c.display).collect();
                assert_eq!(d, vec!["2", "1"]);
            }
            other => panic!("expected polynomial, got {other:?}"),
        }
    }

    #[test]
    fn lcm_via_request_returns_polynomial() {
        // lcm(x+1, x+2) → x^2+3x+2 (monic)
        let resp = PolynomialRequest::Lcm {
            left: poly(vec![int(1), int(1)]),
            right: poly(vec![int(2), int(1)]),
        }
        .evaluate();
        match resp {
            PolynomialResponse::Polynomial { coefficients, .. } => {
                let d: Vec<String> = coefficients.into_iter().map(|c| c.display).collect();
                assert_eq!(d, vec!["2", "3", "1"]);
            }
            other => panic!("expected polynomial, got {other:?}"),
        }
    }

    // ── isolate_roots ─────────────────────────────────────────────────────────

    #[test]
    fn isolates_two_roots_of_quadratic() {
        // x^2 - 2 has roots ±√2 ≈ ±1.414; tolerance 1/10
        let resp = PolynomialRequest::IsolateRoots {
            polynomial: poly(vec![int(-2), int(0), int(1)]),
            tolerance: rat(1, 10),
        }
        .evaluate();
        let intervals = interval_displays(resp);
        assert_eq!(intervals.len(), 2);
        // First interval straddles -√2
        let lo0 = parse_display_f64(&intervals[0].0);
        let hi0 = parse_display_f64(&intervals[0].1);
        assert!(lo0 < -1.41 && hi0 > -1.42);
        // Second interval straddles +√2
        let lo1 = parse_display_f64(&intervals[1].0);
        let hi1 = parse_display_f64(&intervals[1].1);
        assert!(lo1 < 1.42 && hi1 > 1.41);
    }

    #[test]
    fn isolate_roots_finds_zero_roots_for_no_real_roots() {
        // x^2 + 1 has no real roots
        let resp = PolynomialRequest::IsolateRoots {
            polynomial: poly(vec![int(1), int(0), int(1)]),
            tolerance: rat(1, 100),
        }
        .evaluate();
        assert_eq!(interval_displays(resp).len(), 0);
    }

    #[test]
    fn isolate_roots_finds_three_roots_of_cubic() {
        // x^3 - 3x has roots -√3, 0, √3
        // ascending: [0, -3, 0, 1]
        let resp = PolynomialRequest::IsolateRoots {
            polynomial: poly(vec![int(0), int(-3), int(0), int(1)]),
            tolerance: rat(1, 100),
        }
        .evaluate();
        let intervals = interval_displays(resp);
        assert_eq!(intervals.len(), 3);
    }

    #[test]
    fn isolate_roots_rational_root_is_exact_when_at_boundary() {
        // x^2 - 4 = (x-2)(x+2): roots ±2 exactly
        let resp = PolynomialRequest::IsolateRoots {
            polynomial: poly(vec![int(-4), int(0), int(1)]),
            tolerance: rat(1, 10),
        }
        .evaluate();
        let intervals = interval_displays(resp);
        assert_eq!(intervals.len(), 2);
    }

    // ── poly_lcm normalization ────────────────────────────────────────────────

    #[test]
    fn lcm_normalizes_non_unit_lead_to_monic() {
        // lcm(2x, 2x+2): gcd=2(const), p/gcd=x, product=x*(2x+2)=2x^2+2x → monic x^2+x
        // If !lead.is_zero() guard is deleted: skips normalization, returns [0,2,2] not [0,1,1]
        let p = vec![r(0, 1), r(2, 1)];
        let q = vec![r(2, 1), r(2, 1)];
        let result = poly_lcm(&p, &q).unwrap();
        assert_eq!(result.last().unwrap(), &r(1, 1)); // monic
    }

    // ── factor negative roots ─────────────────────────────────────────────────

    #[test]
    fn factors_cubic_with_negative_roots() {
        // (x+1)(x+3)(x+5) = x^3 + 9x^2 + 23x + 15; ascending: [15,23,9,1]
        // Kills 'delete -' (only +sign tried → misses -1,-3,-5)
        // Kills '* → +' (sign+p ≠ sign*p for p≥2 → misses -3 and -5)
        let resp = PolynomialRequest::Factor {
            polynomial: poly(vec![int(15), int(23), int(9), int(1)]),
        }
        .evaluate();
        let mut f = factor_displays(resp);
        f.sort();
        assert_eq!(f, vec![vec!["1", "1"], vec!["3", "1"], vec!["5", "1"]]);
    }

    // ── isolate_interval unit tests ───────────────────────────────────────────

    #[test]
    fn isolate_interval_succeeds_at_depth_200() {
        // x has root 0. Interval [-1/20, 1/20], width=1/10 = tolerance.
        // depth > 200: false at depth=200 → proceeds, finds root → Ok.
        // Mutation depth==200 or depth>=200 → errors immediately.
        let coeffs = vec![r(0, 1), r(1, 1)];
        let sturm = sturm_sequence(&coeffs).unwrap();
        let mut results = Vec::new();
        let result = isolate_interval(&sturm, r(-1, 20), r(1, 20), &r(1, 10), 200, &mut results);
        assert!(result.is_ok(), "depth=200 should not trigger the guard");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn cauchy_bound_exact_value() {
        // 2x + 1 = [1, 2]: lead=2, non-lead=[1], ratio=1/2, bound=3/2.
        // Mutation len-1→len/1: also iterates over lead, ratio=max(1/2,1)=1, bound=2. Different!
        assert_eq!(cauchy_bound(&[r(1, 1), r(2, 1)]).unwrap(), r(3, 2));
    }

    #[test]
    fn isolate_interval_catches_left_branch_depth_increment() {
        // x + 3/2: root at -3/2, always in left half when bisecting [-2, 0].
        // With depth=200 and width=2 > 1/10: must recurse left via line 953.
        // Original (depth+1=201): left count=1, depth>200 → Err.
        // Mutation line 953 (depth*1=200): left count=1, depth not>200 → recurse → Ok.
        // Right half always has count=0 (root not there) → returns before depth guard.
        let coeffs = vec![r(3, 2), r(1, 1)];
        let sturm = sturm_sequence(&coeffs).unwrap();
        let mut results = Vec::new();
        let result = isolate_interval(&sturm, r(-2, 1), r(0, 1), &r(1, 10), 200, &mut results);
        assert!(result.is_err(), "left recursion must increment depth");
    }

    #[test]
    fn isolate_interval_catches_right_branch_depth_increment() {
        // x - 3/2: root at 3/2, always in right half when bisecting [0, 2].
        // With depth=200 and width=2 > 1/10: must recurse right via line 954.
        // Original (depth+1=201): right count=1, depth>200 → Err.
        // Left half count=0 → returns before depth guard (new guard order).
        // Mutation line 954 (depth*1=200): right count=1, depth not>200 → recurse → Ok.
        let coeffs = vec![r(-3, 2), r(1, 1)];
        let sturm = sturm_sequence(&coeffs).unwrap();
        let mut results = Vec::new();
        let result = isolate_interval(&sturm, r(0, 1), r(2, 1), &r(1, 10), 200, &mut results);
        assert!(result.is_err(), "right recursion must increment depth");
    }

    #[test]
    fn isolate_interval_errors_above_depth_200() {
        // x^2 - 2: two roots. depth=200, interval [-2,2], count=2, width=4 > tolerance.
        // Must bisect → recurses to depth=201 → depth>200 errors.
        // Mutation depth+1→depth*1: always stays at depth=200, eventually terminates → Ok.
        let coeffs = vec![r(-2, 1), r(0, 1), r(1, 1)];
        let sturm = sturm_sequence(&coeffs).unwrap();
        let mut results = Vec::new();
        let result = isolate_interval(&sturm, r(-2, 1), r(2, 1), &r(1, 100), 200, &mut results);
        assert!(
            result.is_err(),
            "should error when recursion exceeds depth 200"
        );
    }

    #[test]
    fn isolate_roots_wide_tolerance_gives_two_intervals() {
        // x^2 - 1 = (x-1)(x+1). Cauchy bound=2, interval [-2,2], count=2, width=4.
        // tolerance=4 (>= width): with &&→|| mutation, exits early on count=2 because width<=tol,
        //   returning 1 interval instead of 2.
        // with <=→> mutation: width>tol always true → never terminates (hits depth limit).
        let resp = PolynomialRequest::IsolateRoots {
            polynomial: poly(vec![int(-1), int(0), int(1)]),
            tolerance: rat(4, 1),
        }
        .evaluate();
        let intervals = interval_displays(resp);
        assert_eq!(intervals.len(), 2);
    }
}
