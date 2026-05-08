use agent_calc::{CalculusRequest, CalculusResponse, Expr, SubstituteRequest, SubstituteResponse};
use proptest::prelude::*;
use std::collections::BTreeMap;

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

fn pow(base: Expr, exponent: i32) -> Expr {
    Expr::Pow {
        base: Box::new(base),
        exponent,
    }
}

fn derivative(expr: Expr) -> Expr {
    match (CalculusRequest::Derivative {
        expr,
        variable: "x".to_owned(),
    })
    .evaluate()
    {
        CalculusResponse::Derivative { expr, .. } => expr,
        other => panic!("expected derivative, got {other:?}"),
    }
}

fn eval_at(expr: Expr, x: i32) -> i128 {
    let mut bindings = BTreeMap::new();
    bindings.insert("x".to_owned(), int(x));
    let substituted = match (SubstituteRequest {
        expr,
        bindings,
        eval_after: false,
        decimal_places: 12,
    })
    .substitute()
    {
        SubstituteResponse::Substituted { expr, .. } => expr,
        other => panic!("expected substituted expression, got {other:?}"),
    };
    substituted
        .evaluate()
        .unwrap()
        .to_string()
        .parse::<i128>()
        .unwrap()
}

proptest! {
    #[test]
    fn derivative_of_affine_expression_is_constant(a in -100i32..=100, b in -1_000i32..=1_000, x in -100i32..=100) {
        let expr = add(mul(int(a), symbol("x")), int(b));
        let derivative = derivative(expr);

        prop_assert_eq!(eval_at(derivative, x), i128::from(a));
    }

    #[test]
    fn derivative_of_power_matches_power_rule(exponent in 1i32..=8, x in -10i32..=10) {
        let expr = pow(symbol("x"), exponent);
        let derivative = derivative(expr);
        let expected = i128::from(exponent) * i128::from(x).pow((exponent - 1) as u32);

        prop_assert_eq!(eval_at(derivative, x), expected);
    }

    #[test]
    fn derivative_is_additive(a in -20i32..=20, b in -20i32..=20, x in -20i32..=20) {
        let left = pow(symbol("x"), 2);
        let right = add(mul(int(a), symbol("x")), int(b));
        let derivative_sum = derivative(add(left.clone(), right.clone()));
        let separate = add(derivative(left), derivative(right));

        prop_assert_eq!(eval_at(derivative_sum, x), eval_at(separate, x));
    }
}
