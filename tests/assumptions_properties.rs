use agent_calc::{
    Assumption, AssumptionDomain, AssumptionsRequest, AssumptionsResponse, ComparisonOp, Expr,
};
use proptest::prelude::*;

fn int(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
    }
}

fn cmp(symbol: &str, op: ComparisonOp, value: i32) -> Assumption {
    Assumption::Compare {
        symbol: symbol.to_owned(),
        op,
        value: int(value),
    }
}

fn domain(symbol: &str, domain: AssumptionDomain) -> Assumption {
    Assumption::Domain {
        symbol: symbol.to_owned(),
        domain,
    }
}

proptest! {
    #[test]
    fn stronger_lower_bound_is_retained(a in -1_000i32..=1_000, b in -1_000i32..=1_000) {
        let expected = a.max(b);
        let response = AssumptionsRequest::Bounds {
            assumptions: vec![cmp("x", ComparisonOp::Gte, a), cmp("x", ComparisonOp::Gte, b)],
            symbol: "x".to_owned(),
        }.evaluate();

        match response {
            AssumptionsResponse::Bounds { lower: Some(lower), .. } => {
                prop_assert_eq!(lower.value.display, expected.to_string());
                prop_assert!(lower.inclusive);
            }
            other => prop_assert!(false, "expected bounds response, got {other:?}"),
        }
    }

    #[test]
    fn stronger_upper_bound_is_retained(a in -1_000i32..=1_000, b in -1_000i32..=1_000) {
        let expected = a.min(b);
        let response = AssumptionsRequest::Bounds {
            assumptions: vec![cmp("x", ComparisonOp::Lte, a), cmp("x", ComparisonOp::Lte, b)],
            symbol: "x".to_owned(),
        }.evaluate();

        match response {
            AssumptionsResponse::Bounds { upper: Some(upper), .. } => {
                prop_assert_eq!(upper.value.display, expected.to_string());
                prop_assert!(upper.inclusive);
            }
            other => prop_assert!(false, "expected bounds response, got {other:?}"),
        }
    }

    #[test]
    fn strict_lower_bound_entails_greater_than_smaller_values(bound in -1_000i32..=1_000, delta in 1i32..=100) {
        let query_value = bound - delta;
        let response = AssumptionsRequest::Entails {
            assumptions: vec![cmp("x", ComparisonOp::Gt, bound)],
            query: cmp("x", ComparisonOp::Gt, query_value),
        }.evaluate();

        let entailed = matches!(response, AssumptionsResponse::Entailment { entailed: true, .. });
        prop_assert!(entailed);
    }

    #[test]
    fn positive_domain_entails_nonzero(value in 1i32..=1_000) {
        let response = AssumptionsRequest::Entails {
            assumptions: vec![cmp("x", ComparisonOp::Gte, value)],
            query: domain("x", AssumptionDomain::Nonzero),
        }.evaluate();

        let entailed = matches!(response, AssumptionsResponse::Entailment { entailed: true, .. });
        prop_assert!(entailed);
    }
}
