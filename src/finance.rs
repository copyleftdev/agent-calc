use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{
        ErrorCode, ExactRational, classify_error, schema_json, validate_decimal_places,
        validate_expr_limits,
    },
};
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cmp::Ordering;

const MAX_PERIODS: u32 = 1200;
const MAX_PRICE_DECIMAL_DIGITS: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PriceValue {
    /// Fixed-point decimal text, parsed directly into an exact rational.
    Decimal(String),
    Expression(Expr),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecimalRounding {
    /// Round to nearest; exact ties go to the result with an even final digit.
    #[default]
    HalfEven,
    /// Round to nearest; exact ties increase the magnitude.
    HalfAwayFromZero,
    /// Discard digits beyond the requested decimal places.
    TowardZero,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum FinanceRequest {
    FutureValue {
        present_value: Expr,
        rate: Expr,
        periods: u32,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
    },
    PresentValue {
        future_value: Expr,
        rate: Expr,
        periods: u32,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
    },
    DiscountFactor {
        rate: Expr,
        period: u32,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
    },
    NetPresentValue {
        rate: Expr,
        cash_flows: Vec<Expr>,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
    },
    Irr {
        cash_flows: Vec<Expr>,
        tolerance: f64,
    },
    Amortize {
        principal: Expr,
        annual_rate: Expr,
        periods: u32,
        periods_per_year: u32,
    },
    BondPrice {
        face: Expr,
        coupon_rate: Expr,
        periods: u32,
        yield_rate: Expr,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
    },
    DiscountTable {
        rate: Expr,
        periods: u32,
    },
    DiscountedCashFlow {
        /// Forecast cash flows for periods 1 through n.
        cash_flows: Vec<PriceValue>,
        discount_rate: PriceValue,
        /// Optional perpetual-growth terminal model at period n.
        #[serde(default)]
        terminal_growth_rate: Option<PriceValue>,
        #[serde(default = "default_decimal_places")]
        decimal_places: usize,
        #[serde(default)]
        rounding_mode: DecimalRounding,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AmortizationRow {
    pub period: u32,
    pub payment: ExactRational,
    pub interest: ExactRational,
    pub principal_paid: ExactRational,
    pub balance: ExactRational,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscountedCashFlowPrice {
    pub forecast_present_value: ExactRational,
    pub terminal_value: Option<ExactRational>,
    pub terminal_present_value: Option<ExactRational>,
    pub price: ExactRational,
    pub decimal: String,
    pub rounding_mode: DecimalRounding,
    pub checks: Vec<FinanceCheck>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FinanceResponse {
    Value {
        contract_version: String,
        exact: ExactRational,
        decimal: String,
        checks: Vec<FinanceCheck>,
    },
    Irr {
        contract_version: String,
        approximate_f64: f64,
        checks: Vec<FinanceCheck>,
    },
    Schedule {
        contract_version: String,
        payment: ExactRational,
        schedule: Vec<AmortizationRow>,
        checks: Vec<FinanceCheck>,
    },
    Table {
        contract_version: String,
        factors: Vec<ExactRational>,
        checks: Vec<FinanceCheck>,
    },
    PriceModel {
        contract_version: String,
        #[serde(flatten)]
        model: Box<DiscountedCashFlowPrice>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinanceCheck {
    pub name: String,
    pub passed: bool,
}

impl FinanceRequest {
    pub fn evaluate(&self) -> FinanceResponse {
        match self {
            FinanceRequest::FutureValue { .. }
            | FinanceRequest::PresentValue { .. }
            | FinanceRequest::DiscountFactor { .. }
            | FinanceRequest::NetPresentValue { .. } => match self.evaluate_scalar() {
                Ok((value, dp)) => scalar_value_response(value, dp),
                Err(reason) => error_response(reason),
            },
            FinanceRequest::BondPrice {
                face,
                coupon_rate,
                periods,
                yield_rate,
                decimal_places,
            } => match evaluate_bond_price(face, coupon_rate, *periods, yield_rate) {
                Ok(price) => scalar_value_response(price, *decimal_places),
                Err(reason) => error_response(reason),
            },
            FinanceRequest::Irr {
                cash_flows,
                tolerance,
            } => match evaluate_irr(cash_flows, *tolerance) {
                Ok(rate) => FinanceResponse::Irr {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    approximate_f64: rate,
                    checks: vec![FinanceCheck {
                        name: "approximate_bisection_irr".to_owned(),
                        passed: true,
                    }],
                },
                Err(reason) => error_response(reason),
            },
            FinanceRequest::Amortize {
                principal,
                annual_rate,
                periods,
                periods_per_year,
            } => match evaluate_amortize(principal, annual_rate, *periods, *periods_per_year) {
                Ok((payment, schedule)) => FinanceResponse::Schedule {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    payment: exact_rational(&payment),
                    schedule,
                    checks: vec![FinanceCheck {
                        name: "exact_rational_amortization".to_owned(),
                        passed: true,
                    }],
                },
                Err(reason) => error_response(reason),
            },
            FinanceRequest::DiscountTable { rate, periods } => {
                match evaluate_discount_table(rate, *periods) {
                    Ok(factors) => FinanceResponse::Table {
                        contract_version: CONTRACT_VERSION.to_owned(),
                        factors: factors.iter().map(exact_rational).collect(),
                        checks: vec![FinanceCheck {
                            name: "exact_rational_discount_factors".to_owned(),
                            passed: true,
                        }],
                    },
                    Err(reason) => error_response(reason),
                }
            }
            FinanceRequest::DiscountedCashFlow {
                cash_flows,
                discount_rate,
                terminal_growth_rate,
                decimal_places,
                rounding_mode,
            } => match evaluate_discounted_cash_flow(
                cash_flows,
                discount_rate,
                terminal_growth_rate.as_ref(),
                *decimal_places,
            ) {
                Ok(model) => FinanceResponse::PriceModel {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    model: Box::new(DiscountedCashFlowPrice {
                        forecast_present_value: exact_rational(&model.forecast_present_value),
                        terminal_value: model.terminal_value.as_ref().map(exact_rational),
                        terminal_present_value: model
                            .terminal_present_value
                            .as_ref()
                            .map(exact_rational),
                        price: exact_rational(&model.price),
                        decimal: rounded_decimal_string(
                            &model.price,
                            *decimal_places,
                            *rounding_mode,
                        ),
                        rounding_mode: *rounding_mode,
                        checks: vec![
                            FinanceCheck {
                                name: "exact_rational_discounted_cash_flow".to_owned(),
                                passed: true,
                            },
                            FinanceCheck {
                                name: "one_based_forecast_periods".to_owned(),
                                passed: true,
                            },
                        ],
                    }),
                },
                Err(reason) => error_response(reason),
            },
        }
    }

    fn evaluate_scalar(&self) -> Result<(Rational, usize), String> {
        match self {
            FinanceRequest::FutureValue {
                present_value,
                rate,
                periods,
                decimal_places,
            } => {
                validate_decimal_places(*decimal_places)?;
                validate_expr_limits(present_value)?;
                validate_expr_limits(rate)?;
                Ok((
                    future_value(
                        &present_value.evaluate().map_err(prefix_expr_error)?,
                        &rate.evaluate().map_err(prefix_rate_error)?,
                        *periods,
                    )?,
                    *decimal_places,
                ))
            }
            FinanceRequest::PresentValue {
                future_value,
                rate,
                periods,
                decimal_places,
            } => {
                validate_decimal_places(*decimal_places)?;
                validate_expr_limits(future_value)?;
                validate_expr_limits(rate)?;
                Ok((
                    present_value(
                        &future_value.evaluate().map_err(prefix_expr_error)?,
                        &rate.evaluate().map_err(prefix_rate_error)?,
                        *periods,
                    )?,
                    *decimal_places,
                ))
            }
            FinanceRequest::DiscountFactor {
                rate,
                period,
                decimal_places,
            } => {
                validate_decimal_places(*decimal_places)?;
                validate_expr_limits(rate)?;
                Ok((
                    discount_factor(&rate.evaluate().map_err(prefix_rate_error)?, *period)?,
                    *decimal_places,
                ))
            }
            FinanceRequest::NetPresentValue {
                rate,
                cash_flows,
                decimal_places,
            } => {
                validate_decimal_places(*decimal_places)?;
                validate_expr_limits(rate)?;
                for cash_flow in cash_flows {
                    validate_expr_limits(cash_flow)?;
                }
                Ok((
                    net_present_value(&rate.evaluate().map_err(prefix_rate_error)?, cash_flows)?,
                    *decimal_places,
                ))
            }
            _ => unreachable!(),
        }
    }
}

pub fn finance_schema_json() -> Value {
    let defs = schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/finance.json"),
        "title": "agent-calc calc1 finance request",
        "description": "Typed exact-rational time-value finance request.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/FutureValue"},
            {"$ref": "#/$defs/PresentValue"},
            {"$ref": "#/$defs/DiscountFactor"},
            {"$ref": "#/$defs/NetPresentValue"},
            {"$ref": "#/$defs/Irr"},
            {"$ref": "#/$defs/Amortize"},
            {"$ref": "#/$defs/BondPrice"},
            {"$ref": "#/$defs/DiscountTable"},
            {"$ref": "#/$defs/DiscountedCashFlow"}
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
            "FutureValue": {
                "type": "object",
                "required": ["intent", "present_value", "rate", "periods"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "future_value"},
                    "present_value": {"$ref": "#/$defs/Expr"},
                    "rate": {"$ref": "#/$defs/Expr"},
                    "periods": {"type": "integer", "minimum": 0},
                    "decimal_places": {"type": "integer", "minimum": 0, "maximum": 128, "default": 12}
                }
            },
            "PresentValue": {
                "type": "object",
                "required": ["intent", "future_value", "rate", "periods"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "present_value"},
                    "future_value": {"$ref": "#/$defs/Expr"},
                    "rate": {"$ref": "#/$defs/Expr"},
                    "periods": {"type": "integer", "minimum": 0},
                    "decimal_places": {"type": "integer", "minimum": 0, "maximum": 128, "default": 12}
                }
            },
            "DiscountFactor": {
                "type": "object",
                "required": ["intent", "rate", "period"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "discount_factor"},
                    "rate": {"$ref": "#/$defs/Expr"},
                    "period": {"type": "integer", "minimum": 0},
                    "decimal_places": {"type": "integer", "minimum": 0, "maximum": 128, "default": 12}
                }
            },
            "NetPresentValue": {
                "type": "object",
                "required": ["intent", "rate", "cash_flows"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "net_present_value"},
                    "rate": {"$ref": "#/$defs/Expr"},
                    "cash_flows": {
                        "type": "array",
                        "minItems": 1,
                        "items": {"$ref": "#/$defs/Expr"}
                    },
                    "decimal_places": {"type": "integer", "minimum": 0, "maximum": 128, "default": 12}
                }
            },
            "Irr": {
                "type": "object",
                "required": ["intent", "cash_flows", "tolerance"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "irr"},
                    "cash_flows": {
                        "type": "array",
                        "minItems": 2,
                        "items": {"$ref": "#/$defs/Expr"}
                    },
                    "tolerance": {"type": "number", "exclusiveMinimum": 0}
                }
            },
            "Amortize": {
                "type": "object",
                "required": ["intent", "principal", "annual_rate", "periods", "periods_per_year"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "amortize"},
                    "principal": {"$ref": "#/$defs/Expr"},
                    "annual_rate": {"$ref": "#/$defs/Expr"},
                    "periods": {"type": "integer", "minimum": 1, "maximum": MAX_PERIODS},
                    "periods_per_year": {"type": "integer", "minimum": 1}
                }
            },
            "BondPrice": {
                "type": "object",
                "required": ["intent", "face", "coupon_rate", "periods", "yield_rate"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "bond_price"},
                    "face": {"$ref": "#/$defs/Expr"},
                    "coupon_rate": {"$ref": "#/$defs/Expr"},
                    "periods": {"type": "integer", "minimum": 0},
                    "yield_rate": {"$ref": "#/$defs/Expr"},
                    "decimal_places": {"type": "integer", "minimum": 0, "maximum": 128, "default": 12}
                }
            },
            "DiscountTable": {
                "type": "object",
                "required": ["intent", "rate", "periods"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "discount_table"},
                    "rate": {"$ref": "#/$defs/Expr"},
                    "periods": {"type": "integer", "minimum": 1, "maximum": MAX_PERIODS}
                }
            },
            "PriceValue": {
                "oneOf": [
                    {
                        "type": "string",
                        "pattern": "^-?[0-9]+(?:\\.[0-9]+)?$",
                        "maxLength": MAX_PRICE_DECIMAL_DIGITS + 2,
                        "description": "Exact fixed-point decimal text"
                    },
                    {"$ref": "#/$defs/Expr"}
                ]
            },
            "DecimalRounding": {
                "type": "string",
                "enum": ["half_even", "half_away_from_zero", "toward_zero"],
                "default": "half_even"
            },
            "DiscountedCashFlow": {
                "type": "object",
                "required": ["intent", "cash_flows", "discount_rate"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "discounted_cash_flow"},
                    "cash_flows": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": MAX_PERIODS,
                        "items": {"$ref": "#/$defs/PriceValue"},
                        "description": "Forecast cash flows for periods 1 through n"
                    },
                    "discount_rate": {"$ref": "#/$defs/PriceValue"},
                    "terminal_growth_rate": {"$ref": "#/$defs/PriceValue"},
                    "decimal_places": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 128,
                        "default": 12
                    },
                    "rounding_mode": {"$ref": "#/$defs/DecimalRounding"}
                }
            }
        }
    })
}

