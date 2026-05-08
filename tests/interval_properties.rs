use agent_calc::{Expr, IntervalInput, IntervalRequest, IntervalResponse};
use proptest::prelude::*;

fn int(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
    }
}

fn interval(lower: i32, upper: i32) -> IntervalInput {
    IntervalInput {
        lower: int(lower),
        upper: int(upper),
    }
}

fn bounds(response: IntervalResponse) -> (i128, i128) {
    match response {
        IntervalResponse::Interval { lower, upper, .. } => (
            lower.display.parse::<i128>().unwrap(),
            upper.display.parse::<i128>().unwrap(),
        ),
        IntervalResponse::Error { reason, .. } => panic!("expected interval response: {reason}"),
    }
}

proptest! {
    #[test]
    fn adding_point_intervals_matches_integer_addition(a in -1_000i32..=1_000, b in -1_000i32..=1_000) {
        let (lower, upper) = bounds(IntervalRequest::Add {
            left: interval(a, a),
            right: interval(b, b),
        }.evaluate());

        prop_assert_eq!(lower, i128::from(a + b));
        prop_assert_eq!(upper, i128::from(a + b));
    }

    #[test]
    fn multiplying_point_intervals_matches_integer_multiplication(a in -200i32..=200, b in -200i32..=200) {
        let (lower, upper) = bounds(IntervalRequest::Mul {
            left: interval(a, a),
            right: interval(b, b),
        }.evaluate());

        prop_assert_eq!(lower, i128::from(a * b));
        prop_assert_eq!(upper, i128::from(a * b));
    }

    #[test]
    fn squaring_interval_that_contains_zero_starts_at_zero(lower in -50i32..=0, upper in 0i32..=50) {
        let (range_lower, range_upper) = bounds(IntervalRequest::Pow {
            interval: interval(lower, upper),
            exponent: 2,
        }.evaluate());

        prop_assert_eq!(range_lower, 0);
        prop_assert!(range_upper >= 0);
    }

    #[test]
    fn polynomial_range_contains_endpoint_evaluations(lower in -20i32..=0, upper in 0i32..=20) {
        let (range_lower, range_upper) = bounds(IntervalRequest::PolynomialRange {
            polynomial: agent_calc::PolynomialInput {
                variable: "x".to_owned(),
                coefficients: vec![int(1), int(0), int(1)],
            },
            domain: interval(lower, upper),
        }.evaluate());

        let left_value = i128::from(lower) * i128::from(lower) + 1;
        let right_value = i128::from(upper) * i128::from(upper) + 1;
        prop_assert!(range_lower <= left_value && left_value <= range_upper);
        prop_assert!(range_lower <= right_value && right_value <= range_upper);
    }
}
