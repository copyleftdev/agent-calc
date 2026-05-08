use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uom::si::f64::{Length, Mass, Time, Velocity};
use uom::si::length::{centimeter, foot, inch, kilometer, meter, mile, millimeter};
use uom::si::mass::{gram, kilogram, pound};
use uom::si::time::{hour, minute, second};
use uom::si::velocity::{foot_per_second, kilometer_per_hour, meter_per_second, mile_per_hour};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum UnitRequest {
    Convert {
        quantity: Quantity,
        to: String,
    },
    Add {
        left: Quantity,
        right: Quantity,
        to: String,
    },
    Sub {
        left: Quantity,
        right: Quantity,
        to: String,
    },
    Divide {
        left: Quantity,
        right: Quantity,
        to: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quantity {
    pub dimension: Dimension,
    pub value: f64,
    pub unit: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Length,
    Mass,
    Time,
    Velocity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UnitResponse {
    Solved {
        contract_version: String,
        dimension: Dimension,
        value: f64,
        unit: String,
        exactness: Exactness,
        checks: Vec<UnitCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnitCheck {
    pub name: String,
    pub passed: bool,
}

impl UnitRequest {
    pub fn evaluate(&self) -> UnitResponse {
        match self.evaluate_inner() {
            Ok((dimension, value, unit)) => UnitResponse::Solved {
                contract_version: CONTRACT_VERSION.to_owned(),
                dimension,
                value,
                unit,
                exactness: Exactness::ApproximateF64,
                checks: vec![UnitCheck {
                    name: "dimension_checked_by_uom".to_owned(),
                    passed: true,
                }],
            },
            Err(reason) => UnitResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<(Dimension, f64, String), String> {
        match self {
            UnitRequest::Convert { quantity, to } => convert(quantity, to),
            UnitRequest::Add { left, right, to } => {
                require_same_dimension(left, right)?;
                match left.dimension {
                    Dimension::Length => {
                        let value = parse_length(left)? + parse_length(right)?;
                        Ok((Dimension::Length, get_length(value, to)?, to.clone()))
                    }
                    Dimension::Mass => {
                        let value = parse_mass(left)? + parse_mass(right)?;
                        Ok((Dimension::Mass, get_mass(value, to)?, to.clone()))
                    }
                    Dimension::Time => {
                        let value = parse_time(left)? + parse_time(right)?;
                        Ok((Dimension::Time, get_time(value, to)?, to.clone()))
                    }
                    Dimension::Velocity => {
                        let value = parse_velocity(left)? + parse_velocity(right)?;
                        Ok((Dimension::Velocity, get_velocity(value, to)?, to.clone()))
                    }
                }
            }
            UnitRequest::Sub { left, right, to } => {
                require_same_dimension(left, right)?;
                match left.dimension {
                    Dimension::Length => {
                        let value = parse_length(left)? - parse_length(right)?;
                        Ok((Dimension::Length, get_length(value, to)?, to.clone()))
                    }
                    Dimension::Mass => {
                        let value = parse_mass(left)? - parse_mass(right)?;
                        Ok((Dimension::Mass, get_mass(value, to)?, to.clone()))
                    }
                    Dimension::Time => {
                        let value = parse_time(left)? - parse_time(right)?;
                        Ok((Dimension::Time, get_time(value, to)?, to.clone()))
                    }
                    Dimension::Velocity => {
                        let value = parse_velocity(left)? - parse_velocity(right)?;
                        Ok((Dimension::Velocity, get_velocity(value, to)?, to.clone()))
                    }
                }
            }
            UnitRequest::Divide { left, right, to } => {
                if left.dimension != Dimension::Length || right.dimension != Dimension::Time {
                    return Err(
                        "divide currently supports length divided by time -> velocity".to_owned(),
                    );
                }
                let value = parse_length(left)? / parse_time(right)?;
                Ok((Dimension::Velocity, get_velocity(value, to)?, to.clone()))
            }
        }
    }
}

pub fn units_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/units.json"),
        "title": "agent-calc calc1 units request",
        "description": "Typed unit conversion and dimension-checked quantity arithmetic request.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/Convert"},
            {"$ref": "#/$defs/BinaryQuantity"}
        ],
        "$defs": {
            "Dimension": {
                "type": "string",
                "enum": ["length", "mass", "time", "velocity"]
            },
            "Quantity": {
                "type": "object",
                "required": ["dimension", "value", "unit"],
                "additionalProperties": false,
                "properties": {
                    "dimension": {"$ref": "#/$defs/Dimension"},
                    "value": {"type": "number"},
                    "unit": {"type": "string"}
                }
            },
            "Convert": {
                "type": "object",
                "required": ["intent", "quantity", "to"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "convert"},
                    "quantity": {"$ref": "#/$defs/Quantity"},
                    "to": {"type": "string"}
                }
            },
            "BinaryQuantity": {
                "type": "object",
                "required": ["intent", "left", "right", "to"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["add", "sub", "divide"]},
                    "left": {"$ref": "#/$defs/Quantity"},
                    "right": {"$ref": "#/$defs/Quantity"},
                    "to": {"type": "string"}
                }
            }
        }
    })
}

