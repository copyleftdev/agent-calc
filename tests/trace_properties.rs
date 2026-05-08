use agent_calc::{
    EvalRequest, EvalResponse, Expr, SimplifyRequest, SimplifyResponse, TraceOutput, TraceRequest,
    TraceResponse,
};
use proptest::prelude::*;

fn integer(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
    }
}

fn symbol(name: &str) -> Expr {
    Expr::Symbol {
        name: name.to_owned(),
    }
}

fn add(left: Expr, right: Expr) -> Expr {
    Expr::Add {
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn mul(left: Expr, right: Expr) -> Expr {
    Expr::Mul {
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn assert_contiguous_steps(trace: &[agent_calc::TraceStep]) {
    for (index, step) in trace.iter().enumerate() {
        assert_eq!(step.step, index + 1);
    }
}

proptest! {
    #[test]
    fn trace_eval_matches_direct_eval(left in -10_000i32..10_000, right in -10_000i32..10_000) {
        let expr = add(integer(left), mul(integer(right), integer(2)));
        let direct = EvalRequest {
            expr: expr.clone(),
            decimal_places: 8,
        }.evaluate();
        let traced = TraceRequest::Eval {
            expr,
            decimal_places: 8,
        }.evaluate();

        match (direct, traced) {
            (
                EvalResponse::Solved { exact: direct_exact, decimal: direct_decimal, .. },
                TraceResponse::Evaluated { exact: trace_exact, decimal: trace_decimal, trace, .. },
            ) => {
                prop_assert_eq!(trace_exact, direct_exact);
                prop_assert_eq!(trace_decimal, direct_decimal);
                assert_contiguous_steps(&trace);
                assert!(matches!(
                    trace.last().unwrap().output,
                    TraceOutput::Rational { .. }
                ));
            }
            other => panic!("expected solved/evaluated pair, got {other:?}"),
        }
    }

    #[test]
    fn trace_simplify_matches_direct_simplify(value in -10_000i32..10_000) {
        let expr = add(mul(symbol("x"), integer(1)), integer(value));
        let direct = SimplifyRequest {
            expr: expr.clone(),
        }.simplify();
        let traced = TraceRequest::Simplify { expr }.evaluate();

        match (direct, traced) {
            (
                SimplifyResponse::Simplified { expr: direct_expr, .. },
                TraceResponse::Simplified { expr: trace_expr, trace, .. },
            ) => {
                prop_assert_eq!(trace_expr, direct_expr);
                assert_contiguous_steps(&trace);
                assert!(matches!(
                    trace.last().unwrap().output,
                    TraceOutput::Expr { .. }
                ));
            }
            other => panic!("expected simplified pair, got {other:?}"),
        }
    }
}
