use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uom::si::area::{acre, hectare, square_foot, square_meter};
use uom::si::energy::{btu, calorie, joule, kilojoule, kilowatt_hour};
use uom::si::f64::{
    Area, Energy, Force, Frequency, Length, Mass, Power, Pressure, Time, Velocity, Volume,
};
use uom::si::force::{kilogram_force, kilonewton, newton, pound_force};
use uom::si::frequency::{cycle_per_minute, hertz, kilohertz, megahertz};
use uom::si::length::{centimeter, foot, inch, kilometer, meter, mile, millimeter};
use uom::si::mass::{gram, kilogram, pound};
use uom::si::power::{horsepower, kilowatt, watt};
use uom::si::pressure::{atmosphere, bar, kilopascal, megapascal, pascal, psi};
use uom::si::time::{hour, minute, second};
use uom::si::velocity::{foot_per_second, kilometer_per_hour, meter_per_second, mile_per_hour};
use uom::si::volume::{cubic_foot, cubic_meter, gallon, liter, milliliter};

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
    Pressure,
    Temperature,
    Energy,
    Force,
    Power,
    Area,
    Volume,
    Frequency,
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
                    Dimension::Pressure => {
                        let value = parse_pressure(left)? + parse_pressure(right)?;
                        Ok((Dimension::Pressure, get_pressure(value, to)?, to.clone()))
                    }
                    Dimension::Temperature => {
                        let k = parse_temperature_kelvin(left)? + parse_temperature_kelvin(right)?;
                        Ok((
                            Dimension::Temperature,
                            kelvin_to_temperature(k, to)?,
                            to.clone(),
                        ))
                    }
                    Dimension::Energy => {
                        let value = parse_energy(left)? + parse_energy(right)?;
                        Ok((Dimension::Energy, get_energy(value, to)?, to.clone()))
                    }
                    Dimension::Force => {
                        let value = parse_force(left)? + parse_force(right)?;
                        Ok((Dimension::Force, get_force(value, to)?, to.clone()))
                    }
                    Dimension::Power => {
                        let value = parse_power(left)? + parse_power(right)?;
                        Ok((Dimension::Power, get_power(value, to)?, to.clone()))
                    }
                    Dimension::Area => {
                        let value = parse_area(left)? + parse_area(right)?;
                        Ok((Dimension::Area, get_area(value, to)?, to.clone()))
                    }
                    Dimension::Volume => {
                        let value = parse_volume(left)? + parse_volume(right)?;
                        Ok((Dimension::Volume, get_volume(value, to)?, to.clone()))
                    }
                    Dimension::Frequency => {
                        let value = parse_frequency(left)? + parse_frequency(right)?;
                        Ok((Dimension::Frequency, get_frequency(value, to)?, to.clone()))
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
                    Dimension::Pressure => {
                        let value = parse_pressure(left)? - parse_pressure(right)?;
                        Ok((Dimension::Pressure, get_pressure(value, to)?, to.clone()))
                    }
                    Dimension::Temperature => {
                        let k = parse_temperature_kelvin(left)? - parse_temperature_kelvin(right)?;
                        Ok((
                            Dimension::Temperature,
                            kelvin_to_temperature(k, to)?,
                            to.clone(),
                        ))
                    }
                    Dimension::Energy => {
                        let value = parse_energy(left)? - parse_energy(right)?;
                        Ok((Dimension::Energy, get_energy(value, to)?, to.clone()))
                    }
                    Dimension::Force => {
                        let value = parse_force(left)? - parse_force(right)?;
                        Ok((Dimension::Force, get_force(value, to)?, to.clone()))
                    }
                    Dimension::Power => {
                        let value = parse_power(left)? - parse_power(right)?;
                        Ok((Dimension::Power, get_power(value, to)?, to.clone()))
                    }
                    Dimension::Area => {
                        let value = parse_area(left)? - parse_area(right)?;
                        Ok((Dimension::Area, get_area(value, to)?, to.clone()))
                    }
                    Dimension::Volume => {
                        let value = parse_volume(left)? - parse_volume(right)?;
                        Ok((Dimension::Volume, get_volume(value, to)?, to.clone()))
                    }
                    Dimension::Frequency => {
                        let value = parse_frequency(left)? - parse_frequency(right)?;
                        Ok((Dimension::Frequency, get_frequency(value, to)?, to.clone()))
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
                "enum": ["length", "mass", "time", "velocity", "pressure", "temperature", "energy", "force", "power", "area", "volume", "frequency"]
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
        Dimension::Pressure => Ok((
            Dimension::Pressure,
            get_pressure(parse_pressure(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Temperature => Ok((
            Dimension::Temperature,
            kelvin_to_temperature(parse_temperature_kelvin(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Energy => Ok((
            Dimension::Energy,
            get_energy(parse_energy(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Force => Ok((
            Dimension::Force,
            get_force(parse_force(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Power => Ok((
            Dimension::Power,
            get_power(parse_power(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Area => Ok((
            Dimension::Area,
            get_area(parse_area(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Volume => Ok((
            Dimension::Volume,
            get_volume(parse_volume(quantity)?, to)?,
            to.to_owned(),
        )),
        Dimension::Frequency => Ok((
            Dimension::Frequency,
            get_frequency(parse_frequency(quantity)?, to)?,
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

fn parse_pressure(quantity: &Quantity) -> Result<Pressure, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "Pa" => Ok(Pressure::new::<pascal>(quantity.value)),
        "kPa" => Ok(Pressure::new::<kilopascal>(quantity.value)),
        "MPa" => Ok(Pressure::new::<megapascal>(quantity.value)),
        "bar" => Ok(Pressure::new::<bar>(quantity.value)),
        "atm" => Ok(Pressure::new::<atmosphere>(quantity.value)),
        "psi" => Ok(Pressure::new::<psi>(quantity.value)),
        unit => Err(format!("unsupported pressure unit `{unit}`")),
    }
}

fn get_pressure(value: Pressure, unit: &str) -> Result<f64, String> {
    match unit {
        "Pa" => Ok(value.get::<pascal>()),
        "kPa" => Ok(value.get::<kilopascal>()),
        "MPa" => Ok(value.get::<megapascal>()),
        "bar" => Ok(value.get::<bar>()),
        "atm" => Ok(value.get::<atmosphere>()),
        "psi" => Ok(value.get::<psi>()),
        unit => Err(format!("unsupported pressure unit `{unit}`")),
    }
}

fn parse_temperature_kelvin(quantity: &Quantity) -> Result<f64, String> {
    ensure_finite(quantity.value)?;
    let v = quantity.value;
    match quantity.unit.as_str() {
        "K" => Ok(v),
        "celsius" => Ok(v + 273.15),
        "fahrenheit" => Ok((v - 32.0) * 5.0 / 9.0 + 273.15),
        unit => Err(format!("unsupported temperature unit `{unit}`")),
    }
}

fn kelvin_to_temperature(kelvin: f64, unit: &str) -> Result<f64, String> {
    match unit {
        "K" => Ok(kelvin),
        "celsius" => Ok(kelvin - 273.15),
        "fahrenheit" => Ok((kelvin - 273.15) * 9.0 / 5.0 + 32.0),
        unit => Err(format!("unsupported temperature unit `{unit}`")),
    }
}

fn parse_energy(quantity: &Quantity) -> Result<Energy, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "J" => Ok(Energy::new::<joule>(quantity.value)),
        "kJ" => Ok(Energy::new::<kilojoule>(quantity.value)),
        "kWh" => Ok(Energy::new::<kilowatt_hour>(quantity.value)),
        "BTU" => Ok(Energy::new::<btu>(quantity.value)),
        "cal" => Ok(Energy::new::<calorie>(quantity.value)),
        unit => Err(format!("unsupported energy unit `{unit}`")),
    }
}

fn get_energy(value: Energy, unit: &str) -> Result<f64, String> {
    match unit {
        "J" => Ok(value.get::<joule>()),
        "kJ" => Ok(value.get::<kilojoule>()),
        "kWh" => Ok(value.get::<kilowatt_hour>()),
        "BTU" => Ok(value.get::<btu>()),
        "cal" => Ok(value.get::<calorie>()),
        unit => Err(format!("unsupported energy unit `{unit}`")),
    }
}

fn parse_force(quantity: &Quantity) -> Result<Force, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "N" => Ok(Force::new::<newton>(quantity.value)),
        "kN" => Ok(Force::new::<kilonewton>(quantity.value)),
        "lbf" => Ok(Force::new::<pound_force>(quantity.value)),
        "kgf" => Ok(Force::new::<kilogram_force>(quantity.value)),
        unit => Err(format!("unsupported force unit `{unit}`")),
    }
}

fn get_force(value: Force, unit: &str) -> Result<f64, String> {
    match unit {
        "N" => Ok(value.get::<newton>()),
        "kN" => Ok(value.get::<kilonewton>()),
        "lbf" => Ok(value.get::<pound_force>()),
        "kgf" => Ok(value.get::<kilogram_force>()),
        unit => Err(format!("unsupported force unit `{unit}`")),
    }
}

fn parse_power(quantity: &Quantity) -> Result<Power, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "W" => Ok(Power::new::<watt>(quantity.value)),
        "kW" => Ok(Power::new::<kilowatt>(quantity.value)),
        "hp" => Ok(Power::new::<horsepower>(quantity.value)),
        unit => Err(format!("unsupported power unit `{unit}`")),
    }
}

fn get_power(value: Power, unit: &str) -> Result<f64, String> {
    match unit {
        "W" => Ok(value.get::<watt>()),
        "kW" => Ok(value.get::<kilowatt>()),
        "hp" => Ok(value.get::<horsepower>()),
        unit => Err(format!("unsupported power unit `{unit}`")),
    }
}

fn parse_area(quantity: &Quantity) -> Result<Area, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "m2" => Ok(Area::new::<square_meter>(quantity.value)),
        "ft2" => Ok(Area::new::<square_foot>(quantity.value)),
        "acre" => Ok(Area::new::<acre>(quantity.value)),
        "hectare" => Ok(Area::new::<hectare>(quantity.value)),
        unit => Err(format!("unsupported area unit `{unit}`")),
    }
}

fn get_area(value: Area, unit: &str) -> Result<f64, String> {
    match unit {
        "m2" => Ok(value.get::<square_meter>()),
        "ft2" => Ok(value.get::<square_foot>()),
        "acre" => Ok(value.get::<acre>()),
        "hectare" => Ok(value.get::<hectare>()),
        unit => Err(format!("unsupported area unit `{unit}`")),
    }
}

fn parse_volume(quantity: &Quantity) -> Result<Volume, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "L" => Ok(Volume::new::<liter>(quantity.value)),
        "mL" => Ok(Volume::new::<milliliter>(quantity.value)),
        "m3" => Ok(Volume::new::<cubic_meter>(quantity.value)),
        "gal" => Ok(Volume::new::<gallon>(quantity.value)),
        "ft3" => Ok(Volume::new::<cubic_foot>(quantity.value)),
        unit => Err(format!("unsupported volume unit `{unit}`")),
    }
}

fn get_volume(value: Volume, unit: &str) -> Result<f64, String> {
    match unit {
        "L" => Ok(value.get::<liter>()),
        "mL" => Ok(value.get::<milliliter>()),
        "m3" => Ok(value.get::<cubic_meter>()),
        "gal" => Ok(value.get::<gallon>()),
        "ft3" => Ok(value.get::<cubic_foot>()),
        unit => Err(format!("unsupported volume unit `{unit}`")),
    }
}

fn parse_frequency(quantity: &Quantity) -> Result<Frequency, String> {
    ensure_finite(quantity.value)?;
    match quantity.unit.as_str() {
        "Hz" => Ok(Frequency::new::<hertz>(quantity.value)),
        "kHz" => Ok(Frequency::new::<kilohertz>(quantity.value)),
        "MHz" => Ok(Frequency::new::<megahertz>(quantity.value)),
        "rpm" => Ok(Frequency::new::<cycle_per_minute>(quantity.value)),
        unit => Err(format!("unsupported frequency unit `{unit}`")),
    }
}

fn get_frequency(value: Frequency, unit: &str) -> Result<f64, String> {
    match unit {
        "Hz" => Ok(value.get::<hertz>()),
        "kHz" => Ok(value.get::<kilohertz>()),
        "MHz" => Ok(value.get::<megahertz>()),
        "rpm" => Ok(value.get::<cycle_per_minute>()),
        unit => Err(format!("unsupported frequency unit `{unit}`")),
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

    #[test]
    fn converts_pressure_psi_to_mpa() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Pressure, 45_000.0, "psi"),
            to: "MPa".to_owned(),
        }
        .evaluate();

        match response {
            UnitResponse::Solved { value, unit, .. } => {
                assert!((value - 310.264_2).abs() < 0.01, "got {value}");
                assert_eq!(unit, "MPa");
            }
            other => panic!("expected solved, got {other:?}"),
        }
    }

    #[test]
    fn converts_pressure_bar_atm_kpa() {
        let bar_to_kpa = UnitRequest::Convert {
            quantity: q(Dimension::Pressure, 1.0, "bar"),
            to: "kPa".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(bar_to_kpa, UnitResponse::Solved { value, .. } if (value - 100.0).abs() < 1e-6)
        );

        let atm_to_pa = UnitRequest::Convert {
            quantity: q(Dimension::Pressure, 1.0, "atm"),
            to: "Pa".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(atm_to_pa, UnitResponse::Solved { value, .. } if (value - 101_325.0).abs() < 1.0)
        );
    }

    #[test]
    fn converts_temperature_celsius_to_fahrenheit() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Temperature, 100.0, "celsius"),
            to: "fahrenheit".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 212.0).abs() < 1e-9)
        );
    }

    #[test]
    fn converts_temperature_fahrenheit_to_celsius() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Temperature, 32.0, "fahrenheit"),
            to: "celsius".to_owned(),
        }
        .evaluate();
        assert!(matches!(response, UnitResponse::Solved { value, .. } if value.abs() < 1e-9));
    }

    #[test]
    fn converts_temperature_celsius_to_kelvin() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Temperature, 0.0, "celsius"),
            to: "K".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 273.15).abs() < 1e-9)
        );
    }

    #[test]
    fn converts_energy_kwh_to_joules() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Energy, 1.0, "kWh"),
            to: "J".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 3_600_000.0).abs() < 1.0)
        );
    }

    #[test]
    fn converts_energy_btu_to_kj() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Energy, 1.0, "BTU"),
            to: "kJ".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 1.054_350).abs() < 1e-4)
        );
    }

    #[test]
    fn converts_force_lbf_to_newtons() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Force, 1.0, "lbf"),
            to: "N".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 4.448_222).abs() < 1e-4)
        );
    }

    #[test]
    fn converts_force_kgf_and_kn() {
        let kgf_to_n = UnitRequest::Convert {
            quantity: q(Dimension::Force, 1.0, "kgf"),
            to: "N".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(kgf_to_n, UnitResponse::Solved { value, .. } if (value - 9.806_65).abs() < 1e-4)
        );

        let kn_to_n = UnitRequest::Convert {
            quantity: q(Dimension::Force, 1.0, "kN"),
            to: "N".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(kn_to_n, UnitResponse::Solved { value, .. } if (value - 1000.0).abs() < 1e-9)
        );
    }

    #[test]
    fn converts_power_hp_to_kw() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Power, 1.0, "hp"),
            to: "kW".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 0.745_699_9).abs() < 1e-4)
        );
    }

    #[test]
    fn converts_area_acre_to_hectare() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Area, 1.0, "acre"),
            to: "hectare".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 0.404_687_3).abs() < 1e-4)
        );
    }

    #[test]
    fn converts_area_m2_to_ft2() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Area, 1.0, "m2"),
            to: "ft2".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 10.763_91).abs() < 1e-3)
        );
    }

    #[test]
    fn converts_volume_liter_to_gallon() {
        let response = UnitRequest::Convert {
            quantity: q(Dimension::Volume, 1.0, "L"),
            to: "gal".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(response, UnitResponse::Solved { value, .. } if (value - 0.264_172).abs() < 1e-4)
        );
    }

    #[test]
    fn converts_volume_m3_ft3_ml() {
        let m3_to_l = UnitRequest::Convert {
            quantity: q(Dimension::Volume, 1.0, "m3"),
            to: "L".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(m3_to_l, UnitResponse::Solved { value, .. } if (value - 1000.0).abs() < 1e-9)
        );

        let ml_to_l = UnitRequest::Convert {
            quantity: q(Dimension::Volume, 1000.0, "mL"),
            to: "L".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(ml_to_l, UnitResponse::Solved { value, .. } if (value - 1.0).abs() < 1e-9)
        );

        let ft3_to_l = UnitRequest::Convert {
            quantity: q(Dimension::Volume, 1.0, "ft3"),
            to: "L".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(ft3_to_l, UnitResponse::Solved { value, .. } if (value - 28.316_85).abs() < 1e-3)
        );
    }

    #[test]
    fn converts_frequency_hz_khz_mhz_rpm() {
        let khz = UnitRequest::Convert {
            quantity: q(Dimension::Frequency, 1000.0, "Hz"),
            to: "kHz".to_owned(),
        }
        .evaluate();
        assert!(matches!(khz, UnitResponse::Solved { value, .. } if (value - 1.0).abs() < 1e-9));

        let mhz = UnitRequest::Convert {
            quantity: q(Dimension::Frequency, 1.0, "MHz"),
            to: "kHz".to_owned(),
        }
        .evaluate();
        assert!(matches!(mhz, UnitResponse::Solved { value, .. } if (value - 1000.0).abs() < 1e-9));

        let rpm = UnitRequest::Convert {
            quantity: q(Dimension::Frequency, 60.0, "rpm"),
            to: "Hz".to_owned(),
        }
        .evaluate();
        assert!(matches!(rpm, UnitResponse::Solved { value, .. } if (value - 1.0).abs() < 1e-9));
    }

    #[test]
    fn adds_and_subs_new_dimensions() {
        let add_pressure = UnitRequest::Add {
            left: q(Dimension::Pressure, 100.0, "kPa"),
            right: q(Dimension::Pressure, 1.0, "bar"),
            to: "kPa".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_pressure, UnitResponse::Solved { value, .. } if (value - 200.0).abs() < 1e-6)
        );

        let sub_energy = UnitRequest::Sub {
            left: q(Dimension::Energy, 1000.0, "J"),
            right: q(Dimension::Energy, 500.0, "J"),
            to: "J".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(sub_energy, UnitResponse::Solved { value, .. } if (value - 500.0).abs() < 1e-9)
        );

        let add_force = UnitRequest::Add {
            left: q(Dimension::Force, 1.0, "kN"),
            right: q(Dimension::Force, 500.0, "N"),
            to: "N".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_force, UnitResponse::Solved { value, .. } if (value - 1500.0).abs() < 1e-9)
        );

        let add_power = UnitRequest::Add {
            left: q(Dimension::Power, 1.0, "kW"),
            right: q(Dimension::Power, 500.0, "W"),
            to: "W".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_power, UnitResponse::Solved { value, .. } if (value - 1500.0).abs() < 1e-9)
        );

        let add_area = UnitRequest::Add {
            left: q(Dimension::Area, 1.0, "hectare"),
            right: q(Dimension::Area, 10_000.0, "m2"),
            to: "m2".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_area, UnitResponse::Solved { value, .. } if (value - 20_000.0).abs() < 1e-6)
        );

        let add_volume = UnitRequest::Add {
            left: q(Dimension::Volume, 1.0, "L"),
            right: q(Dimension::Volume, 500.0, "mL"),
            to: "L".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_volume, UnitResponse::Solved { value, .. } if (value - 1.5).abs() < 1e-9)
        );

        let add_freq = UnitRequest::Add {
            left: q(Dimension::Frequency, 1.0, "kHz"),
            right: q(Dimension::Frequency, 500.0, "Hz"),
            to: "Hz".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_freq, UnitResponse::Solved { value, .. } if (value - 1500.0).abs() < 1e-9)
        );
    }

    #[test]
    fn temperature_add_and_sub_arithmetic_is_correct() {
        // Add: 100K + 200K = 300K (kills + → - and + → *)
        let add_k = UnitRequest::Add {
            left: q(Dimension::Temperature, 100.0, "K"),
            right: q(Dimension::Temperature, 200.0, "K"),
            to: "K".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(add_k, UnitResponse::Solved { value, .. } if (value - 300.0).abs() < 1e-9)
        );

        // Sub: 300K - 100K = 200K (kills - → + and - → /)
        let sub_k = UnitRequest::Sub {
            left: q(Dimension::Temperature, 300.0, "K"),
            right: q(Dimension::Temperature, 100.0, "K"),
            to: "K".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(sub_k, UnitResponse::Solved { value, .. } if (value - 200.0).abs() < 1e-9)
        );
    }

    #[test]
    fn energy_add_is_addition_not_subtraction() {
        // Add: 1kJ + 2kJ = 3kJ (kills + → -)
        let add = UnitRequest::Add {
            left: q(Dimension::Energy, 1.0, "kJ"),
            right: q(Dimension::Energy, 2.0, "kJ"),
            to: "kJ".to_owned(),
        }
        .evaluate();
        assert!(matches!(add, UnitResponse::Solved { value, .. } if (value - 3.0).abs() < 1e-9));
    }

    #[test]
    fn sub_dimension_operators_are_subtraction() {
        // Each Sub case uses a distinct (a, b, a-b) triple so + → - is always caught

        // Pressure: 300kPa - 100kPa = 200kPa
        let p = UnitRequest::Sub {
            left: q(Dimension::Pressure, 300.0, "kPa"),
            right: q(Dimension::Pressure, 100.0, "kPa"),
            to: "kPa".to_owned(),
        }
        .evaluate();
        assert!(matches!(p, UnitResponse::Solved { value, .. } if (value - 200.0).abs() < 1e-6));

        // Force: 3kN - 1kN = 2kN
        let f = UnitRequest::Sub {
            left: q(Dimension::Force, 3.0, "kN"),
            right: q(Dimension::Force, 1.0, "kN"),
            to: "kN".to_owned(),
        }
        .evaluate();
        assert!(matches!(f, UnitResponse::Solved { value, .. } if (value - 2.0).abs() < 1e-9));

        // Power: 5kW - 2kW = 3kW
        let pw = UnitRequest::Sub {
            left: q(Dimension::Power, 5.0, "kW"),
            right: q(Dimension::Power, 2.0, "kW"),
            to: "kW".to_owned(),
        }
        .evaluate();
        assert!(matches!(pw, UnitResponse::Solved { value, .. } if (value - 3.0).abs() < 1e-9));

        // Area: 3 hectares - 1 hectare = 2 hectares
        let a = UnitRequest::Sub {
            left: q(Dimension::Area, 3.0, "hectare"),
            right: q(Dimension::Area, 1.0, "hectare"),
            to: "hectare".to_owned(),
        }
        .evaluate();
        assert!(matches!(a, UnitResponse::Solved { value, .. } if (value - 2.0).abs() < 1e-9));

        // Volume: 5L - 2L = 3L
        let v = UnitRequest::Sub {
            left: q(Dimension::Volume, 5.0, "L"),
            right: q(Dimension::Volume, 2.0, "L"),
            to: "L".to_owned(),
        }
        .evaluate();
        assert!(matches!(v, UnitResponse::Solved { value, .. } if (value - 3.0).abs() < 1e-9));

        // Frequency: 5kHz - 2kHz = 3kHz
        let fr = UnitRequest::Sub {
            left: q(Dimension::Frequency, 5.0, "kHz"),
            right: q(Dimension::Frequency, 2.0, "kHz"),
            to: "kHz".to_owned(),
        }
        .evaluate();
        assert!(matches!(fr, UnitResponse::Solved { value, .. } if (value - 3.0).abs() < 1e-9));
    }

    #[test]
    fn fahrenheit_non_zero_offset_conversion_is_correct() {
        // 212°F → 100°C; kills * → /, / → *, / → % (since 212-32 = 180 ≠ 0)
        let f_to_c = UnitRequest::Convert {
            quantity: q(Dimension::Temperature, 212.0, "fahrenheit"),
            to: "celsius".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(f_to_c, UnitResponse::Solved { value, .. } if (value - 100.0).abs() < 1e-6)
        );

        // -40°F = -40°C (additional non-trivial check)
        let neg = UnitRequest::Convert {
            quantity: q(Dimension::Temperature, -40.0, "fahrenheit"),
            to: "celsius".to_owned(),
        }
        .evaluate();
        assert!(
            matches!(neg, UnitResponse::Solved { value, .. } if (value - (-40.0)).abs() < 1e-6)
        );
    }

    #[test]
    fn rejects_unsupported_units_for_new_dimensions() {
        let cases: Vec<(Dimension, &str)> = vec![
            (Dimension::Pressure, "torr"),
            (Dimension::Temperature, "rankine"),
            (Dimension::Energy, "erg"),
            (Dimension::Force, "dyn"),
            (Dimension::Power, "BTU/h"),
            (Dimension::Area, "yd2"),
            (Dimension::Volume, "qt"),
            (Dimension::Frequency, "GHz"),
        ];

        for (dim, unit) in cases {
            let response = UnitRequest::Convert {
                quantity: q(dim, 1.0, unit),
                to: unit.to_owned(),
            }
            .evaluate();
            assert!(
                matches!(&response, UnitResponse::Error { reason, .. } if reason.contains("unsupported")),
                "expected unsupported error for {dim:?}/{unit}, got {response:?}"
            );
        }
    }
}
