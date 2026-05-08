use agent_calc::{
    Assumption, AssumptionDomain, AssumptionsRequest, AssumptionsResponse, CalculusRequest,
    CalculusResponse, ComplexInput, ComplexRequest, ComplexResponse, ConstraintRelation, Dimension,
    ErrorCode, EvalRequest, EvalResponse, Expr, FinanceRequest, FinanceResponse, InequalityInput,
    InequalityRelation, InequalityRequest, InequalityResponse, IntervalInput, IntervalRequest,
    IntervalResponse, LinearConstraint, LinearRequest, LinearResponse, MatrixInput, MatrixRequest,
    MatrixResponse, Objective, OptimizeRequest, OptimizeResponse, PolynomialInput,
    PolynomialRequest, PolynomialResponse, Quantity, SolveRequest, SolveResponse, StatsRequest,
    StatsResponse, TraceOutput, TraceRequest, TraceResponse, UnitRequest, UnitResponse,
};

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

fn code_of_trace_error(response: TraceResponse) -> ErrorCode {
    match response {
        TraceResponse::Error { code, .. } => code,
        other => panic!("expected trace error, got {other:?}"),
    }
}

#[test]
fn exact_symbolic_domains_emit_error_codes() {
    assert!(matches!(
        SolveRequest::Solve {
            variable: "x".to_owned(),
            equation: agent_calc::EquationInput {
                left: Expr::Mul {
                    left: Box::new(symbol("x")),
                    right: Box::new(symbol("x")),
                },
                right: integer(1),
            },
        }
        .evaluate(),
        SolveResponse::Error {
            code: ErrorCode::Unsupported,
            ..
        }
    ));

    assert!(matches!(
        CalculusRequest::Derivative {
            variable: "1x".to_owned(),
            expr: symbol("x"),
        }
        .evaluate(),
        CalculusResponse::Error {
            code: ErrorCode::InvalidSymbol,
            ..
        }
    ));

    assert!(matches!(
        InequalityRequest::Solve {
            variable: "x".to_owned(),
            inequality: InequalityInput {
                left: symbol("y"),
                relation: InequalityRelation::Lt,
                right: integer(1),
            },
        }
        .evaluate(),
        InequalityResponse::Error {
            code: ErrorCode::Unsupported,
            ..
        }
    ));

    assert!(matches!(
        PolynomialRequest::Normalize {
            polynomial: PolynomialInput {
                variable: "1x".to_owned(),
                coefficients: vec![integer(1)],
            },
        }
        .evaluate(),
        PolynomialResponse::Error {
            code: ErrorCode::InvalidSymbol,
            ..
        }
    ));

    assert!(matches!(
        AssumptionsRequest::Validate {
            assumptions: vec![Assumption::Domain {
                symbol: "1x".to_owned(),
                domain: AssumptionDomain::Positive,
            }],
        }
        .evaluate(),
        AssumptionsResponse::Error {
            code: ErrorCode::InvalidSymbol,
            ..
        }
    ));
}

#[test]
fn exact_numeric_domains_emit_error_codes() {
    assert!(matches!(
        IntervalRequest::Div {
            left: IntervalInput {
                lower: integer(1),
                upper: integer(2),
            },
            right: IntervalInput {
                lower: integer(-1),
                upper: integer(1),
            },
        }
        .evaluate(),
        IntervalResponse::Error {
            code: ErrorCode::DivisionByZero,
            ..
        }
    ));

    assert!(matches!(
        FinanceRequest::DiscountFactor {
            rate: integer(-1),
            period: 1,
            decimal_places: 2,
        }
        .evaluate(),
        FinanceResponse::Error {
            code: ErrorCode::DivisionByZero,
            ..
        }
    ));

    let response = TraceRequest::Eval {
        expr: symbol("x"),
        decimal_places: 2,
    }
    .evaluate();
    assert_eq!(
        code_of_trace_error(response.clone()),
        ErrorCode::UnboundSymbol
    );
    match response {
        TraceResponse::Error { trace, .. } => {
            assert!(matches!(
                trace[0].output,
                TraceOutput::Error {
                    code: ErrorCode::UnboundSymbol,
                    ..
                }
            ));
        }
        other => panic!("expected trace error, got {other:?}"),
    }
}

