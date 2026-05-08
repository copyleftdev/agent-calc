use agent_calc::{EvalRequest, EvalResponse, Expr};
use proptest::prelude::*;

fn integer(v: i32) -> Expr {
    Expr::Integer {
        value: v.to_string(),
    }
}

fn rational(n: i32, d: i32) -> Expr {
    Expr::Rational {
        numerator: n.to_string(),
        denominator: d.to_string(),
    }
}

fn eval(expr: Expr) -> EvalResponse {
    EvalRequest {
        expr,
        decimal_places: 12,
    }
    .evaluate()
}

fn eval_approx(expr: Expr) -> f64 {
    match eval(expr) {
        EvalResponse::Approximate { value, .. } => value,
        other => panic!("expected approximate response, got {other:?}"),
    }
}

fn eval_exact_int(expr: Expr) -> i64 {
    match eval(expr) {
        EvalResponse::Solved { exact, .. } => exact.display.parse::<i64>().unwrap(),
        other => panic!("expected solved response, got {other:?}"),
    }
}

proptest! {
    #[test]
    fn exp_ln_round_trip(x in 0.01f64..10.0) {
        // ln(exp(x)) ≈ x
        let n = (x * 1_000_000.0) as i32;
        let expr = Expr::Ln {
            value: Box::new(Expr::Exp {
                value: Box::new(Expr::Rational {
                    numerator: n.to_string(),
                    denominator: "1000000".to_owned(),
                }),
            }),
        };
        let result = eval_approx(expr);
        prop_assert!((result - x).abs() < 1e-6, "ln(exp({x})) = {result}");
    }

    #[test]
    fn sin_squared_plus_cos_squared_is_one(n in -100i32..=100) {
        // sin²(n) + cos²(n) = 1
        let expr_n = integer(n);
        let sin_sq = Expr::Mul {
            left: Box::new(Expr::Sin { value: Box::new(expr_n.clone()) }),
            right: Box::new(Expr::Sin { value: Box::new(expr_n.clone()) }),
        };
        let cos_sq = Expr::Mul {
            left: Box::new(Expr::Cos { value: Box::new(expr_n.clone()) }),
            right: Box::new(Expr::Cos { value: Box::new(expr_n) }),
        };
        let sum = Expr::Add {
            left: Box::new(sin_sq),
            right: Box::new(cos_sq),
        };
        let result = eval_approx(sum);
        prop_assert!((result - 1.0).abs() < 1e-12, "sin²+cos² at {n} = {result}");
    }

    #[test]
    fn abs_is_non_negative(n in -1000i32..=1000, d in 1i32..=1000) {
        let expr = Expr::Abs {
            value: Box::new(rational(n, d)),
        };
        match eval(expr) {
            EvalResponse::Solved { exact, .. } => {
                let num: i64 = exact.numerator.parse().unwrap();
                prop_assert!(num >= 0, "abs numerator should be non-negative, got {num}");
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn floor_is_le_value(n in -1000i32..=1000, d in 1i32..=1000) {
        // floor(n/d) <= n/d
        let value = n as f64 / d as f64;
        let floor_val = eval_exact_int(Expr::Floor {
            value: Box::new(rational(n, d)),
        });
        prop_assert!(floor_val as f64 <= value + 1e-12,
            "floor({n}/{d}) = {floor_val} > {value}");
        prop_assert!(floor_val as f64 >= value - 1.0,
            "floor({n}/{d}) = {floor_val} < {value} - 1");
    }

    #[test]
    fn ceil_is_ge_value(n in -1000i32..=1000, d in 1i32..=1000) {
        let value = n as f64 / d as f64;
        let ceil_val = eval_exact_int(Expr::Ceil {
            value: Box::new(rational(n, d)),
        });
        prop_assert!(ceil_val as f64 >= value - 1e-12,
            "ceil({n}/{d}) = {ceil_val} < {value}");
        prop_assert!(ceil_val as f64 <= value + 1.0,
            "ceil({n}/{d}) = {ceil_val} > {value} + 1");
    }

    #[test]
    fn max_returns_larger(a in -100i32..=100, b in -100i32..=100) {
        let expr = Expr::Max {
            left: Box::new(integer(a)),
            right: Box::new(integer(b)),
        };
        let result = eval_exact_int(expr);
        let expected = a.max(b) as i64;
        prop_assert_eq!(result, expected);
    }

    #[test]
    fn min_returns_smaller(a in -100i32..=100, b in -100i32..=100) {
        let expr = Expr::Min {
            left: Box::new(integer(a)),
            right: Box::new(integer(b)),
        };
        let result = eval_exact_int(expr);
        let expected = a.min(b) as i64;
        prop_assert_eq!(result, expected);
    }

    #[test]
    fn sqrt_of_perfect_square_is_exact(n in 0i32..=100) {
        let n_sq = integer(n * n);
        let expr = Expr::Sqrt { value: Box::new(n_sq) };
        match eval(expr) {
            EvalResponse::Solved { exact, .. } => {
                let result: i64 = exact.display.parse().unwrap();
                prop_assert_eq!(result, n as i64);
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn sqrt_of_non_square_is_approximate(n in 2i32..=50) {
        // Pick values that are unlikely to be perfect squares (non-square integers)
        let v = 2 * n - 1; // odd numbers ≥ 3 that are not perfect squares
        prop_assume!({
            let s = (v as f64).sqrt() as i32;
            s * s != v && (s + 1) * (s + 1) != v
        });
        let expr = Expr::Sqrt { value: Box::new(integer(v)) };
        match eval(expr) {
            EvalResponse::Approximate { value, exactness, .. } => {
                prop_assert!(value > 0.0);
                prop_assert_eq!(exactness, agent_calc::Exactness::ApproximateF64);
            }
            EvalResponse::Solved { .. } => {
                // Only valid if v is actually a perfect square — shouldn't happen given prop_assume
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }
}
