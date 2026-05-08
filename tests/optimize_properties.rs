use agent_calc::{Objective, OptimizeRequest, OptimizeResponse};
use proptest::prelude::*;

proptest! {
    #[test]
    fn convex_quadratic_minimizer_matches_analytic_vertex(
        a in 0.1f64..1000.0,
        vertex in -1000.0f64..1000.0,
        c in -1000.0f64..1000.0,
    ) {
        prop_assume!(a.is_finite() && vertex.is_finite() && c.is_finite());
        let b = -2.0 * a * vertex;
        let lower = vertex - 100.0;
        let upper = vertex + 100.0;

        match (OptimizeRequest::Minimize1d {
            objective: Objective::Quadratic { a, b, c },
            lower,
            upper,
            max_iters: 100,
            abs_tolerance: 1e-8,
        }).evaluate() {
            OptimizeResponse::Optimum { minimizer, minimum, .. } => {
                let analytic_minimum = c - a * vertex * vertex;
                prop_assert!((minimizer - vertex).abs() <= 1e-4);
                prop_assert!((minimum - analytic_minimum).abs() <= analytic_minimum.abs().max(1.0) * 1e-6);
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn objective_evaluation_matches_quadratic_formula(
        a in 0.1f64..1000.0,
        b in -1000.0f64..1000.0,
        c in -1000.0f64..1000.0,
        x in -1000.0f64..1000.0,
    ) {
        prop_assume!(a.is_finite() && b.is_finite() && c.is_finite() && x.is_finite());
        let objective = Objective::Quadratic { a, b, c };
        prop_assert_eq!(objective.evaluate_at(x), a * x * x + b * x + c);
    }
}
