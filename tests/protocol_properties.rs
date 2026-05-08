use agent_calc::{
    ErrorCode, EvalRequest, EvalResponse, Expr, Rational, SimplifyRequest, SimplifyResponse,
    SubstituteRequest, SubstituteResponse, classify_error,
};
use num_bigint::BigInt;
use proptest::prelude::*;
use std::collections::BTreeMap;

fn leaf_strategy() -> impl Strategy<Value = Expr> {
    (-1_000i128..=1_000, 1i128..=1_000).prop_map(|(n, d)| Expr::Rational {
        numerator: n.to_string(),
        denominator: d.to_string(),
    })
}

fn expr_strategy() -> impl Strategy<Value = Expr> {
    leaf_strategy().prop_recursive(4, 64, 3, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Add {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Sub {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Mul {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), -4i32..=4).prop_map(|(base, exponent)| Expr::Pow {
                base: Box::new(base),
                exponent,
            }),
            inner.prop_map(|value| Expr::Neg {
                value: Box::new(value),
            }),
        ]
    })
}

fn symbol_strategy() -> impl Strategy<Value = Expr> {
    "[A-Za-z_][A-Za-z0-9_]{0,8}".prop_map(|name| Expr::Symbol { name })
}

fn symbolic_expr_strategy() -> impl Strategy<Value = Expr> {
    prop_oneof![leaf_strategy(), symbol_strategy()].prop_recursive(4, 64, 3, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Add {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Sub {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Mul {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Expr::Div {
                left: Box::new(left),
                right: Box::new(right),
            }),
            (inner.clone(), -4i32..=4).prop_map(|(base, exponent)| Expr::Pow {
                base: Box::new(base),
                exponent,
            }),
            inner.prop_map(|value| Expr::Neg {
                value: Box::new(value),
            }),
        ]
    })
}

proptest! {
    #[test]
    fn successful_eval_returns_canonical_exact_result(expr in expr_strategy()) {
        let request = EvalRequest {
            expr,
            decimal_places: 8,
        };

        let response = request.evaluate();
        match response {
            EvalResponse::Solved { exact, checks, .. } => {
                let numerator = exact.numerator.parse::<BigInt>().unwrap();
                let denominator = exact.denominator.parse::<BigInt>().unwrap();
                let rational = Rational::new(numerator, denominator).unwrap();

                prop_assert_eq!(exact.denominator, rational.denominator().to_string());
                prop_assert!(checks.iter().any(|check| check.name == "canonical_rational_form" && check.passed));
            }
            EvalResponse::Approximate { value, .. } => {
                prop_assert!(value.is_finite() || value.is_nan());
            }
            EvalResponse::Error { reason, .. } => {
                prop_assert!(
                    reason.contains("overflow")
                        || reason == "denominator must not be zero"
                        || reason.contains("absolute exponent")
                );
            }
        }
    }

    #[test]
    fn eval_is_deterministic(expr in expr_strategy()) {
        let request = EvalRequest {
            expr,
            decimal_places: 12,
        };

        let first = serde_json::to_value(request.evaluate()).unwrap();
        let second = serde_json::to_value(request.evaluate()).unwrap();
        prop_assert_eq!(first, second);
    }

    #[test]
    fn simplification_is_idempotent(expr in symbolic_expr_strategy()) {
        let first = SimplifyRequest { expr }.simplify();
        match first {
            SimplifyResponse::Simplified { expr, .. } => {
                let second = SimplifyRequest { expr: expr.clone() }.simplify();
                prop_assert_eq!(
                    serde_json::to_value(second).unwrap(),
                    serde_json::to_value(SimplifyRequest { expr }.simplify()).unwrap()
                );
            }
            SimplifyResponse::Error { reason, .. } => {
                prop_assert!(
                    reason.contains("invalid integer")
                        || reason.contains("denominator")
                        || reason.contains("invalid symbol name")
                );
            }
        }
    }

    #[test]
    fn simplification_preserves_symbol_free_eval(expr in expr_strategy()) {
        let before = EvalRequest {
            expr: expr.clone(),
            decimal_places: 8,
        }.evaluate();

        let simplified = SimplifyRequest { expr }.simplify();
        if let SimplifyResponse::Simplified { expr, .. } = simplified {
            let after = EvalRequest {
                expr,
                decimal_places: 8,
            }.evaluate();
            prop_assert_eq!(serde_json::to_value(after).unwrap(), serde_json::to_value(before).unwrap());
        } else {
            panic!("symbol-free simplification should not fail");
        }
    }

    #[test]
    fn substituting_symbol_with_exact_expr_matches_direct_eval(expr in expr_strategy()) {
        let mut bindings = BTreeMap::new();
        bindings.insert("x".to_owned(), expr.clone());

        let substituted = SubstituteRequest {
            expr: Expr::Symbol {
                name: "x".to_owned(),
            },
            bindings,
            eval_after: false,
            decimal_places: 12,
        }
        .substitute();

        match substituted {
            SubstituteResponse::Substituted { expr: substituted_expr, .. } => {
                let after = EvalRequest {
                    expr: substituted_expr,
                    decimal_places: 8,
                }
                .evaluate();
                let before = EvalRequest {
                    expr,
                    decimal_places: 8,
                }
                .evaluate();
                prop_assert_eq!(serde_json::to_value(after).unwrap(), serde_json::to_value(before).unwrap());
            }
            SubstituteResponse::Error { reason, .. } => {
                prop_assert!(
                    reason.contains("overflow") || reason.contains("denominator must not be zero")
                );
            }
            SubstituteResponse::SubstitutedAndEvaluated { .. }
            | SubstituteResponse::SubstitutedAndApproximated { .. }
            | SubstituteResponse::SubstitutionEvalError { .. } => {
                panic!("unexpected eval_after response when eval_after is false")
            }
        }
    }
}