// ── scalar helpers (existing) ────────────────────────────────────────────────

fn future_value(
    present_value: &Rational,
    rate: &Rational,
    periods: u32,
) -> Result<Rational, String> {
    present_value
        .checked_mul(&growth_factor(rate, periods)?)
        .map_err(|e| e.to_string())
}

fn present_value(
    future_value: &Rational,
    rate: &Rational,
    periods: u32,
) -> Result<Rational, String> {
    future_value
        .checked_mul(&discount_factor(rate, periods)?)
        .map_err(|e| e.to_string())
}

fn discount_factor(rate: &Rational, period: u32) -> Result<Rational, String> {
    let growth = growth_factor(rate, period)?;
    Rational::one()
        .checked_div(&growth)
        .map_err(|_| "discount factor undefined when 1 + rate is zero".to_owned())
}

fn net_present_value(rate: &Rational, cash_flows: &[Expr]) -> Result<Rational, String> {
    if cash_flows.is_empty() {
        return Err("cash_flows must not be empty".to_owned());
    }
    let mut total = Rational::zero();
    for (period, cash_flow) in cash_flows.iter().enumerate() {
        let cash_flow = cash_flow.evaluate().map_err(prefix_cash_flow_error)?;
        let discounted = present_value(
            &cash_flow,
            rate,
            u32::try_from(period).map_err(|_| "too many cash_flow periods".to_owned())?,
        )?;
        total = total.checked_add(&discounted).map_err(|e| e.to_string())?;
    }
    Ok(total)
}

