use agent_calc::{Expr, PolynomialInput, PolynomialRequest, PolynomialResponse};
use proptest::prelude::*;

fn int(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
    }
}

fn poly(coefficients: Vec<i32>) -> PolynomialInput {
    PolynomialInput {
        variable: "x".to_owned(),
        coefficients: coefficients.into_iter().map(int).collect(),
    }
}

fn coefficients(response: PolynomialResponse) -> Vec<String> {
    match response {
        PolynomialResponse::Polynomial { coefficients, .. } => coefficients
            .into_iter()
            .map(|value| value.display)
            .collect(),
        PolynomialResponse::Error { reason, .. } => {
            panic!("expected polynomial response: {reason}")
        }
        other => panic!("expected polynomial response, got {other:?}"),
    }
}

fn value(response: PolynomialResponse) -> String {
    match response {
        PolynomialResponse::Value { exact, .. } => exact.display,
        PolynomialResponse::Error { reason, .. } => panic!("expected value response: {reason}"),
        other => panic!("expected value response, got {other:?}"),
    }
}

fn small_coefficients() -> impl Strategy<Value = Vec<i32>> {
    prop::collection::vec(-20i32..=20, 0..8)
}

proptest! {
    #[test]
    fn adding_zero_polynomial_preserves_normalized_polynomial(coeffs in small_coefficients()) {
        let normalized = coefficients(PolynomialRequest::Normalize {
            polynomial: poly(coeffs.clone()),
        }.evaluate());
        let added = coefficients(PolynomialRequest::Add {
            left: poly(coeffs),
            right: poly(vec![0]),
        }.evaluate());

        prop_assert_eq!(added, normalized);
    }

    #[test]
    fn multiplying_by_one_polynomial_preserves_normalized_polynomial(coeffs in small_coefficients()) {
        let normalized = coefficients(PolynomialRequest::Normalize {
            polynomial: poly(coeffs.clone()),
        }.evaluate());
        let multiplied = coefficients(PolynomialRequest::Mul {
            left: poly(coeffs),
            right: poly(vec![1]),
        }.evaluate());

        prop_assert_eq!(multiplied, normalized);
    }

    #[test]
    fn derivative_of_constant_is_zero(c in -100i32..=100) {
        let derivative = coefficients(PolynomialRequest::Derivative {
            polynomial: poly(vec![c]),
        }.evaluate());

        prop_assert_eq!(derivative, vec!["0"]);
    }

    #[test]
    fn evaluating_sum_equals_sum_of_evaluations(
        left in small_coefficients(),
        right in small_coefficients(),
        at in -8i32..=8,
    ) {
        let sum = value(PolynomialRequest::Evaluate {
            polynomial: match (PolynomialRequest::Add {
                left: poly(left.clone()),
                right: poly(right.clone()),
            }).evaluate() {
                PolynomialResponse::Polynomial { coefficients, .. } => PolynomialInput {
                    variable: "x".to_owned(),
                    coefficients: coefficients.into_iter().map(|value| Expr::Rational {
                        numerator: value.numerator,
                        denominator: value.denominator,
                    }).collect(),
                },
                other => panic!("expected polynomial response, got {other:?}"),
            },
            at: int(at),
        }.evaluate());

        let left_value = value(PolynomialRequest::Evaluate {
            polynomial: poly(left),
            at: int(at),
        }.evaluate()).parse::<i128>().unwrap();
        let right_value = value(PolynomialRequest::Evaluate {
            polynomial: poly(right),
            at: int(at),
        }.evaluate()).parse::<i128>().unwrap();

        prop_assert_eq!(sum.parse::<i128>().unwrap(), left_value + right_value);
    }

    #[test]
    fn solving_constructed_linear_root_evaluates_to_zero(root in -50i32..=50, slope in 1i32..=20) {
        let polynomial = poly(vec![-slope * root, slope]);
        let roots = match (PolynomialRequest::Solve {
            polynomial: polynomial.clone(),
        }).evaluate() {
            PolynomialResponse::Roots { roots, .. } => roots,
            other => panic!("expected roots response, got {other:?}"),
        };

        prop_assert_eq!(roots.len(), 1);
        let value_at_root = value(PolynomialRequest::Evaluate {
            polynomial,
            at: Expr::Rational {
                numerator: roots[0].numerator.clone(),
                denominator: roots[0].denominator.clone(),
            },
        }.evaluate());
        prop_assert_eq!(value_at_root, "0");
    }

    #[test]
    fn solving_constructed_quadratic_roots_evaluate_to_zero(
        first in -20i32..=20,
        second in -20i32..=20,
    ) {
        let sum = first + second;
        let product = first * second;
        let polynomial = poly(vec![product, -sum, 1]);
        let roots = match (PolynomialRequest::Solve {
            polynomial: polynomial.clone(),
        }).evaluate() {
            PolynomialResponse::Roots { roots, .. } => roots,
            other => panic!("expected roots response, got {other:?}"),
        };

        for root in roots {
            let value_at_root = value(PolynomialRequest::Evaluate {
                polynomial: polynomial.clone(),
                at: Expr::Rational {
                    numerator: root.numerator,
                    denominator: root.denominator,
                },
            }.evaluate());
            prop_assert_eq!(value_at_root, "0");
        }
    }
}
