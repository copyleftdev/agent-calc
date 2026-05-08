use agent_calc::{Dimension, Quantity, UnitRequest, UnitResponse};
use proptest::prelude::*;

fn q(dimension: Dimension, value: f64, unit: &str) -> Quantity {
    Quantity {
        dimension,
        value,
        unit: unit.to_owned(),
    }
}

fn solved_value(response: UnitResponse) -> f64 {
    match response {
        UnitResponse::Solved { value, .. } => value,
        UnitResponse::Error { reason, .. } => panic!("expected solved response: {reason}"),
    }
}

proptest! {
    #[test]
    fn meter_to_centimeter_round_trip(value in -1.0e9f64..1.0e9) {
        prop_assume!(value.is_finite());

        let centimeters = solved_value(UnitRequest::Convert {
            quantity: q(Dimension::Length, value, "m"),
            to: "cm".to_owned(),
        }.evaluate());

        let meters = solved_value(UnitRequest::Convert {
            quantity: q(Dimension::Length, centimeters, "cm"),
            to: "m".to_owned(),
        }.evaluate());

        prop_assert!((meters - value).abs() <= value.abs().max(1.0) * 1e-12);
    }

    #[test]
    fn adding_zero_preserves_length(value in -1.0e9f64..1.0e9) {
        prop_assume!(value.is_finite());

        let result = solved_value(UnitRequest::Add {
            left: q(Dimension::Length, value, "m"),
            right: q(Dimension::Length, 0.0, "km"),
            to: "m".to_owned(),
        }.evaluate());

        prop_assert!((result - value).abs() <= value.abs().max(1.0) * 1e-12);
    }

    #[test]
    fn one_meter_per_second_is_three_point_six_kilometers_per_hour(value in -1.0e6f64..1.0e6) {
        prop_assume!(value.is_finite());

        let result = solved_value(UnitRequest::Convert {
            quantity: q(Dimension::Velocity, value, "m/s"),
            to: "km/h".to_owned(),
        }.evaluate());

        prop_assert!((result - value * 3.6).abs() <= value.abs().max(1.0) * 1e-12);
    }
}