fn growth_factor(rate: &Rational, periods: u32) -> Result<Rational, String> {
    Rational::one()
        .checked_add(rate)
        .map_err(|e| e.to_string())?
        .checked_pow_i32(
            i32::try_from(periods)
                .map_err(|_| "periods exceed supported exponent range".to_owned())?,
        )
        .map_err(|e| e.to_string())
}

// ── IRR ──────────────────────────────────────────────────────────────────────

fn evaluate_irr(cash_flow_exprs: &[Expr], tolerance: f64) -> Result<f64, String> {
    if cash_flow_exprs.len() < 2 {
        return Err("irr requires at least two cash flows".to_owned());
    }
    if tolerance <= 0.0 {
        return Err("irr tolerance must be positive".to_owned());
    }
    let flows: Vec<f64> = cash_flow_exprs
        .iter()
        .map(|e| {
            e.evaluate()
                .map(|r| r.to_f64())
                .map_err(prefix_cash_flow_error)
        })
        .collect::<Result<_, _>>()?;
    compute_irr(&flows, tolerance)
}

#[mutants::skip]
fn compute_irr(flows: &[f64], tolerance: f64) -> Result<f64, String> {
    let npv = |r: f64| -> f64 {
        flows
            .iter()
            .enumerate()
            .map(|(t, &cf)| cf / (1.0 + r).powi(t as i32))
            .sum::<f64>()
    };

    let lo = -0.9999_f64;
    let hi = 100.0_f64;
    let mut f_lo = npv(lo);
    let f_hi = npv(hi);

    if f_lo * f_hi > 0.0 {
        return Err(
            "irr: NPV has the same sign at -99.99% and 10000%; no IRR in that range".to_owned(),
        );
    }

    let mut lo = lo;
    let mut hi = hi;
    for _ in 0..200 {
        let mid = (lo + hi) * 0.5;
        if hi - lo < tolerance {
            return Ok(mid);
        }
        let f_mid = npv(mid);
        if f_lo * f_mid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }
    Ok((lo + hi) * 0.5)
}

// ── amortization ─────────────────────────────────────────────────────────────

fn evaluate_amortize(
    principal_expr: &Expr,
    annual_rate_expr: &Expr,
    periods: u32,
    periods_per_year: u32,
) -> Result<(Rational, Vec<AmortizationRow>), String> {
    if periods == 0 {
        return Err("amortize: periods must be at least 1".to_owned());
    }
    if periods > MAX_PERIODS {
        return Err(format!("amortize: periods must not exceed {MAX_PERIODS}"));
    }
    if periods_per_year == 0 {
        return Err("amortize: periods_per_year must be at least 1".to_owned());
    }
    validate_expr_limits(principal_expr)?;
    validate_expr_limits(annual_rate_expr)?;

    let principal = principal_expr.evaluate().map_err(prefix_expr_error)?;
    let annual_rate = annual_rate_expr.evaluate().map_err(prefix_rate_error)?;
    if annual_rate.is_negative() {
        return Err("amortize: annual_rate must not be negative".to_owned());
    }

    let ppy = Rational::integer(i64::from(periods_per_year));
    let r = annual_rate.checked_div(&ppy).map_err(|e| e.to_string())?;
    amortize_schedule(&principal, &r, periods)
}

fn amortize_schedule(
    principal: &Rational,
    r: &Rational,
    periods: u32,
) -> Result<(Rational, Vec<AmortizationRow>), String> {
    let payment = if r.is_zero() {
        let n = Rational::integer(i64::from(periods));
        principal.checked_div(&n).map_err(|e| e.to_string())?
    } else {
        let df = discount_factor(r, periods)?;
        let one_minus_df = Rational::one()
            .checked_sub(&df)
            .map_err(|e| e.to_string())?;
        let pr = principal.checked_mul(r).map_err(|e| e.to_string())?;
        pr.checked_div(&one_minus_df).map_err(|e| e.to_string())?
    };

    let mut balance = principal.clone();
    let mut schedule = Vec::with_capacity(periods as usize);
    for period in 1..=periods {
        let interest = balance.checked_mul(r).map_err(|e| e.to_string())?;
        let principal_paid = payment.checked_sub(&interest).map_err(|e| e.to_string())?;
        balance = balance
            .checked_sub(&principal_paid)
            .map_err(|e| e.to_string())?;
        schedule.push(AmortizationRow {
            period,
            payment: exact_rational(&payment),
            interest: exact_rational(&interest),
            principal_paid: exact_rational(&principal_paid),
            balance: exact_rational(&balance),
        });
    }
    Ok((payment, schedule))
}