#[test]
fn approximate_domains_emit_error_codes() {
    assert!(matches!(
        UnitRequest::Add {
            left: Quantity {
                dimension: Dimension::Length,
                value: 1.0,
                unit: "m".to_owned(),
            },
            right: Quantity {
                dimension: Dimension::Time,
                value: 1.0,
                unit: "s".to_owned(),
            },
            to: "m".to_owned(),
        }
        .evaluate(),
        UnitResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    assert!(matches!(
        MatrixRequest::Mul {
            left: MatrixInput {
                rows: 1,
                cols: 2,
                data: vec![1.0, 2.0],
            },
            right: MatrixInput {
                rows: 1,
                cols: 2,
                data: vec![3.0, 4.0],
            },
        }
        .evaluate(),
        MatrixResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    assert!(matches!(
        StatsRequest::NormalQuantile {
            mean: 0.0,
            std_dev: 1.0,
            p: 1.0,
        }
        .evaluate(),
        StatsResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    assert!(matches!(
        OptimizeRequest::Minimize1d {
            objective: Objective::Quadratic {
                a: 1.0,
                b: 0.0,
                c: 0.0,
            },
            lower: 1.0,
            upper: 1.0,
            max_iters: 10,
            abs_tolerance: 1.0e-8,
        }
        .evaluate(),
        OptimizeResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    assert!(matches!(
        LinearRequest::SolveLp {
            direction: agent_calc::ObjectiveDirection::Maximize,
            objective: vec![1.0],
            variables: vec![],
            constraints: vec![LinearConstraint {
                coefficients: vec![1.0, 2.0],
                relation: ConstraintRelation::Le,
                rhs: 1.0,
            }],
        }
        .evaluate(),
        LinearResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    assert!(matches!(
        ComplexRequest::Div {
            left: ComplexInput { re: 1.0, im: 0.0 },
            right: ComplexInput { re: 0.0, im: 0.0 },
        }
        .evaluate(),
        ComplexResponse::Error {
            code: ErrorCode::DivisionByZero,
            ..
        }
    ));
}

#[test]
fn transcendental_nodes_emit_error_codes() {
    // sqrt of negative → InvalidInput
    assert!(matches!(
        EvalRequest {
            expr: Expr::Sqrt {
                value: Box::new(integer(-1))
            },
            decimal_places: 12,
        }
        .evaluate(),
        EvalResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    // ln of non-positive → InvalidInput
    assert!(matches!(
        EvalRequest {
            expr: Expr::Ln {
                value: Box::new(integer(0))
            },
            decimal_places: 12,
        }
        .evaluate(),
        EvalResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    // log with base <= 0 → InvalidInput
    assert!(matches!(
        EvalRequest {
            expr: Expr::Log {
                base: Box::new(integer(-2)),
                value: Box::new(integer(8)),
            },
            decimal_places: 12,
        }
        .evaluate(),
        EvalResponse::Error {
            code: ErrorCode::InvalidInput,
            ..
        }
    ));

    // derivative of abs → Unsupported
    assert!(matches!(
        CalculusRequest::Derivative {
            expr: Expr::Abs {
                value: Box::new(symbol("x"))
            },
            variable: "x".to_owned(),
        }
        .evaluate(),
        CalculusResponse::Error {
            code: ErrorCode::Unsupported,
            ..
        }
    ));

    // transcendental in solve → Unsupported
    assert!(matches!(
        SolveRequest::Solve {
            equation: agent_calc::EquationInput {
                left: Expr::Exp {
                    value: Box::new(symbol("x")),
                },
                right: integer(1),
            },
            variable: "x".to_owned(),
        }
        .evaluate(),
        SolveResponse::Error {
            code: ErrorCode::Unsupported,
            ..
        }
    ));
}