fn convert(quantity: &Quantity, to: &str) -> Result<(Dimension, f64, String), String> {
    match quantity.dimension {
        Dimension::Length => Ok((
            Dimension::Length,
            get_length(parse_length(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Mass => Ok((
            Dimension::Mass,
            get_mass(parse_mass(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Time => Ok((
            Dimension::Time,
            get_time(parse_time(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Velocity => Ok((
            Dimension::Velocity,
            get_velocity(parse_velocity(quantity)?, to)?,
            to.to_owned(),
        )),
    }
}

fn require_same_dimension(left: &Quantity, right: &Quantity) -> Result<(), String> {
    if left.dimension == right.dimension {
        Ok(())
    } else {
        Err(format!(
            "dimension mismatch: left is {:?}, right is {:?}",
            left.dimension, right.dimension
        ))
    }
}

fn parse_length(quantity: &Quantity) -> Result<Length, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "m" => Ok(Length::new::<meter>(quantity.value)),
        "km" => Ok(Length::new::<kilometer>(quantity.value)),
        "cm" => Ok(Length::new::<centimeter>(quantity.value)),
        "mm" => Ok(Length::new::<millimeter>(quantity.value)),
        "in" => Ok(Length::new::<inch>(quantity.value)),
        "ft" => Ok(Length::new::<foot>(quantity.value)),
        "mi" => Ok(Length::new::<mile>(quantity.value)),
        unit => Err(format!("unsupported length unit `{unit}`")),
    }
}

fn get_length(value: Length, unit: &str) -> Result<f64, String> {
    match unit {
        "m" => Ok(value.get::<meter>()),
        "km" => Ok(value.get::<kilometer>()),
        "cm" => Ok(value.get::<centimeter>()),
        "mm" => Ok(value.get::<millimeter>()),
        "in" => Ok(value.get::<inch>()),
        "ft" => Ok(value.get::<foot>()),
        "mi" => Ok(value.get::<mile>()),
        unit => Err(format!("unsupported length unit `{unit}`")),
    }
}

fn parse_mass(quantity: &Quantity) -> Result<Mass, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "kg" => Ok(Mass::new::<kilogram>(quantity.value)),
        "g" => Ok(Mass::new::<gram>(quantity.value)),
        "lb" => Ok(Mass::new::<pound>(quantity.value)),
        unit => Err(format!("unsupported mass unit `{unit}`")),
    }
}

fn get_mass(value: Mass, unit: &str) -> Result<f64, String> {
    match unit {
        "kg" => Ok(value.get::<kilogram>()),
        "g" => Ok(value.get::<gram>()),
        "lb" => Ok(value.get::<pound>()),
        unit => Err(format!("unsupported mass unit `{unit}`")),
    }
}

fn parse_time(quantity: &Quantity) -> Result<Time, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "s" => Ok(Time::new::<second>(quantity.value)),
        "min" => Ok(Time::new::<minute>(quantity.value)),
        "h" => Ok(Time::new::<hour>(quantity.value)),
        unit => Err(format!("unsupported time unit `{unit}`")),
    }
}

fn get_time(value: Time, unit: &str) -> Result<f64, String> {
    match unit {
        "s" => Ok(value.get::<second>()),
        "min" => Ok(value.get::<minute>()),
        "h" => Ok(value.get::<hour>()),
        unit => Err(format!("unsupported time unit `{unit}`")),
    }
}

fn parse_velocity(quantity: &Quantity) -> Result<Velocity, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "m/s" => Ok(Velocity::new::<meter_per_second>(quantity.value)),
        "km/h" => Ok(Velocity::new::<kilometer_per_hour>(quantity.value)),
        "mph" => Ok(Velocity::new::<mile_per_hour>(quantity.value)),
        "ft/s" => Ok(Velocity::new::<foot_per_second>(quantity.value)),
        unit => Err(format!("unsupported velocity unit `{unit}`")),
    }
}

fn get_velocity(value: Velocity, unit: &str) -> Result<f64, String> {
    match unit {
        "m/s" => Ok(value.get::<meter_per_second>()),
        "km/h" => Ok(value.get::<kilometer_per_hour>()),
        "mph" => Ok(value.get::<mile_per_hour>()),
        "ft/s" => Ok(value.get::<foot_per_second>()),
        unit => Err(format!("unsupported velocity unit `{unit}`")),
    }
}

fn ensure_finite(value: f64) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err("quantity value must be finite".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(dimension: Dimension, value: f64, unit: &str) -> Quantity {
        Quantity {
            dimension,
            value,
            unit: unit.to_owned(),
        }
    }

    #[test]
    fn converts_length() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Length, 1.5, "km"),
            to: "m".to_owned(),
        }
        .evaluate();

        match response {
            UnitResponse::Solved {
                value,
                unit,
                exactness,
                ..
            } => {
                assert_eq!(value, 1500.0);
                assert_eq!(unit, "m");
                assert_eq!(exactness, Exactness::ApproximateF64);
            }
            other => panic!("expected solved response, got {other:?}"),
        }
    }

    #[test]
    fn divides_length_by_time_to_velocity() {
        let response = UnitRequest::Divide {
            left: q(Dimension::Length, 100.0, "m"),
            right: q(Dimension::Time, 9.58, "s"),
            to: "m/s".to_owned(),
        }
        .evaluate();

        match response {
            UnitResponse::Solved {
                value, dimension, ..
            } => {
                assert_eq!(dimension, Dimension::Velocity);
                assert!((value - 10.438413361169102).abs() < 1e-12);
            }
            other => panic!("expected solved response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_dimension_mismatch() {
        let response = UnitRequest::Add {
            left: q(Dimension::Length, 1.0, "m"),
            right: q(Dimension::Time, 1.0, "s"),
            to: "m".to_owned(),
        }
        .evaluate();

        assert!(matches!(
            response,
            UnitResponse::Error { reason, .. } if reason.contains("dimension mismatch")
        ));
    }

    #[test]
    fn covers_all_addition_branches() {
        let cases = [
            (
                UnitRequest::Add {
                    left: q(Dimension::Length, 2.0, "m"),
                    right: q(Dimension::Length, 50.0, "cm"),
                    to: "m".to_owned(),
                },
                2.5,
            ),
            (
                UnitRequest::Add {
                    left: q(Dimension::Mass, 1.0, "kg"),
                    right: q(Dimension::Mass, 500.0, "g"),
                    to: "kg".to_owned(),
                },
                1.5,
            ),
            (
                UnitRequest::Add {
                    left: q(Dimension::Time, 1.0, "h"),
                    right: q(Dimension::Time, 30.0, "min"),
                    to: "min".to_owned(),
                },
                90.0,
            ),
            (
                UnitRequest::Add {
                    left: q(Dimension::Velocity, 10.0, "m/s"),
                    right: q(Dimension::Velocity, 36.0, "km/h"),
                    to: "m/s".to_owned(),
                },
                20.0,
            ),
        ];

        for (request, expected) in cases {
            match request.evaluate() {
                UnitResponse::Solved { value, .. } => {
                    assert!((value - expected).abs() < 1e-12);
                }
                other => panic!("expected solved response, got {other:?}"),
            }
        }
    }

    #[test]
    fn covers_all_subtraction_branches() {
        let cases = [
            (
                UnitRequest::Sub {
                    left: q(Dimension::Length, 2.0, "m"),
                    right: q(Dimension::Length, 50.0, "cm"),
                    to: "m".to_owned(),
                },
                1.5,
            ),
            (
                UnitRequest::Sub {
                    left: q(Dimension::Mass, 1.0, "kg"),
                    right: q(Dimension::Mass, 500.0, "g"),
                    to: "g".to_owned(),
                },
                500.0,
            ),
            (
                UnitRequest::Sub {
                    left: q(Dimension::Time, 1.0, "h"),
                    right: q(Dimension::Time, 30.0, "min"),
                    to: "min".to_owned(),
                },
                30.0,
            ),
            (
                UnitRequest::Sub {
                    left: q(Dimension::Velocity, 20.0, "m/s"),
                    right: q(Dimension::Velocity, 36.0, "km/h"),
                    to: "m/s".to_owned(),
                },
                10.0,
            ),
        ];

        for (request, expected) in cases {
            match request.evaluate() {
                UnitResponse::Solved { value, .. } => {
                    assert!((value - expected).abs() < 1e-12);
                }
                other => panic!("expected solved response, got {other:?}"),
            }
        }
    }

    #[test]
    fn converts_mass_and_time() {
        let mass = UnitRequest::Convert {
            quantity: q(Dimension::Mass, 2.0, "kg"),
            to: "g".to_owned(),
        }
        .evaluate();
        let time = UnitRequest::Convert {
            quantity: q(Dimension::Time, 2.0, "h"),
            to: "min".to_owned(),
        }
        .evaluate();

        assert!(matches!(mass, UnitResponse::Solved { value, .. } if value == 2000.0));
        assert!(matches!(time, UnitResponse::Solved { value, .. } if value == 120.0));
    }

    #[test]
    fn divide_rejects_non_length_over_time_with_specific_error() {
        let response = UnitRequest::Divide {
            left: q(Dimension::Mass, 1.0, "kg"),
            right: q(Dimension::Time, 1.0, "s"),
            to: "m/s".to_owned(),
        }
        .evaluate();

        assert!(matches!(
            response,
            UnitResponse::Error { reason, .. }
                if reason == "divide currently supports length divided by time -> velocity"
        ));
    }

    #[test]
    fn rejects_non_finite_values() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Length, f64::NAN, "m"),
            to: "cm".to_owned(),
        }
        .evaluate();

        assert!(matches!(
            response,
            UnitResponse::Error { reason, .. } if reason == "quantity value must be finite"
        ));
    }
}