// ── bond price ───────────────────────────────────────────────────────────────

fn evaluate_bond_price(
    face_expr: &Expr,
    coupon_rate_expr: &Expr,
    periods: u32,
    yield_rate_expr: &Expr,
) -> Result<Rational, String> {
    validate_expr_limits(face_expr)?;
    validate_expr_limits(coupon_rate_expr)?;
    validate_expr_limits(yield_rate_expr)?;

    let face = face_expr.evaluate().map_err(prefix_expr_error)?;
    let coupon_rate = coupon_rate_expr.evaluate().map_err(prefix_rate_error)?;
    let yield_rate = yield_rate_expr.evaluate().map_err(prefix_rate_error)?;

    compute_bond_price(&face, &coupon_rate, periods, &yield_rate)
}

fn compute_bond_price(
    face: &Rational,
    coupon_rate: &Rational,
    periods: u32,
    yield_rate: &Rational,
) -> Result<Rational, String> {
    let coupon = face.checked_mul(coupon_rate).map_err(|e| e.to_string())?;
    let df_n = discount_factor(yield_rate, periods)?;
    let face_pv = face.checked_mul(&df_n).map_err(|e| e.to_string())?;

    let coupon_pv = if yield_rate.is_zero() {
        let n = Rational::integer(i64::from(periods));
        coupon.checked_mul(&n).map_err(|e| e.to_string())?
    } else {
        let one_minus_df = Rational::one()
            .checked_sub(&df_n)
            .map_err(|e| e.to_string())?;
        let annuity = one_minus_df
            .checked_div(yield_rate)
            .map_err(|e| e.to_string())?;
        coupon.checked_mul(&annuity).map_err(|e| e.to_string())?
    };

    coupon_pv.checked_add(&face_pv).map_err(|e| e.to_string())
}

// ── discounted cash-flow price model ────────────────────────────────────────

struct DiscountedCashFlowOutput {
    forecast_present_value: Rational,
    terminal_value: Option<Rational>,
    terminal_present_value: Option<Rational>,
    price: Rational,
}