#[test]
fn schema_has_contract_identity_and_expression_defs() {
    let schema = agent_calc::schema_json();
    assert_eq!(schema["title"], "agent-calc calc1 request");
    assert_eq!(schema["required"], serde_json::json!(["expr"]));
    assert_eq!(schema["$defs"]["Add"]["properties"]["kind"]["const"], "add");
    assert_eq!(schema["$defs"]["Div"]["properties"]["kind"]["const"], "div");
    assert_eq!(schema["$defs"]["Pow"]["properties"]["kind"]["const"], "pow");
    assert_eq!(
        schema["$defs"]["Symbol"]["properties"]["kind"]["const"],
        "symbol"
    );
    assert_eq!(
        schema["$defs"]["Pow"]["properties"]["exponent"]["minimum"],
        -1024
    );
    assert_eq!(
        schema["$defs"]["Pow"]["properties"]["exponent"]["maximum"],
        1024
    );
}

#[test]
fn simplify_schema_has_contract_identity_and_symbol_defs() {
    let schema = agent_calc::simplify_schema_json();
    assert_eq!(schema["title"], "agent-calc calc1 simplify request");
    assert_eq!(
        schema["$defs"]["Symbol"]["properties"]["name"]["pattern"],
        "^[A-Za-z_][A-Za-z0-9_]*$"
    );
}

#[test]
fn substitute_schema_has_contract_identity_and_binding_defs() {
    let schema = agent_calc::substitute_schema_json();
    assert_eq!(schema["title"], "agent-calc calc1 substitute request");
    assert_eq!(schema["required"], serde_json::json!(["expr", "bindings"]));
    assert_eq!(
        schema["properties"]["bindings"]["propertyNames"]["pattern"],
        "^[A-Za-z_][A-Za-z0-9_]*$"
    );
}

#[test]
fn default_decimal_places_are_applied_when_omitted() {
    let request: EvalRequest = serde_json::from_str(
        r#"{
            "expr": {
                "kind": "rational",
                "numerator": "1",
                "denominator": "4"
            }
        }"#,
    )
    .unwrap();

    match request.evaluate() {
        EvalResponse::Solved { decimal, exact, .. } => {
            assert_eq!(decimal, "0.250000000000");
            assert_eq!(exact.display, "1/4");
        }
        other => panic!("expected solved response, got {other:?}"),
    }
}

#[test]
fn invalid_integer_and_zero_division_return_typed_errors() {
    let invalid = EvalRequest {
        expr: Expr::Rational {
            numerator: "2".to_owned(),
            denominator: "not-an-int".to_owned(),
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        invalid,
        EvalResponse::Error { code: ErrorCode::InvalidInteger, reason, .. }
            if reason == "invalid integer `not-an-int`"
    ));

    let divide_by_zero = EvalRequest {
        expr: Expr::Div {
            left: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
            right: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        divide_by_zero,
        EvalResponse::Error { code: ErrorCode::DivisionByZero, reason, .. }
            if reason == "denominator must not be zero"
    ));
}

