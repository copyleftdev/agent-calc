use agent_calc::{EquationInput, Expr, SolveRequest, SolveResponse};
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

fn solve(left: Expr, right: Expr) -> SolveResponse {
    SolveRequest::Solve {
        equation: EquationInput { left, right },
        variable: "x".to_owned(),
    }
    .evaluate()
}

proptest! {
    #[test]
    fn constructed_integer_solution_is_recovered(a in -100i32..=100, x in -100i32..=100, b in -1_000i32..=1_000) {
        prop_assume!(a != 0);
        let rhs = a * x + b;
        let response = solve(add(mul(int(a), symbol("x")), int(b)), int(rhs));

        match response {
            SolveResponse::Solutions { solutions, .. } => {
                prop_assert_eq!(solutions.len(), 1);
                prop_assert_eq!(solutions[0].display.as_str(), x.to_string());
            }
            other => prop_assert!(false, "expected one solution, got {other:?}"),
        }
    }

    #[test]
    fn equal_affine_forms_have_infinite_solutions(a in -100i32..=100, b in -1_000i32..=1_000) {
        let expr = add(mul(int(a), symbol("x")), int(b));
        let response = solve(expr.clone(), expr);

        let infinite = matches!(response, SolveResponse::InfiniteSolutions { .. });
        prop_assert!(infinite);
    }

    #[test]
    fn shifted_equal_coefficients_have_no_solution(a in -100i32..=100, b in -1_000i32..=1_000, shift in 1i32..=100) {
        let left = add(mul(int(a), symbol("x")), int(b));
        let right = add(mul(int(a), symbol("x")), int(b + shift));
        let response = solve(left, right);

        let no_solution = matches!(response, SolveResponse::NoSolution { .. });
        prop_assert!(no_solution);
    }
}
