#![recursion_limit = "512"]
pub mod assumptions;
pub mod calculus;
pub mod complex;
pub mod finance;
pub mod inequality;
pub mod interval;
pub mod linear;
pub mod matrix;
pub mod optimize;
pub mod polynomial;
pub mod protocol;
pub mod rational;
pub mod solve;
pub mod stats;
pub mod trace;
pub mod units;

pub use assumptions::{
    Assumption, AssumptionDomain, AssumptionsRequest, AssumptionsResponse, ComparisonOp,
    assumptions_schema_json,
};
pub use calculus::{CalculusRequest, CalculusResponse, calculus_schema_json};
pub use finance::{FinanceCheck, FinanceRequest, FinanceResponse, finance_schema_json};
pub use inequality::{
    InequalityInput, InequalityRelation, InequalityRequest, InequalityResponse, SolutionSet,
    inequality_schema_json,
};
pub use matrix::{MatrixInput, MatrixRequest, MatrixResponse, matrix_schema_json};
pub use optimize::{Objective, OptimizeRequest, OptimizeResponse, optimize_schema_json};
pub use polynomial::{
    PolynomialInput, PolynomialRequest, PolynomialResponse, polynomial_schema_json,
};
pub use protocol::{
    Describe, ErrorCode, EvalRequest, EvalResponse, Exactness, Expr, SimplifyRequest,
    SimplifyResponse, SubstituteRequest, SubstituteResponse, classify_error, schema_json,
    simplify_schema_json, substitute_schema_json, validate_decimal_places, validate_expr_limits,
};
pub use rational::Rational;
pub use solve::{EquationInput, SolveRequest, SolveResponse, solve_schema_json};
pub use stats::{StatsRequest, StatsResponse, stats_schema_json};
pub use trace::{TraceOutput, TraceRequest, TraceResponse, TraceStep, trace_schema_json};
pub use units::{Dimension, Quantity, UnitRequest, UnitResponse, units_schema_json};

pub const CONTRACT_VERSION: &str = "calc1/0.1.0";
pub use complex::{ComplexInput, ComplexRequest, ComplexResponse, complex_schema_json};
pub use interval::{IntervalInput, IntervalRequest, IntervalResponse, interval_schema_json};
pub use linear::{
    ConstraintRelation, LinearConstraint, LinearRequest, LinearResponse, ObjectiveDirection,
    VariableBounds, linear_schema_json,
};
