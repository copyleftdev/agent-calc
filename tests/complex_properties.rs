use agent_calc::{ComplexInput, ComplexRequest, ComplexResponse};
use proptest::prelude::*;

fn z(re: f64, im: f64) -> ComplexInput {
    ComplexInput { re, im }
}

fn finite_pair() -> impl Strategy<Value = ComplexInput> {
    (-1.0e6f64..1.0e6, -1.0e6f64..1.0e6)
        .prop_filter("finite complex parts", |(re, im)| {
            re.is_finite() && im.is_finite()
        })
        .prop_map(|(re, im)| z(re, im))
}

fn complex(response: ComplexResponse) -> ComplexInput {
    match response {
        ComplexResponse::Complex { re, im, .. } => z(re, im),
        ComplexResponse::Error { reason, .. } => panic!("expected complex response: {reason}"),
        other => panic!("expected complex response, got {other:?}"),
    }
}

fn scalar(response: ComplexResponse) -> f64 {
    match response {
        ComplexResponse::Scalar { value, .. } => value,
        ComplexResponse::Error { reason, .. } => panic!("expected scalar response: {reason}"),
        other => panic!("expected scalar response, got {other:?}"),
    }
}

proptest! {
    #[test]
    fn addition_is_commutative(a in finite_pair(), b in finite_pair()) {
        let left = complex(ComplexRequest::Add { left: a, right: b }.evaluate());
        let right = complex(ComplexRequest::Add { left: b, right: a }.evaluate());
        prop_assert_eq!(left, right);
    }

    #[test]
    fn multiplication_is_commutative(a in finite_pair(), b in finite_pair()) {
        let left = complex(ComplexRequest::Mul { left: a, right: b }.evaluate());
        let right = complex(ComplexRequest::Mul { left: b, right: a }.evaluate());
        prop_assert!((left.re - right.re).abs() <= left.re.abs().max(right.re.abs()).max(1.0) * 1e-12);
        prop_assert!((left.im - right.im).abs() <= left.im.abs().max(right.im.abs()).max(1.0) * 1e-12);
    }

    #[test]
    fn conjugate_twice_preserves_value(a in finite_pair()) {
        let first = complex(ComplexRequest::Conjugate { value: a }.evaluate());
        let second = complex(ComplexRequest::Conjugate { value: first }.evaluate());
        prop_assert_eq!(second, a);
    }

    #[test]
    fn abs_squared_matches_norm_formula(a in finite_pair()) {
        let abs = scalar(ComplexRequest::Abs { value: a }.evaluate());
        let expected_sq = a.re * a.re + a.im * a.im;
        prop_assert!((abs * abs - expected_sq).abs() <= expected_sq.max(1.0) * 1e-10);
    }
}