#[test]
fn eval_rejects_unbound_symbols() {
    let response = EvalRequest {
        expr: Expr::Symbol {
            name: "x".to_owned(),
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        response,
        EvalResponse::Error { code: ErrorCode::UnboundSymbol, reason, .. }
            if reason == "unbound symbol `x`"
    ));
}

#[test]
fn eval_enforces_runtime_validation_limits() {
    let max_decimal_places = EvalRequest {
        expr: Expr::Integer {
            value: "1".to_owned(),
        },
        decimal_places: 128,
    }
    .evaluate();
    assert!(matches!(max_decimal_places, EvalResponse::Solved { .. }));

    let decimal_places = EvalRequest {
        expr: Expr::Integer {
            value: "1".to_owned(),
        },
        decimal_places: 129,
    }
    .evaluate();

    assert!(matches!(
        decimal_places,
        EvalResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "decimal_places must be <= 128"
    ));

    let exponent = EvalRequest {
        expr: Expr::Pow {
            base: Box::new(Expr::Integer {
                value: "2".to_owned(),
            }),
            exponent: 1025,
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        exponent,
        EvalResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "absolute exponent must be <= 1024"
    ));

    let max_exponent = EvalRequest {
        expr: Expr::Pow {
            base: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
            exponent: -1024,
        },
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(max_exponent, EvalResponse::Solved { .. }));

    let empty_integer = EvalRequest {
        expr: Expr::Integer {
            value: String::new(),
        },
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(
        empty_integer,
        EvalResponse::Error { code: ErrorCode::InvalidInteger, reason, .. }
            if reason == "invalid integer ``"
    ));

    let bare_minus = EvalRequest {
        expr: Expr::Integer {
            value: "-".to_owned(),
        },
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(
        bare_minus,
        EvalResponse::Error { code: ErrorCode::InvalidInteger, reason, .. }
            if reason == "invalid integer `-`"
    ));

    let long_invalid_integer = "x".repeat(1025);
    let long_invalid = EvalRequest {
        expr: Expr::Integer {
            value: long_invalid_integer.clone(),
        },
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(
        long_invalid,
        EvalResponse::Error { code: ErrorCode::InvalidInteger, reason, .. }
            if reason == format!("invalid integer `{long_invalid_integer}`")
    ));

    let max_integer_digits = EvalRequest {
        expr: Expr::Integer {
            value: "1".repeat(1024),
        },
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(max_integer_digits, EvalResponse::Solved { .. }));

    let integer_digits = EvalRequest {
        expr: Expr::Integer {
            value: "1".repeat(1025),
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        integer_digits,
        EvalResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "integer digit count must be <= 1024"
    ));
}

#[test]
fn eval_enforces_symbol_length_and_node_count_limits() {
    let max_symbol_length = EvalRequest {
        expr: Expr::Symbol {
            name: "x".repeat(128),
        },
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(
        max_symbol_length,
        EvalResponse::Error { code: ErrorCode::UnboundSymbol, reason, .. }
            if reason == format!("unbound symbol `{}`", "x".repeat(128))
    ));

    let symbol_length = EvalRequest {
        expr: Expr::Symbol {
            name: "x".repeat(129),
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        symbol_length,
        EvalResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "symbol length must be <= 128"
    ));

    fn tree_with_nodes(nodes: usize) -> Expr {
        if nodes == 1 {
            Expr::Integer {
                value: "1".to_owned(),
            }
        } else if nodes.is_multiple_of(2) {
            Expr::Neg {
                value: Box::new(tree_with_nodes(nodes - 1)),
            }
        } else {
            let left_nodes = (nodes - 1) / 2;
            let right_nodes = nodes - 1 - left_nodes;
            Expr::Add {
                left: Box::new(tree_with_nodes(left_nodes)),
                right: Box::new(tree_with_nodes(right_nodes)),
            }
        }
    }

    let max_node_count = EvalRequest {
        expr: tree_with_nodes(4096),
        decimal_places: 3,
    }
    .evaluate();
    assert!(matches!(max_node_count, EvalResponse::Solved { .. }));

    let node_count = EvalRequest {
        expr: tree_with_nodes(4097),
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        node_count,
        EvalResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "expression node count must be <= 4096"
    ));
}

