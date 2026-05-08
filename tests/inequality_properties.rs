use agent_calc::{
    Expr, InequalityInput, InequalityRelation, InequalityRequest, InequalityResponse, SolutionSet,
};
use proptest::prelude::*;

fn int(value: i32) -> Expr {
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

fn solve(left: Expr, relation: InequalityRelation, right: Expr) -> InequalityResponse {
    InequalityRequest::Solve {
        inequality: InequalityInput {
            left,
            relation,
            right,
        },
        variable: "x".to_owned(),
    }
    .evaluate()
}

proptest! {
    #[test]
    fn positive_coefficient_less_than_has_upper_bound(a in 1i32..=100, root in -100i32..=100) {
        let rhs = a * root;
        let response = solve(mul(int(a), symbol("x")), InequalityRelation::Lt, int(rhs));

        match response {
            InequalityResponse::SolutionSet { set: SolutionSet::Interval { lower: None, upper: Some(upper) }, .. } => {
                prop_assert_eq!(upper.value.display.as_str(), root.to_string());
                prop_assert!(!upper.inclusive);
            }
            other => prop_assert!(false, "expected upper interval, got {other:?}"),
        }
    }

    #[test]
    fn negative_coefficient_less_than_flips_to_lower_bound(a in -100i32..=-1, root in -100i32..=100) {
        let rhs = a * root;
        let response = solve(mul(int(a), symbol("x")), InequalityRelation::Lt, int(rhs));

        match response {
            InequalityResponse::SolutionSet { set: SolutionSet::Interval { lower: Some(lower), upper: None }, .. } => {
                prop_assert_eq!(lower.value.display.as_str(), root.to_string());
                prop_assert!(!lower.inclusive);
            }
            other => prop_assert!(false, "expected lower interval, got {other:?}"),
        }
    }

    #[test]
    fn constructed_equality_root_is_point(a in -100i32..=100, root in -100i32..=100, b in -1_000i32..=1_000) {
        prop_assume!(a != 0);
        let rhs = a * root + b;
        let response = solve(add(mul(int(a), symbol("x")), int(b)), InequalityRelation::Eq, int(rhs));

        match response {
            InequalityResponse::SolutionSet { set: SolutionSet::Point { value }, .. } => {
                prop_assert_eq!(value.display.as_str(), root.to_string());
            }
            other => prop_assert!(false, "expected point, got {other:?}"),
        }
    }
}
