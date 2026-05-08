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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FinanceResponse {
    Value {
        contract_version: String,
        exact: ExactRational,
        decimal: String,
        checks: Vec<FinanceCheck>,
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
        match self.evaluate_inner() {
            Ok((value, decimal_places)) => FinanceResponse::Value {
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
            },
            Err(reason) => FinanceResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<(Rational, usize), String> {
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
            {"$ref": "#/$defs/NetPresentValue"}
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
            }
        }
    })
}

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

    fn exact(response: FinanceResponse) -> ExactRational {
        match response {
            FinanceResponse::Value { exact, .. } => exact,
            FinanceResponse::Error { reason, .. } => {
                panic!("expected finance value response: {reason}")
            }
        }
    }

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
}
