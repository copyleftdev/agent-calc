use agent_calc::{
    ConstraintRelation, LinearConstraint, LinearRequest, LinearResponse, ObjectiveDirection,
    VariableBounds,
};
use proptest::prelude::*;

fn nonnegative_vars(n: usize) -> Vec<VariableBounds> {
    vec![
        VariableBounds {
            lower: Some(0.0),
            upper: None,
            ..Default::default()
        };
        n
    ]
}

proptest! {
    #[test]
    fn one_variable_upper_bound_maximization_matches_bound(
        coeff in 0.1f64..1000.0,
        upper in 0.1f64..1000.0,
    ) {
        prop_assume!(coeff.is_finite() && upper.is_finite());

        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Maximize,
            objective: vec![coeff],
            variables: vec![VariableBounds { lower: Some(0.0), upper: Some(upper), ..Default::default() }],
            constraints: vec![],
            max_nodes: 1000,
        }).evaluate() {
            LinearResponse::Optimal { objective_value, variables, .. } => {
                prop_assert!((variables[0] - upper).abs() <= upper.max(1.0) * 1e-8);
                prop_assert!((objective_value - coeff * upper).abs() <= (coeff * upper).abs().max(1.0) * 1e-8);
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }

    #[test]
    fn one_variable_lower_bound_minimization_matches_bound(
        coeff in 0.1f64..1000.0,
        lower in -1000.0f64..1000.0,
        width in 0.1f64..1000.0,
    ) {
        prop_assume!(coeff.is_finite() && lower.is_finite() && width.is_finite());
        let upper = lower + width;

        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Minimize,
            objective: vec![coeff],
            variables: vec![VariableBounds { lower: Some(lower), upper: Some(upper), ..Default::default() }],
            constraints: vec![],
            max_nodes: 1000,
        }).evaluate() {
            LinearResponse::Optimal { objective_value, variables, .. } => {
                prop_assert!((variables[0] - lower).abs() <= lower.abs().max(1.0) * 1e-8);
                prop_assert!((objective_value - coeff * lower).abs() <= (coeff * lower).abs().max(1.0) * 1e-8);
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }

    #[test]
    fn nonnegative_sum_constraint_never_exceeds_rhs(
        c1 in 0.1f64..100.0,
        c2 in 0.1f64..100.0,
        rhs in 0.1f64..1000.0,
    ) {
        prop_assume!(c1.is_finite() && c2.is_finite() && rhs.is_finite());

        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Maximize,
            objective: vec![c1, c2],
            variables: nonnegative_vars(2),
            constraints: vec![LinearConstraint {
                coefficients: vec![1.0, 1.0],
                relation: ConstraintRelation::Le,
                rhs,
            }],
            max_nodes: 1000,
        }).evaluate() {
            LinearResponse::Optimal { variables, .. } => {
                prop_assert!(variables[0] >= -1e-7);
                prop_assert!(variables[1] >= -1e-7);
                prop_assert!(variables[0] + variables[1] <= rhs + rhs.max(1.0) * 1e-7);
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }
}