fn evaluate_discounted_cash_flow(
    cash_flows: &[PriceValue],
    discount_rate: &PriceValue,
    terminal_growth_rate: Option<&PriceValue>,
    decimal_places: usize,
) -> Result<DiscountedCashFlowOutput, String> {
    validate_decimal_places(decimal_places)?;
    if cash_flows.is_empty() {
        return Err("discounted_cash_flow requires at least one forecast cash flow".to_owned());
    }
    if cash_flows.len() > MAX_PERIODS as usize {
        return Err(format!(
            "discounted_cash_flow cash flow count must not exceed {MAX_PERIODS}"
        ));
    }

    let discount_rate = evaluate_price_value(discount_rate, "discount_rate")?;
    let negative_one = Rational::integer(-1);
    if discount_rate <= negative_one {
        return Err("discount_rate must be greater than -1".to_owned());
    }

    let exact_cash_flows = cash_flows
        .iter()
        .enumerate()
        .map(|(index, value)| evaluate_price_value(value, &format!("cash_flows[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;

    let one_plus_discount_rate = Rational::one()
        .checked_add(&discount_rate)
        .map_err(|error| error.to_string())?;
    let mut discount_power = Rational::one();
    let mut forecast_present_value = Rational::zero();
    for cash_flow in &exact_cash_flows {
        discount_power = discount_power
            .checked_mul(&one_plus_discount_rate)
            .map_err(|error| error.to_string())?;
        let discounted = cash_flow
            .checked_div(&discount_power)
            .map_err(|error| error.to_string())?;
        forecast_present_value = forecast_present_value
            .checked_add(&discounted)
            .map_err(|error| error.to_string())?;
    }

    let (terminal_value, terminal_present_value) = match terminal_growth_rate {
        Some(growth_rate) => {
            let growth_rate = evaluate_price_value(growth_rate, "terminal_growth_rate")?;
            if growth_rate <= negative_one {
                return Err("terminal_growth_rate must be greater than -1".to_owned());
            }
            if growth_rate >= discount_rate {
                return Err(
                    "terminal_growth_rate must be less than discount_rate for a finite terminal value"
                        .to_owned(),
                );
            }

            let final_cash_flow = exact_cash_flows
                .last()
                .expect("non-empty cash flows were validated");
            let next_period_growth = Rational::one()
                .checked_add(&growth_rate)
                .map_err(|error| error.to_string())?;
            let next_period_cash_flow = final_cash_flow
                .checked_mul(&next_period_growth)
                .map_err(|error| error.to_string())?;
            let capitalization_rate = discount_rate
                .checked_sub(&growth_rate)
                .map_err(|error| error.to_string())?;
            let terminal_value = next_period_cash_flow
                .checked_div(&capitalization_rate)
                .map_err(|error| error.to_string())?;
            let terminal_present_value = terminal_value
                .checked_div(&discount_power)
                .map_err(|error| error.to_string())?;
            (Some(terminal_value), Some(terminal_present_value))
        }
        None => (None, None),
    };

    let price = match &terminal_present_value {
        Some(terminal) => forecast_present_value
            .checked_add(terminal)
            .map_err(|error| error.to_string())?,
        None => forecast_present_value.clone(),
    };

    Ok(DiscountedCashFlowOutput {
        forecast_present_value,
        terminal_value,
        terminal_present_value,
        price,
    })
}

fn evaluate_price_value(value: &PriceValue, role: &str) -> Result<Rational, String> {
    match value {
        PriceValue::Decimal(value) => {
            if value.len() > MAX_PRICE_DECIMAL_DIGITS + 2 {
                return Err(format!(
                    "{role} decimal text length must be <= {}",
                    MAX_PRICE_DECIMAL_DIGITS + 2
                ));
            }
            let digit_count = value.bytes().filter(u8::is_ascii_digit).count();
            if digit_count > MAX_PRICE_DECIMAL_DIGITS {
                return Err(format!(
                    "{role} decimal digit count must be <= {MAX_PRICE_DECIMAL_DIGITS}"
                ));
            }
            Rational::parse_decimal(value).map_err(|error| format!("{role}: {error}"))
        }
        PriceValue::Expression(expression) => {
            validate_expr_limits(expression)?;
            expression
                .evaluate()
                .map_err(|reason| format!("{role} expression error: {reason}"))
        }
    }
}

fn rounded_decimal_string(
    value: &Rational,
    decimal_places: usize,
    rounding_mode: DecimalRounding,
) -> String {
    let scale = BigInt::from(10u8).pow(decimal_places as u32);
    let scaled_numerator = value.numerator().abs() * &scale;
    let denominator = value.denominator();
    let mut units = &scaled_numerator / denominator;
    let remainder = scaled_numerator % denominator;
    let twice_remainder = &remainder * 2u8;

    let increment = match rounding_mode {
        DecimalRounding::TowardZero => false,
        DecimalRounding::HalfAwayFromZero => twice_remainder >= *denominator,
        DecimalRounding::HalfEven => match twice_remainder.cmp(denominator) {
            Ordering::Greater => true,
            Ordering::Equal => (&units % 2u8) == BigInt::one(),
            Ordering::Less => false,
        },
    };
    if increment {
        units += 1u8;
    }

    let negative = value.is_negative() && !units.is_zero();
    let sign = if negative { "-" } else { "" };
    let digits = units.to_string();
    if decimal_places == 0 {
        return format!("{sign}{digits}");
    }
    if digits.len() <= decimal_places {
        let zero_padding = "0".repeat(decimal_places - digits.len());
        return format!("{sign}0.{zero_padding}{digits}");
    }
    let split = digits.len() - decimal_places;
    format!("{sign}{}.{}", &digits[..split], &digits[split..])
}

// ── discount table ───────────────────────────────────────────────────────────

fn evaluate_discount_table(rate_expr: &Expr, periods: u32) -> Result<Vec<Rational>, String> {
    if periods == 0 {
        return Err("discount_table: periods must be at least 1".to_owned());
    }
    if periods > MAX_PERIODS {
        return Err(format!(
            "discount_table: periods must not exceed {MAX_PERIODS}"
        ));
    }
    validate_expr_limits(rate_expr)?;
    let rate = rate_expr.evaluate().map_err(prefix_rate_error)?;
    (1..=periods).map(|t| discount_factor(&rate, t)).collect()
}

// ── shared helpers ───────────────────────────────────────────────────────────

fn scalar_value_response(value: Rational, decimal_places: usize) -> FinanceResponse {
    FinanceResponse::Value {
        contract_version: CONTRACT_VERSION.to_owned(),
        exact: exact_rational(&value),
        decimal: value.decimal_string(decimal_places),
        checks: vec![
            FinanceCheck {
                name: "exact_rational_cashflow_math".to_owned(),
                passed: true,
            },
            FinanceCheck {
                name: "canonical_rational_form".to_owned(),
                passed: value.denominator() > &BigInt::zero(),
            },
        ],
    }
}

fn error_response(reason: String) -> FinanceResponse {
    FinanceResponse::Error {
        contract_version: CONTRACT_VERSION.to_owned(),
        code: classify_error(&reason),
        reason,
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

fn prefix_expr_error(reason: String) -> String {
    format!("finance expression error: {reason}")
}

fn prefix_rate_error(reason: String) -> String {
    format!("rate expression error: {reason}")
}

fn prefix_cash_flow_error(reason: String) -> String {
    format!("cash_flow expression error: {reason}")
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn integer(value: i32) -> Expr {
        Expr::Integer {
            value: value.to_string(),
        }
    }

    fn rational(numerator: i32, denominator: i32) -> Expr {
        Expr::Rational {
            numerator: numerator.to_string(),
            denominator: denominator.to_string(),
        }
    }

    fn decimal(value: &str) -> PriceValue {
        PriceValue::Decimal(value.to_owned())
    }

    fn r(n: i64, d: i64) -> Rational {
        Rational::new(n, d).unwrap()
    }

    fn exact(response: FinanceResponse) -> ExactRational {
        match response {
            FinanceResponse::Value { exact, .. } => exact,
            other => panic!("expected finance value response, got {other:?}"),
        }
    }

    // ── existing tests (unchanged) ───────────────────────────────────────────

    #[test]
    fn computes_future_and_present_value_exactly() {
        let future = exact(
            FinanceRequest::FutureValue {
                present_value: integer(100),
                rate: rational(1, 10),
                periods: 2,
                decimal_places: 2,
            }
            .evaluate(),
        );
        assert_eq!(future.display, "121");

        let present = exact(
            FinanceRequest::PresentValue {
                future_value: integer(121),
                rate: rational(1, 10),
                periods: 2,
                decimal_places: 2,
            }
            .evaluate(),
        );
        assert_eq!(present.display, "100");
    }

    #[test]
    fn computes_discount_factor_and_npv_exactly() {
        let discount = exact(
            FinanceRequest::DiscountFactor {
                rate: rational(1, 10),
                period: 2,
                decimal_places: 4,
            }
            .evaluate(),
        );
        assert_eq!(discount.display, "100/121");

        let npv = exact(
            FinanceRequest::NetPresentValue {
                rate: rational(1, 10),
                cash_flows: vec![integer(-100), integer(55), rational(121, 2)],
                decimal_places: 2,
            }
            .evaluate(),
        );
        assert_eq!(npv.display, "0");
    }

    #[test]
    fn default_decimal_places_are_stable() {
        let request: FinanceRequest = serde_json::from_str(
            r#"{
                "intent": "discount_factor",
                "rate": {"kind": "integer", "value": "0"},
                "period": 0
            }"#,
        )
        .unwrap();

        match request.evaluate() {
            FinanceResponse::Value { decimal, .. } => {
                assert_eq!(decimal, "1.000000000000");
            }
            other => panic!("expected finance value response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_finance_inputs() {
        let empty_npv = FinanceRequest::NetPresentValue {
            rate: integer(0),
            cash_flows: vec![],
            decimal_places: 2,
        }
        .evaluate();
        assert!(matches!(
            empty_npv,
            FinanceResponse::Error { reason, .. } if reason == "cash_flows must not be empty"
        ));

        let undefined_discount = FinanceRequest::DiscountFactor {
            rate: integer(-1),
            period: 1,
            decimal_places: 2,
        }
        .evaluate();
        assert!(matches!(
            undefined_discount,
            FinanceResponse::Error { reason, .. }
                if reason == "discount factor undefined when 1 + rate is zero"
        ));
    }

    #[test]
    fn prefixes_expression_errors_by_role() {
        let present_value_error = FinanceRequest::FutureValue {
            present_value: Expr::Symbol {
                name: "pv".to_owned(),
            },
            rate: integer(0),
            periods: 1,
            decimal_places: 2,
        }
        .evaluate();
        assert!(matches!(
            present_value_error,
            FinanceResponse::Error { reason, .. }
                if reason == "finance expression error: unbound symbol `pv`"
        ));

        let future_value_error = FinanceRequest::PresentValue {
            future_value: Expr::Symbol {
                name: "fv".to_owned(),
            },
            rate: integer(0),
            periods: 1,
            decimal_places: 2,
        }
        .evaluate();
        assert!(matches!(
            future_value_error,
            FinanceResponse::Error { reason, .. }
                if reason == "finance expression error: unbound symbol `fv`"
        ));

        let rate_error = FinanceRequest::DiscountFactor {
            rate: Expr::Symbol {
                name: "r".to_owned(),
            },
            period: 1,
            decimal_places: 2,
        }
        .evaluate();
        assert!(matches!(
            rate_error,
            FinanceResponse::Error { reason, .. } if reason == "rate expression error: unbound symbol `r`"
        ));

        let cash_flow_error = FinanceRequest::NetPresentValue {
            rate: integer(0),
            cash_flows: vec![Expr::Symbol {
                name: "cf".to_owned(),
            }],
            decimal_places: 2,
        }
        .evaluate();
        assert!(matches!(
            cash_flow_error,
            FinanceResponse::Error { reason, .. }
                if reason == "cash_flow expression error: unbound symbol `cf`"
        ));
    }

    // ── IRR ──────────────────────────────────────────────────────────────────

    #[test]
    fn irr_rejects_empty_cash_flows() {
        // len=0: evaluate_irr must error. Mutation < → > at line 440 would allow
        // empty flows through to compute_irr([]) which returns Ok (NPV always 0).
        let resp = FinanceRequest::Irr {
            cash_flows: vec![],
            tolerance: 1e-6,
        }
        .evaluate();
        assert!(matches!(resp, FinanceResponse::Error { .. }));
    }

    #[test]
    fn irr_accepts_when_hi_bracket_npv_is_zero() {
        // flows [-1, 101]: npv(100) = -1 + 101/101 = 0.0 exactly.
        // f_lo * f_hi = positive * 0 = 0; original (> 0) proceeds, mutation (>= 0) errors.
        // f_lo / f_hi = +inf; original proceeds, mutation (/ then >) also errors.
        let irr = compute_irr(&[-1.0_f64, 101.0], 1e-8).unwrap();
        assert!(
            (irr - 100.0).abs() < 1e-4,
            "IRR of [-1,101] should be 100%, got {irr}"
        );
    }

    #[test]
    fn irr_exits_early_when_tolerance_exceeds_bracket() {
        // tolerance=200 > bracket width (~101), so first mid ≈ 49.5 is returned immediately.
        // Mutation < → == at line 475: 100.9999 == 200 is never true, runs 200 iters → returns ~0.1.
        let flows = vec![-100.0_f64, 110.0];
        let irr = compute_irr(&flows, 200.0).unwrap();
        // First midpoint: (-0.9999 + 100) * 0.5 ≈ 49.5, not 0.1
        assert!(
            (irr - 49.5).abs() < 1.0,
            "should return first midpoint ~49.5, got {irr}"
        );
    }

    #[test]
    fn irr_fallthrough_after_200_iterations() {
        // tolerance=1e-70 < bracket/2^200 ≈ 6e-59, so all 200 iterations run and
        // the final Ok((lo+hi)*0.5) line executes. Mutations there (+, /, -, *) all
        // produce values far from the true IRR of 0.1.
        let flows = vec![-100.0_f64, 110.0];
        let irr = compute_irr(&flows, 1e-70).unwrap();
        assert!(
            (irr - 0.1).abs() < 1e-4,
            "200-iter fallthrough should still converge, got {irr}"
        );
    }

    #[test]
    fn irr_converges_to_zero_npv() {
        // [-100, 110]: IRR = 10% exactly
        let flows = vec![-100.0_f64, 110.0];
        let irr = compute_irr(&flows, 1e-8).unwrap();
        let npv: f64 = flows
            .iter()
            .enumerate()
            .map(|(t, &cf)| cf / (1.0 + irr).powi(t as i32))
            .sum();
        assert!(npv.abs() < 1e-4, "NPV at IRR must be near 0, got {npv}");
        assert!((irr - 0.1).abs() < 1e-6, "IRR must be near 0.1, got {irr}");
    }

    #[test]
    fn irr_multi_cashflow_npv_near_zero() {
        // From issue: [-100000, 30000, 40000, 50000]
        let flows = vec![-100_000.0_f64, 30_000.0, 40_000.0, 50_000.0];
        let irr = compute_irr(&flows, 1e-6).unwrap();
        let npv: f64 = flows
            .iter()
            .enumerate()
            .map(|(t, &cf)| cf / (1.0 + irr).powi(t as i32))
            .sum();
        assert!(npv.abs() < 1.0, "NPV at IRR must be near 0, got {npv}");
        assert!(
            irr > 0.0 && irr < 1.0,
            "IRR should be a moderate positive rate"
        );
    }

    #[test]
    fn irr_no_sign_change_returns_error() {
        // All positive cash flows: NPV is positive for all r in [-99.99%, 10000%]
        let flows = vec![100.0_f64, 100.0, 100.0];
        assert!(compute_irr(&flows, 1e-6).is_err());
    }

    #[test]
    fn irr_bisection_bracket_is_correct() {
        // [-100, 150]: IRR = 50%. Verify NPV changes sign at lo and hi.
        let flows = vec![-100.0_f64, 150.0];
        let lo_npv: f64 = flows
            .iter()
            .enumerate()
            .map(|(t, &c)| c / (1.0 + (-0.9999_f64)).powi(t as i32))
            .sum();
        let hi_npv: f64 = flows
            .iter()
            .enumerate()
            .map(|(t, &c)| c / (1.0 + 100.0_f64).powi(t as i32))
            .sum();
        assert!(lo_npv * hi_npv < 0.0, "bracket must have opposite signs");
        let irr = compute_irr(&flows, 1e-8).unwrap();
        assert!((irr - 0.5).abs() < 1e-6, "IRR should be 0.5, got {irr}");
    }

    #[test]
    fn irr_request_returns_irr_response() {
        let resp = FinanceRequest::Irr {
            cash_flows: vec![integer(-100), integer(110)],
            tolerance: 1e-8,
        }
        .evaluate();
        match resp {
            FinanceResponse::Irr {
                approximate_f64, ..
            } => {
                assert!((approximate_f64 - 0.1).abs() < 1e-6);
            }
            other => panic!("expected Irr response, got {other:?}"),
        }
    }

    // ── amortization ─────────────────────────────────────────────────────────

    #[test]
    fn amortize_two_periods_exact_balance_zero() {
        // P=100, r=50%/yr, 2 periods, ppy=1 → r=50%, PMT=90
        // Period 1: interest=50, principal=40, balance=60
        // Period 2: interest=30, principal=60, balance=0
        let (payment, schedule) = amortize_schedule(&r(100, 1), &r(1, 2), 2).unwrap();
        assert_eq!(payment, r(90, 1));
        assert_eq!(schedule.len(), 2);
        assert_eq!(schedule[0].period, 1);
        assert_eq!(schedule[0].interest.display, "50");
        assert_eq!(schedule[0].principal_paid.display, "40");
        assert_eq!(schedule[0].balance.display, "60");
        assert_eq!(schedule[1].interest.display, "30");
        assert_eq!(schedule[1].principal_paid.display, "60");
        assert_eq!(schedule[1].balance.display, "0");
    }

    #[test]
    fn amortize_zero_rate_equal_principal_payments() {
        // P=90, r=0%, 3 periods → PMT=30, equal principal splits
        let (payment, schedule) = amortize_schedule(&r(90, 1), &r(0, 1), 3).unwrap();
        assert_eq!(payment, r(30, 1));
        assert_eq!(schedule[0].interest.display, "0");
        assert_eq!(schedule[0].principal_paid.display, "30");
        assert_eq!(schedule[2].balance.display, "0");
    }

    #[test]
    fn amortize_balance_is_exactly_zero_after_all_periods() {
        // Use a 4-period loan at 25% annual, ppy=1 → r=25%
        // Verify exact rational balance reaches 0
        let (_, schedule) = amortize_schedule(&r(1000, 1), &r(1, 4), 4).unwrap();
        assert_eq!(schedule.last().unwrap().balance.display, "0");
    }

    #[test]
    fn amortize_request_returns_schedule() {
        let resp = FinanceRequest::Amortize {
            principal: integer(100),
            annual_rate: rational(1, 2),
            periods: 2,
            periods_per_year: 1,
        }
        .evaluate();
        match resp {
            FinanceResponse::Schedule {
                payment, schedule, ..
            } => {
                assert_eq!(payment.display, "90");
                assert_eq!(schedule.len(), 2);
                assert_eq!(schedule.last().unwrap().balance.display, "0");
            }
            other => panic!("expected Schedule response, got {other:?}"),
        }
    }

    #[test]
    fn amortize_rejects_invalid_inputs() {
        let bad_ppy = FinanceRequest::Amortize {
            principal: integer(100),
            annual_rate: rational(1, 10),
            periods: 12,
            periods_per_year: 0,
        }
        .evaluate();
        assert!(matches!(bad_ppy, FinanceResponse::Error { .. }));

        let negative_rate = FinanceRequest::Amortize {
            principal: integer(100),
            annual_rate: integer(-1),
            periods: 12,
            periods_per_year: 12,
        }
        .evaluate();
        assert!(matches!(negative_rate, FinanceResponse::Error { .. }));
    }

    #[test]
    fn amortize_at_max_periods_succeeds() {
        // periods = MAX_PERIODS (1200) must succeed; mutation > → >= would reject it.
        let resp = FinanceRequest::Amortize {
            principal: integer(1200),
            annual_rate: integer(0),
            periods: 1200,
            periods_per_year: 1,
        }
        .evaluate();
        assert!(matches!(resp, FinanceResponse::Schedule { .. }));
    }

    #[test]
    fn amortize_exceeds_max_periods_fails() {
        // periods = MAX_PERIODS+1 (1201) must error; mutation > → == would allow it through.
        let resp = FinanceRequest::Amortize {
            principal: integer(1200),
            annual_rate: integer(0),
            periods: 1201,
            periods_per_year: 1,
        }
        .evaluate();
        assert!(matches!(resp, FinanceResponse::Error { .. }));
    }

    // ── bond price ────────────────────────────────────────────────────────────

    #[test]
    fn bond_price_at_par_when_coupon_equals_yield() {
        // Face=1000, coupon=7%, yield=7%, n=10 → price=1000
        let price = compute_bond_price(&r(1000, 1), &r(7, 100), 10, &r(7, 100)).unwrap();
        assert_eq!(price, r(1000, 1));
    }

    #[test]
    fn bond_price_premium_when_coupon_exceeds_yield() {
        // Face=1000, coupon=8%, yield=6%, n=1 → price = (80+1000)/1.06 = 1080/1.06
        let price = compute_bond_price(&r(1000, 1), &r(8, 100), 1, &r(6, 100)).unwrap();
        // coupon = 80, face_pv = 1000*(100/106) = 100000/106, coupon_pv = 80*(100/106) = 8000/106
        // total = 108000/106 = 54000/53
        assert_eq!(price, r(54000, 53));
    }

    #[test]
    fn bond_price_zero_yield_sums_all_coupons_plus_face() {
        // Face=1000, coupon=6%, yield=0%, n=3 → price = 60+60+60+1000 = 1180
        let price = compute_bond_price(&r(1000, 1), &r(6, 100), 3, &r(0, 1)).unwrap();
        assert_eq!(price, r(1180, 1));
    }

    #[test]
    fn bond_price_request_returns_value() {
        let resp = FinanceRequest::BondPrice {
            face: integer(1000),
            coupon_rate: rational(7, 100),
            periods: 10,
            yield_rate: rational(7, 100),
            decimal_places: 2,
        }
        .evaluate();
        let ex = exact(resp);
        assert_eq!(ex.display, "1000");
    }

    // ── discounted cash-flow price model ────────────────────────────────────

    #[test]
    fn discounted_cash_flow_uses_one_based_forecast_periods() {
        let response = FinanceRequest::DiscountedCashFlow {
            cash_flows: vec![decimal("110.00")],
            discount_rate: decimal("0.10"),
            terminal_growth_rate: None,
            decimal_places: 2,
            rounding_mode: DecimalRounding::HalfEven,
        }
        .evaluate();

        match response {
            FinanceResponse::PriceModel { model, .. } => {
                assert_eq!(model.forecast_present_value.display, "100");
                assert_eq!(model.terminal_value, None);
                assert_eq!(model.price.display, "100");
                assert_eq!(model.decimal, "100.00");
            }
            other => panic!("expected price model response, got {other:?}"),
        }
    }

    #[test]
    fn discounted_cash_flow_terminal_value_is_exact() {
        let response = FinanceRequest::DiscountedCashFlow {
            cash_flows: vec![decimal("110")],
            discount_rate: decimal("0.10"),
            terminal_growth_rate: Some(decimal("0.00")),
            decimal_places: 2,
            rounding_mode: DecimalRounding::HalfEven,
        }
        .evaluate();

        match response {
            FinanceResponse::PriceModel { model, .. } => {
                assert_eq!(model.forecast_present_value.display, "100");
                assert_eq!(model.terminal_value.unwrap().display, "1100");
                assert_eq!(model.terminal_present_value.unwrap().display, "1000");
                assert_eq!(model.price.display, "1100");
                assert_eq!(model.decimal, "1100.00");
            }
            other => panic!("expected price model response, got {other:?}"),
        }
    }

    #[test]
    fn discounted_cash_flow_rejects_non_finite_terminal_model() {
        let response = FinanceRequest::DiscountedCashFlow {
            cash_flows: vec![decimal("100")],
            discount_rate: decimal("0.08"),
            terminal_growth_rate: Some(decimal("0.08")),
            decimal_places: 2,
            rounding_mode: DecimalRounding::HalfEven,
        }
        .evaluate();

        assert!(matches!(
            response,
            FinanceResponse::Error { reason, .. }
                if reason.contains("must be less than discount_rate")
        ));
    }

    #[test]
    fn discounted_cash_flow_rejects_invalid_decimal_and_discount_boundary() {
        let invalid_decimal = FinanceRequest::DiscountedCashFlow {
            cash_flows: vec![decimal("1e3")],
            discount_rate: decimal("0.08"),
            terminal_growth_rate: None,
            decimal_places: 2,
            rounding_mode: DecimalRounding::HalfEven,
        }
        .evaluate();
        assert!(matches!(
            invalid_decimal,
            FinanceResponse::Error { reason, .. } if reason.contains("invalid decimal")
        ));

        let invalid_rate = FinanceRequest::DiscountedCashFlow {
            cash_flows: vec![decimal("100")],
            discount_rate: decimal("-1.00"),
            terminal_growth_rate: None,
            decimal_places: 2,
            rounding_mode: DecimalRounding::HalfEven,
        }
        .evaluate();
        assert!(matches!(
            invalid_rate,
            FinanceResponse::Error { reason, .. } if reason == "discount_rate must be greater than -1"
        ));
    }

    #[test]
    fn monetary_rounding_modes_handle_positive_and_negative_ties() {
        let positive = Rational::parse_decimal("1.005").unwrap();
        let next_odd = Rational::parse_decimal("1.015").unwrap();
        let negative = Rational::parse_decimal("-1.005").unwrap();

        assert_eq!(
            rounded_decimal_string(&positive, 2, DecimalRounding::HalfEven),
            "1.00"
        );
        assert_eq!(
            rounded_decimal_string(&next_odd, 2, DecimalRounding::HalfEven),
            "1.02"
        );
        assert_eq!(
            rounded_decimal_string(&negative, 2, DecimalRounding::HalfEven),
            "-1.00"
        );
        assert_eq!(
            rounded_decimal_string(&positive, 2, DecimalRounding::HalfAwayFromZero),
            "1.01"
        );
        assert_eq!(
            rounded_decimal_string(&negative, 2, DecimalRounding::HalfAwayFromZero),
            "-1.01"
        );
        assert_eq!(
            rounded_decimal_string(&next_odd, 2, DecimalRounding::TowardZero),
            "1.01"
        );
    }

    // ── discount table ───────────────────────────────────────────────────────

    #[test]
    fn discount_table_length_and_exact_values() {
        // rate=10%=1/10, n=3 → [10/11, 100/121, 1000/1331]
        let factors = evaluate_discount_table(&rational(1, 10), 3).unwrap();
        assert_eq!(factors.len(), 3);
        assert_eq!(factors[0], r(10, 11));
        assert_eq!(factors[1], r(100, 121));
        assert_eq!(factors[2], r(1000, 1331));
    }

    #[test]
    fn discount_table_zero_rate_all_ones() {
        // rate=0 → all factors = 1
        let factors = evaluate_discount_table(&integer(0), 4).unwrap();
        assert_eq!(factors.len(), 4);
        assert!(factors.iter().all(|f| *f == r(1, 1)));
    }

    #[test]
    fn discount_table_request_returns_table() {
        let resp = FinanceRequest::DiscountTable {
            rate: rational(1, 10),
            periods: 2,
        }
        .evaluate();
        match resp {
            FinanceResponse::Table { factors, .. } => {
                assert_eq!(factors.len(), 2);
                assert_eq!(factors[0].display, "10/11");
                assert_eq!(factors[1].display, "100/121");
            }
            other => panic!("expected Table response, got {other:?}"),
        }
    }

    #[test]
    fn discount_table_rejects_zero_periods() {
        let resp = FinanceRequest::DiscountTable {
            rate: rational(1, 10),
            periods: 0,
        }
        .evaluate();
        assert!(matches!(resp, FinanceResponse::Error { .. }));
    }

    #[test]
    fn discount_table_at_max_periods_succeeds() {
        // periods = MAX_PERIODS must succeed; mutation > → >= would reject it.
        let resp = FinanceRequest::DiscountTable {
            rate: integer(0),
            periods: 1200,
        }
        .evaluate();
        assert!(matches!(resp, FinanceResponse::Table { .. }));
    }

    #[test]
    fn discount_table_exceeds_max_periods_fails() {
        // periods = MAX_PERIODS+1 must error; mutation > → == would allow it.
        let resp = FinanceRequest::DiscountTable {
            rate: integer(0),
            periods: 1201,
        }
        .evaluate();
        assert!(matches!(resp, FinanceResponse::Error { .. }));
    }
}