#[test]
fn simplify_enforces_expression_depth_limit() {
    fn neg_chain(wrappers: usize) -> Expr {
        let mut expr = Expr::Integer {
            value: "1".to_owned(),
        };
        for _ in 0..wrappers {
            expr = Expr::Neg {
                value: Box::new(expr),
            };
        }
        expr
    }

    let valid = SimplifyRequest {
        expr: neg_chain(63),
    }
    .simplify();
    assert!(matches!(valid, SimplifyResponse::Simplified { .. }));

    let mut expr = Expr::Integer {
        value: "1".to_owned(),
    };
    for _ in 0..64 {
        expr = Expr::Neg {
            value: Box::new(expr),
        };
    }

    let response = SimplifyRequest { expr }.simplify();
    assert!(matches!(
        response,
        SimplifyResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "expression depth must be <= 64"
    ));

    let binary_left = SimplifyRequest {
        expr: Expr::Add {
            left: Box::new(neg_chain(63)),
            right: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        },
    }
    .simplify();
    assert!(matches!(
        binary_left,
        SimplifyResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "expression depth must be <= 64"
    ));

    let binary_right = SimplifyRequest {
        expr: Expr::Add {
            left: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
            right: Box::new(neg_chain(63)),
        },
    }
    .simplify();
    assert!(matches!(
        binary_right,
        SimplifyResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "expression depth must be <= 64"
    ));

    let pow_base = SimplifyRequest {
        expr: Expr::Pow {
            base: Box::new(neg_chain(63)),
            exponent: 2,
        },
    }
    .simplify();
    assert!(matches!(
        pow_base,
        SimplifyResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "expression depth must be <= 64"
    ));
}

#[test]
fn simplify_applies_first_symbolic_identities() {
    let x = Expr::Symbol {
        name: "x".to_owned(),
    };
    let zero = Expr::Integer {
        value: "0".to_owned(),
    };
    let one = Expr::Integer {
        value: "1".to_owned(),
    };
    let rational_zero = Expr::Rational {
        numerator: "0".to_owned(),
        denominator: "7".to_owned(),
    };
    let rational_one = Expr::Rational {
        numerator: "7".to_owned(),
        denominator: "7".to_owned(),
    };

    let cases = vec![
        (
            Expr::Add {
                left: Box::new(x.clone()),
                right: Box::new(zero.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Add {
                left: Box::new(zero.clone()),
                right: Box::new(x.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Sub {
                left: Box::new(x.clone()),
                right: Box::new(zero.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Add {
                left: Box::new(rational_zero.clone()),
                right: Box::new(x.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Mul {
                left: Box::new(x.clone()),
                right: Box::new(one.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Mul {
                left: Box::new(rational_one.clone()),
                right: Box::new(x.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Mul {
                left: Box::new(one.clone()),
                right: Box::new(x.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Mul {
                left: Box::new(x.clone()),
                right: Box::new(zero.clone()),
            },
            zero.clone(),
        ),
        (
            Expr::Mul {
                left: Box::new(zero.clone()),
                right: Box::new(x.clone()),
            },
            zero.clone(),
        ),
        (
            Expr::Div {
                left: Box::new(x.clone()),
                right: Box::new(one.clone()),
            },
            x.clone(),
        ),
        (
            Expr::Pow {
                base: Box::new(x.clone()),
                exponent: 1,
            },
            x.clone(),
        ),
        (
            Expr::Pow {
                base: Box::new(x.clone()),
                exponent: 0,
            },
            one.clone(),
        ),
        (
            Expr::Neg {
                value: Box::new(Expr::Neg {
                    value: Box::new(x.clone()),
                }),
            },
            x.clone(),
        ),
    ];

    for (expr, expected) in cases {
        let response = SimplifyRequest { expr }.simplify();
        assert!(matches!(
            response,
            SimplifyResponse::Simplified { expr, checks, .. }
                if expr == expected
                    && checks.iter().any(|check| check.name == "symbolic_identities_applied" && check.passed)
        ));
    }
}

#[test]
fn simplify_validates_symbols_and_numeric_leaves() {
    let underscore_symbol = SimplifyRequest {
        expr: Expr::Symbol {
            name: "_x".to_owned(),
        },
    }
    .simplify();

    assert!(matches!(
        underscore_symbol,
        SimplifyResponse::Simplified { expr, .. }
            if expr == (Expr::Symbol {
                name: "_x".to_owned(),
            })
    ));

    let inner_underscore_symbol = SimplifyRequest {
        expr: Expr::Symbol {
            name: "x_y".to_owned(),
        },
    }
    .simplify();

    assert!(matches!(
        inner_underscore_symbol,
        SimplifyResponse::Simplified { expr, .. }
            if expr == (Expr::Symbol {
                name: "x_y".to_owned(),
            })
    ));

    let invalid_symbol = SimplifyRequest {
        expr: Expr::Symbol {
            name: "1x".to_owned(),
        },
    }
    .simplify();

    assert!(matches!(
        invalid_symbol,
        SimplifyResponse::Error { code: ErrorCode::InvalidSymbol, reason, .. }
            if reason == "invalid symbol name `1x`"
    ));

    let invalid_rational_hidden_by_zero = SimplifyRequest {
        expr: Expr::Mul {
            left: Box::new(Expr::Rational {
                numerator: "1".to_owned(),
                denominator: "0".to_owned(),
            }),
            right: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
        },
    }
    .simplify();

    assert!(matches!(
        invalid_rational_hidden_by_zero,
        SimplifyResponse::Error { reason, .. } if reason == "denominator must not be zero"
    ));
}

#[test]
fn substitute_replaces_bound_symbols_and_simplifies() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "x".to_owned(),
        Expr::Rational {
            numerator: "2".to_owned(),
            denominator: "3".to_owned(),
        },
    );

    let response = SubstituteRequest {
        expr: Expr::Mul {
            left: Box::new(Expr::Symbol {
                name: "x".to_owned(),
            }),
            right: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        },
        bindings,
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();

    assert!(matches!(
        response,
        SubstituteResponse::Substituted { expr, checks, .. }
            if expr == (Expr::Rational {
                numerator: "2".to_owned(),
                denominator: "3".to_owned(),
            })
            && checks.iter().any(|check| check.name == "all_symbols_bound" && check.passed)
            && checks.iter().any(|check| check.name == "binding_values_exactly_evaluable" && check.passed)
    ));
}

#[test]
fn substitute_reports_missing_and_invalid_bindings() {
    let missing = SubstituteRequest {
        expr: Expr::Symbol {
            name: "x".to_owned(),
        },
        bindings: BTreeMap::new(),
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();

    assert!(matches!(
        missing,
        SubstituteResponse::Error { reason, .. } if reason == "missing binding for symbol `x`"
    ));

    let mut invalid_name = BTreeMap::new();
    invalid_name.insert(
        "1x".to_owned(),
        Expr::Integer {
            value: "1".to_owned(),
        },
    );
    let invalid_name = SubstituteRequest {
        expr: Expr::Integer {
            value: "1".to_owned(),
        },
        bindings: invalid_name,
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();

    assert!(matches!(
        invalid_name,
        SubstituteResponse::Error { code: ErrorCode::InvalidSymbol, reason, .. }
            if reason == "invalid binding name `1x`"
    ));

    let mut symbolic_binding = BTreeMap::new();
    symbolic_binding.insert(
        "x".to_owned(),
        Expr::Symbol {
            name: "y".to_owned(),
        },
    );
    let symbolic_binding = SubstituteRequest {
        expr: Expr::Symbol {
            name: "x".to_owned(),
        },
        bindings: symbolic_binding,
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();

    assert!(matches!(
        symbolic_binding,
        SubstituteResponse::Error { reason, .. }
            if reason == "binding `x` is not exactly evaluable: unbound symbol `y`"
    ));

    let mut invalid_numeric_binding = BTreeMap::new();
    invalid_numeric_binding.insert(
        "x".to_owned(),
        Expr::Rational {
            numerator: "1".to_owned(),
            denominator: "0".to_owned(),
        },
    );
    let invalid_numeric_binding = SubstituteRequest {
        expr: Expr::Symbol {
            name: "x".to_owned(),
        },
        bindings: invalid_numeric_binding,
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();

    assert!(matches!(
        invalid_numeric_binding,
        SubstituteResponse::Error { reason, .. }
            if reason == "binding `x` is not exactly evaluable: denominator must not be zero"
    ));
}

#[test]
fn substitute_enforces_binding_count_limit() {
    let mut max_bindings = BTreeMap::new();
    for index in 0..1024 {
        max_bindings.insert(
            format!("x{index}"),
            Expr::Integer {
                value: "1".to_owned(),
            },
        );
    }

    let max_response = SubstituteRequest {
        expr: Expr::Integer {
            value: "1".to_owned(),
        },
        bindings: max_bindings,
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();
    assert!(matches!(
        max_response,
        SubstituteResponse::Substituted { .. }
    ));

    let mut bindings = BTreeMap::new();
    for index in 0..1025 {
        bindings.insert(
            format!("x{index}"),
            Expr::Integer {
                value: "1".to_owned(),
            },
        );
    }

    let response = SubstituteRequest {
        expr: Expr::Integer {
            value: "1".to_owned(),
        },
        bindings,
        eval_after: false,
        decimal_places: 12,
    }
    .substitute();

    assert!(matches!(
        response,
        SubstituteResponse::Error { code: ErrorCode::ResourceLimit, reason, .. }
            if reason == "binding count must be <= 1024"
    ));
}

#[test]
fn classifies_stable_error_codes() {
    let cases = [
        ("denominator must not be zero", ErrorCode::DivisionByZero),
        ("complex division by zero", ErrorCode::DivisionByZero),
        ("range must not contain zero", ErrorCode::DivisionByZero),
        (
            "discount factor undefined when 1 + rate is zero",
            ErrorCode::DivisionByZero,
        ),
        ("invalid integer `x`", ErrorCode::InvalidInteger),
        ("invalid symbol name `1x`", ErrorCode::InvalidSymbol),
        ("invalid binding name `1x`", ErrorCode::InvalidSymbol),
        ("invalid polynomial variable `1x`", ErrorCode::InvalidSymbol),
        ("unbound symbol `x`", ErrorCode::UnboundSymbol),
        ("missing binding for symbol `x`", ErrorCode::UnboundSymbol),
        (
            "absolute exponent must be <= 1024",
            ErrorCode::ResourceLimit,
        ),
        ("too many cash_flow periods", ErrorCode::ResourceLimit),
        (
            "periods exceed supported exponent range",
            ErrorCode::ResourceLimit,
        ),
        ("expression node count overflowed", ErrorCode::ResourceLimit),
        (
            "unsupported symbol `y` in solve for `x`",
            ErrorCode::Unsupported,
        ),
        (
            "polynomial degree 3 is not supported",
            ErrorCode::Unsupported,
        ),
        ("nonlinear term involving `x`", ErrorCode::Unsupported),
        ("lower must be less than upper", ErrorCode::InvalidInput),
    ];

    for (reason, expected) in cases {
        assert_eq!(classify_error(reason), expected);
    }
}

#[test]
fn exact_integer_power_supports_positive_zero_and_negative_exponents() {
    let positive = EvalRequest {
        expr: Expr::Pow {
            base: Box::new(Expr::Rational {
                numerator: "2".to_owned(),
                denominator: "3".to_owned(),
            }),
            exponent: 3,
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        positive,
        EvalResponse::Solved { exact, .. } if exact.display == "8/27"
    ));

    let reciprocal = EvalRequest {
        expr: Expr::Pow {
            base: Box::new(Expr::Rational {
                numerator: "2".to_owned(),
                denominator: "3".to_owned(),
            }),
            exponent: -2,
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        reciprocal,
        EvalResponse::Solved { exact, .. } if exact.display == "9/4"
    ));

    let zero_to_zero = EvalRequest {
        expr: Expr::Pow {
            base: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
            exponent: 0,
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        zero_to_zero,
        EvalResponse::Solved { exact, .. } if exact.display == "1"
    ));

    let zero_to_negative = EvalRequest {
        expr: Expr::Pow {
            base: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
            exponent: -1,
        },
        decimal_places: 3,
    }
    .evaluate();

    assert!(matches!(
        zero_to_negative,
        EvalResponse::Error { reason, .. } if reason == "denominator must not be zero"
    ));
}

proptest! {
    #[test]
    fn pow_zero_is_one_for_any_rational(base in leaf_strategy()) {
        let request = EvalRequest {
            expr: Expr::Pow {
                base: Box::new(base),
                exponent: 0,
            },
            decimal_places: 3,
        };

        match request.evaluate() {
            EvalResponse::Solved { exact, .. } => prop_assert_eq!(exact.display, "1"),
            other => panic!("expected solved response, got {other:?}"),
        }
    }

    #[test]
    fn pow_one_preserves_any_rational(base in leaf_strategy()) {
        let expected = EvalRequest {
            expr: base.clone(),
            decimal_places: 3,
        }
        .evaluate();
        let powered = EvalRequest {
            expr: Expr::Pow {
                base: Box::new(base),
                exponent: 1,
            },
            decimal_places: 3,
        }
        .evaluate();

        prop_assert_eq!(serde_json::to_value(powered).unwrap(), serde_json::to_value(expected).unwrap());
    }
}

// ── Cluster 1: evaluate_float must forward the exact value ──────────────────

#[test]
fn evaluate_float_matches_exact_rational_to_f64() {
    let cases = [
        ("3", "4", 0.75_f64),
        ("1", "3", 1.0 / 3.0),
        ("-7", "2", -3.5),
        ("0", "1", 0.0),
        ("22", "7", 22.0 / 7.0),
    ];
    for (n, d, expected) in cases {
        let expr = Expr::Rational {
            numerator: n.to_owned(),
            denominator: d.to_owned(),
        };
        let got = expr.evaluate_float().unwrap();
        assert!(
            (got - expected).abs() < 1e-14,
            "evaluate_float({n}/{d}) = {got}, expected {expected}"
        );
    }
}

// ── Cluster 2: approximate arithmetic (Sub, Div, Neg approx paths) ──────────

fn approx_expr(expr: Expr) -> f64 {
    match (EvalRequest {
        expr,
        decimal_places: 12,
    })
    .evaluate()
    {
        EvalResponse::Approximate { value, .. } => value,
        other => panic!("expected approximate, got {other:?}"),
    }
}

#[test]
fn approx_sub_is_correct() {
    // sin(1) - sin(1) should be 0 (both approx, sub path)
    let expr = Expr::Sub {
        left: Box::new(Expr::Sin {
            value: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        }),
        right: Box::new(Expr::Sin {
            value: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        }),
    };
    assert!((approx_expr(expr) - 0.0).abs() < 1e-14);
}

#[test]
fn approx_sub_mixed_is_correct() {
    // exp(1) - 2 ≈ e - 2
    let expr = Expr::Sub {
        left: Box::new(Expr::Exp {
            value: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        }),
        right: Box::new(Expr::Integer {
            value: "2".to_owned(),
        }),
    };
    let got = approx_expr(expr);
    let expected = std::f64::consts::E - 2.0;
    assert!((got - expected).abs() < 1e-12);
}

#[test]
fn approx_div_is_correct() {
    // exp(1) / 2 ≈ e/2
    let expr = Expr::Div {
        left: Box::new(Expr::Exp {
            value: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        }),
        right: Box::new(Expr::Integer {
            value: "2".to_owned(),
        }),
    };
    let got = approx_expr(expr);
    let expected = std::f64::consts::E / 2.0;
    assert!((got - expected).abs() < 1e-12);
}

#[test]
fn approx_neg_is_correct() {
    // neg(exp(1)) ≈ -e
    let expr = Expr::Neg {
        value: Box::new(Expr::Exp {
            value: Box::new(Expr::Integer {
                value: "1".to_owned(),
            }),
        }),
    };
    let got = approx_expr(expr);
    assert!((got + std::f64::consts::E).abs() < 1e-12);
}

// ── Cluster 3: sqrt approx branch boundary ──────────────────────────────────

#[test]
fn sqrt_of_negative_approx_returns_error() {
    // neg(exp(1)) ≈ -e, so sqrt of that is an error
    let expr = Expr::Sqrt {
        value: Box::new(Expr::Neg {
            value: Box::new(Expr::Exp {
                value: Box::new(Expr::Integer {
                    value: "1".to_owned(),
                }),
            }),
        }),
    };
    let result = EvalRequest {
        expr,
        decimal_places: 12,
    }
    .evaluate();
    assert!(
        matches!(result, EvalResponse::Error { reason, .. } if reason.contains("negative")),
        "expected error for sqrt of approx negative"
    );
}

#[test]
fn sqrt_of_positive_approx_is_correct() {
    // exp(0) = 1.0 (approx), sqrt(1.0) = 1.0
    let expr = Expr::Sqrt {
        value: Box::new(Expr::Exp {
            value: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
        }),
    };
    let got = approx_expr(expr);
    assert!((got - 1.0).abs() < 1e-12);
}

#[test]
fn sqrt_of_approx_zero_is_valid() {
    // sin(0) = 0.0 (approx), sqrt(0.0) should succeed with result 0.0
    // Kills the f < 0.0 → f <= 0.0 mutation: 0.0 is a valid sqrt argument
    let expr = Expr::Sqrt {
        value: Box::new(Expr::Sin {
            value: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
        }),
    };
    let got = approx_expr(expr);
    assert!((got - 0.0).abs() < 1e-12);
}

// ── Cluster 4: log boundary conditions ──────────────────────────────────────

#[test]
fn log_base_one_is_invalid() {
    let expr = Expr::Log {
        base: Box::new(Expr::Integer {
            value: "1".to_owned(),
        }),
        value: Box::new(Expr::Integer {
            value: "5".to_owned(),
        }),
    };
    let result = EvalRequest {
        expr,
        decimal_places: 12,
    }
    .evaluate();
    assert!(
        matches!(result, EvalResponse::Error { .. }),
        "log base 1 should be an error"
    );
}

#[test]
fn log_zero_argument_is_invalid() {
    let expr = Expr::Log {
        base: Box::new(Expr::Integer {
            value: "2".to_owned(),
        }),
        value: Box::new(Expr::Integer {
            value: "0".to_owned(),
        }),
    };
    let result = EvalRequest {
        expr,
        decimal_places: 12,
    }
    .evaluate();
    assert!(
        matches!(result, EvalResponse::Error { .. }),
        "log(2, 0) should be an error"
    );
}

#[test]
fn log_negative_argument_is_invalid() {
    let expr = Expr::Log {
        base: Box::new(Expr::Integer {
            value: "2".to_owned(),
        }),
        value: Box::new(Expr::Integer {
            value: "-1".to_owned(),
        }),
    };
    let result = EvalRequest {
        expr,
        decimal_places: 12,
    }
    .evaluate();
    assert!(
        matches!(result, EvalResponse::Error { .. }),
        "log(2, -1) should be an error"
    );
}

#[test]
fn log_change_of_base_is_correct() {
    // log_10(100) = 2
    let expr = Expr::Log {
        base: Box::new(Expr::Integer {
            value: "10".to_owned(),
        }),
        value: Box::new(Expr::Integer {
            value: "100".to_owned(),
        }),
    };
    let got = approx_expr(expr);
    assert!(
        (got - 2.0).abs() < 1e-12,
        "log_10(100) should be 2, got {got}"
    );
}

#[test]
fn log_change_of_base_non_power_of_ten() {
    // log_3(27) = 3
    let expr = Expr::Log {
        base: Box::new(Expr::Integer {
            value: "3".to_owned(),
        }),
        value: Box::new(Expr::Integer {
            value: "27".to_owned(),
        }),
    };
    let got = approx_expr(expr);
    assert!(
        (got - 3.0).abs() < 1e-12,
        "log_3(27) should be 3, got {got}"
    );
}

// ── Cluster 5: validate_expr_limits depth on Log/Max/Min nodes ───────────────

fn make_nested_max(depth: usize) -> Expr {
    let leaf = Expr::Integer {
        value: "1".to_owned(),
    };
    let mut expr = leaf;
    for _ in 0..depth {
        expr = Expr::Max {
            left: Box::new(expr),
            right: Box::new(Expr::Integer {
                value: "0".to_owned(),
            }),
        };
    }
    expr
}

#[test]
fn validate_depth_limit_enforced_on_max_node() {
    use agent_calc::validate_expr_limits;
    // depth 63 (64 levels including root) should pass
    assert!(validate_expr_limits(&make_nested_max(63)).is_ok());
    // depth 64 (65 levels) should fail
    assert!(validate_expr_limits(&make_nested_max(64)).is_err());
}

#[test]
fn validate_depth_limit_enforced_on_log_node() {
    use agent_calc::validate_expr_limits;
    let mut expr = Expr::Integer {
        value: "1".to_owned(),
    };
    // Wrap in 64 nested Log nodes — should exceed depth limit
    for _ in 0..64 {
        expr = Expr::Log {
            base: Box::new(Expr::Integer {
                value: "2".to_owned(),
            }),
            value: Box::new(expr),
        };
    }
    assert!(validate_expr_limits(&expr).is_err());
}
