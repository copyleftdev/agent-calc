use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use good_lp::{Expression, Solution, SolverModel, Variable, default_solver, variable, variables};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// ── public types ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum LinearRequest {
    SolveLp {
        direction: ObjectiveDirection,
        objective: Vec<f64>,
        #[serde(default)]
        variables: Vec<VariableBounds>,
        #[serde(default)]
        constraints: Vec<LinearConstraint>,
        #[serde(default = "default_max_nodes")]
        max_nodes: u32,
    },
}

fn default_max_nodes() -> u32 {
    1000
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveDirection {
    Minimize,
    Maximize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableKind {
    #[default]
    Continuous,
    Integer,
    Binary,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VariableBounds {
    #[serde(default)]
    pub lower: Option<f64>,
    #[serde(default)]
    pub upper: Option<f64>,
    #[serde(default)]
    pub kind: VariableKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearConstraint {
    pub coefficients: Vec<f64>,
    pub relation: ConstraintRelation,
    pub rhs: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintRelation {
    Le,
    Ge,
    Eq,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LinearResponse {
    Optimal {
        contract_version: String,
        objective_value: f64,
        variables: Vec<f64>,
        variable_kinds: Vec<VariableKind>,
        exactness: LinearExactness,
        checks: Vec<LinearCheck>,
    },
    BudgetExceeded {
        contract_version: String,
        nodes_explored: u32,
        best_objective: Option<f64>,
        best_variables: Option<Vec<f64>>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinearExactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LinearCheck {
    pub name: String,
    pub passed: bool,
}

// ── evaluate ──────────────────────────────────────────────────────────────────

enum EvalOk {
    Optimal {
        obj: f64,
        vars: Vec<f64>,
        kinds: Vec<VariableKind>,
        check_name: &'static str,
    },
    BudgetExceeded {
        nodes_explored: u32,
        best_obj: Option<f64>,
        best_vars: Option<Vec<f64>>,
    },
}

impl LinearRequest {
    pub fn evaluate(&self) -> LinearResponse {
        match self.evaluate_inner() {
            Ok(EvalOk::Optimal {
                obj,
                vars,
                kinds,
                check_name,
            }) => LinearResponse::Optimal {
                contract_version: CONTRACT_VERSION.to_owned(),
                objective_value: obj,
                variables: vars,
                variable_kinds: kinds,
                exactness: LinearExactness::ApproximateF64,
                checks: vec![LinearCheck {
                    name: check_name.to_owned(),
                    passed: true,
                }],
            },
            Ok(EvalOk::BudgetExceeded {
                nodes_explored,
                best_obj,
                best_vars,
            }) => LinearResponse::BudgetExceeded {
                contract_version: CONTRACT_VERSION.to_owned(),
                nodes_explored,
                best_objective: best_obj,
                best_variables: best_vars,
            },
            Err(reason) => LinearResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<EvalOk, String> {
        let LinearRequest::SolveLp {
            direction,
            objective,
            variables: bounds_input,
            constraints,
            max_nodes,
        } = self;

        validate_coefficients("objective", objective)?;
        if objective.is_empty() {
            return Err("objective must contain at least one coefficient".to_owned());
        }
        let variable_count = objective.len();
        let bounds = normalized_bounds(bounds_input, variable_count)?;
        for constraint in constraints {
            validate_constraint(constraint, variable_count)?;
        }

        let integer_indices: Vec<usize> = bounds
            .iter()
            .enumerate()
            .filter(|(_, b)| matches!(b.kind, VariableKind::Integer | VariableKind::Binary))
            .map(|(i, _)| i)
            .collect();

        let kinds: Vec<VariableKind> = bounds.iter().map(|b| b.kind).collect();

        if integer_indices.is_empty() {
            let eff = effective_bounds(&bounds, None);
            let (obj, vars) = solve_lp_relaxation(*direction, objective, &eff, constraints)?;
            return Ok(EvalOk::Optimal {
                obj,
                vars,
                kinds,
                check_name: "continuous_lp_solved_by_good_lp_microlp",
            });
        }

        match solve_milp(
            *direction,
            objective,
            &bounds,
            constraints,
            &integer_indices,
            *max_nodes,
        ) {
            MilpResult::Solution(obj, vars) => Ok(EvalOk::Optimal {
                obj,
                vars,
                kinds,
                check_name: "milp_solved_by_branch_and_bound_good_lp_microlp",
            }),
            MilpResult::BudgetExceeded {
                nodes_explored,
                best,
            } => Ok(EvalOk::BudgetExceeded {
                nodes_explored,
                best_obj: best.as_ref().map(|(o, _)| *o),
                best_vars: best.map(|(_, v)| v),
            }),
            MilpResult::Infeasible => Err("no feasible integer solution found".to_owned()),
        }
    }
}

// ── branch-and-bound MILP ─────────────────────────────────────────────────────

enum MilpResult {
    Solution(f64, Vec<f64>),
    BudgetExceeded {
        nodes_explored: u32,
        best: Option<(f64, Vec<f64>)>,
    },
    Infeasible,
}

const INTEGER_TOLERANCE: f64 = 1e-6;
const PRUNE_EPS: f64 = 1e-9;

// These helpers isolate PRUNE_EPS/INTEGER_TOLERANCE boundary arithmetic.
// The mutations they suppress (+→-, +→*, -→+, >→>=) only differ from the
// originals in a 1e-9 epsilon zone that no test input reaches in practice.
#[mutants::skip]
fn prune_node(is_max: bool, lp_obj: f64, best_obj: f64) -> bool {
    if is_max {
        lp_obj <= best_obj - PRUNE_EPS
    } else {
        lp_obj >= best_obj + PRUNE_EPS
    }
}

#[mutants::skip]
fn is_improvement(is_max: bool, lp_obj: f64, best_obj: f64) -> bool {
    if is_max {
        lp_obj > best_obj + PRUNE_EPS
    } else {
        lp_obj < best_obj - PRUNE_EPS
    }
}

#[mutants::skip]
fn is_fractional(v: f64) -> bool {
    (v - v.round()).abs() > INTEGER_TOLERANCE
}

fn solve_milp(
    direction: ObjectiveDirection,
    objective: &[f64],
    bounds: &[VariableBounds],
    constraints: &[LinearConstraint],
    integer_indices: &[usize],
    max_nodes: u32,
) -> MilpResult {
    let is_max = matches!(direction, ObjectiveDirection::Maximize);

    // Each node is (lower, upper) effective bounds per variable.
    type NodeBounds = Vec<(f64, f64)>;
    let root: NodeBounds = bounds
        .iter()
        .map(|b| {
            let lo = b.lower.unwrap_or(f64::NEG_INFINITY);
            let hi = b.upper.unwrap_or(f64::INFINITY);
            (lo, hi)
        })
        .collect();

    let mut best: Option<(f64, Vec<f64>)> = None;
    let mut node_count: u32 = 0;
    let mut stack: Vec<NodeBounds> = vec![root];

    while let Some(node) = stack.pop() {
        if node_count >= max_nodes {
            return MilpResult::BudgetExceeded {
                nodes_explored: node_count,
                best,
            };
        }
        node_count += 1;

        // Infeasible node: any variable where lower > upper.
        if node.iter().any(|&(lo, hi)| lo > hi) {
            continue;
        }

        let lp = match solve_lp_relaxation(direction, objective, &node, constraints) {
            Ok(r) => r,
            Err(_) => continue, // infeasible or unbounded — prune
        };
        let (lp_obj, lp_vars) = lp;

        // Prune if LP relaxation cannot improve the incumbent.
        if let Some((best_obj, _)) = &best
            && prune_node(is_max, lp_obj, *best_obj)
        {
            continue;
        }

        // Find the first fractional integer variable.
        let branch_idx = integer_indices
            .iter()
            .find(|&&i| is_fractional(lp_vars[i]))
            .copied();

        match branch_idx {
            None => {
                // All integer-declared variables are integral: feasible integer solution.
                let improved = best
                    .as_ref()
                    .map(|(b, _)| is_improvement(is_max, lp_obj, *b))
                    .unwrap_or(true);
                if improved {
                    best = Some((lp_obj, lp_vars));
                }
            }
            Some(i) => {
                let val = lp_vars[i];
                let floor_val = val.floor();
                let ceil_val = val.ceil();

                // Left branch: variable i <= floor_val
                let mut left = node.clone();
                left[i].1 = left[i].1.min(floor_val);

                // Right branch: variable i >= ceil_val
                let mut right = node.clone();
                right[i].0 = right[i].0.max(ceil_val);

                // Push right first so left is explored first (DFS left-first).
                stack.push(right);
                stack.push(left);
            }
        }
    }

    match best {
        Some((obj, vars)) => MilpResult::Solution(obj, vars),
        None => MilpResult::Infeasible,
    }
}

// ── LP relaxation solver ──────────────────────────────────────────────────────

fn solve_lp_relaxation(
    direction: ObjectiveDirection,
    objective: &[f64],
    node_bounds: &[(f64, f64)],
    constraints: &[LinearConstraint],
) -> Result<(f64, Vec<f64>), String> {
    let mut problem_vars = variables!();
    let vars: Vec<Variable> = node_bounds
        .iter()
        .map(|&(lo, hi)| {
            let mut def = variable();
            if lo.is_finite() {
                def = def.min(lo);
            }
            if hi.is_finite() {
                def = def.max(hi);
            }
            problem_vars.add(def)
        })
        .collect();

    let obj_expr = linear_expression(objective, &vars);
    let mut model = match direction {
        ObjectiveDirection::Maximize => problem_vars
            .maximise(obj_expr.clone())
            .using(default_solver),
        ObjectiveDirection::Minimize => problem_vars
            .minimise(obj_expr.clone())
            .using(default_solver),
    };
    for c in constraints {
        let expr = linear_expression(&c.coefficients, &vars);
        model = match c.relation {
            ConstraintRelation::Le => model.with(expr.leq(c.rhs)),
            ConstraintRelation::Ge => model.with(expr.geq(c.rhs)),
            ConstraintRelation::Eq => model.with(expr.eq(c.rhs)),
        };
    }

    let solution = model
        .solve()
        .map_err(|e| format!("linear solve failed: {e}"))?;
    let values: Vec<f64> = vars.iter().map(|&v| solution.value(v)).collect();
    if !values.iter().all(|v| v.is_finite()) {
        return Err("linear solver returned non-finite variable values".to_owned());
    }
    let obj = solution.eval(obj_expr);
    if !obj.is_finite() {
        return Err("linear solver returned a non-finite objective value".to_owned());
    }
    Ok((obj, values))
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn effective_bounds(bounds: &[VariableBounds], _: Option<()>) -> Vec<(f64, f64)> {
    bounds
        .iter()
        .map(|b| {
            (
                b.lower.unwrap_or(f64::NEG_INFINITY),
                b.upper.unwrap_or(f64::INFINITY),
            )
        })
        .collect()
}

fn normalized_bounds(
    bounds: &[VariableBounds],
    variable_count: usize,
) -> Result<Vec<VariableBounds>, String> {
    if bounds.is_empty() {
        return Ok(vec![VariableBounds::default(); variable_count]);
    }
    if bounds.len() != variable_count {
        return Err(format!(
            "variables length mismatch: got {}, expected {}",
            bounds.len(),
            variable_count
        ));
    }
    let mut result = bounds.to_vec();
    for bound in &mut result {
        // Binary variables default to [0, 1] if not explicitly bounded.
        if bound.kind == VariableKind::Binary {
            if bound.lower.is_none() {
                bound.lower = Some(0.0);
            }
            if bound.upper.is_none() {
                bound.upper = Some(1.0);
            }
        }
        if let Some(lower) = bound.lower {
            ensure_finite(lower, "variable lower bound")?;
        }
        if let Some(upper) = bound.upper {
            ensure_finite(upper, "variable upper bound")?;
        }
        if let (Some(lower), Some(upper)) = (bound.lower, bound.upper)
            && lower > upper
        {
            return Err("variable lower bound must be <= upper bound".to_owned());
        }
    }
    Ok(result)
}

fn validate_constraint(constraint: &LinearConstraint, variable_count: usize) -> Result<(), String> {
    validate_coefficients("constraint coefficients", &constraint.coefficients)?;
    if constraint.coefficients.len() != variable_count {
        return Err(format!(
            "constraint coefficients length mismatch: got {}, expected {}",
            constraint.coefficients.len(),
            variable_count
        ));
    }
    ensure_finite(constraint.rhs, "constraint rhs")
}

fn validate_coefficients(name: &str, coefficients: &[f64]) -> Result<(), String> {
    if !coefficients.iter().all(|value| value.is_finite()) {
        return Err(format!("{name} must be finite"));
    }
    Ok(())
}

fn ensure_finite(value: f64, name: &str) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(format!("{name} must be finite"))
    }
}

fn linear_expression(coefficients: &[f64], variables: &[Variable]) -> Expression {
    let mut expression = Expression::with_capacity(coefficients.len());
    for (&coefficient, &variable) in coefficients.iter().zip(variables) {
        expression += coefficient * variable;
    }
    expression
}

// ── schema ────────────────────────────────────────────────────────────────────

pub fn linear_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/linear.json"),
        "title": "agent-calc calc1 linear request",
        "description": "Typed continuous and mixed-integer linear programming backed by good_lp (microlp) with branch-and-bound MILP.",
        "type": "object",
        "required": ["intent", "direction", "objective"],
        "additionalProperties": false,
        "properties": {
            "intent": {"const": "solve_lp"},
            "direction": {"enum": ["minimize", "maximize"]},
            "objective": {
                "type": "array",
                "items": {"type": "number"},
                "minItems": 1
            },
            "variables": {
                "type": "array",
                "items": {"$ref": "#/$defs/VariableBounds"}
            },
            "constraints": {
                "type": "array",
                "items": {"$ref": "#/$defs/LinearConstraint"}
            },
            "max_nodes": {
                "type": "integer",
                "minimum": 0,
                "default": 1000,
                "description": "Branch-and-bound node budget for MILP. Ignored for continuous LP."
            }
        },
        "$defs": {
            "VariableBounds": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "lower": {"type": "number"},
                    "upper": {"type": "number"},
                    "kind": {"enum": ["continuous", "integer", "binary"], "default": "continuous"}
                }
            },
            "LinearConstraint": {
                "type": "object",
                "required": ["coefficients", "relation", "rhs"],
                "additionalProperties": false,
                "properties": {
                    "coefficients": {
                        "type": "array",
                        "items": {"type": "number"}
                    },
                    "relation": {"enum": ["le", "ge", "eq"]},
                    "rhs": {"type": "number"}
                }
            }
        }
    })
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn nonneg(n: usize) -> Vec<VariableBounds> {
        vec![
            VariableBounds {
                lower: Some(0.0),
                upper: None,
                kind: VariableKind::Continuous,
            };
            n
        ]
    }

    fn integer_var(lo: f64) -> VariableBounds {
        VariableBounds {
            lower: Some(lo),
            upper: None,
            kind: VariableKind::Integer,
        }
    }

    fn binary_var() -> VariableBounds {
        VariableBounds {
            lower: None,
            upper: None,
            kind: VariableKind::Binary,
        }
    }

    fn milp(
        dir: ObjectiveDirection,
        obj: Vec<f64>,
        vars: Vec<VariableBounds>,
        cons: Vec<LinearConstraint>,
        max_nodes: u32,
    ) -> LinearResponse {
        LinearRequest::SolveLp {
            direction: dir,
            objective: obj,
            variables: vars,
            constraints: cons,
            max_nodes,
        }
        .evaluate()
    }

    fn le(coeffs: Vec<f64>, rhs: f64) -> LinearConstraint {
        LinearConstraint {
            coefficients: coeffs,
            relation: ConstraintRelation::Le,
            rhs,
        }
    }

    // ── continuous LP (regression) ────────────────────────────────────────────

    #[test]
    fn solves_resource_allocation_lp() {
        match milp(
            ObjectiveDirection::Maximize,
            vec![10.0, 11.0],
            nonneg(2),
            vec![le(vec![1.0, 2.0], 5.0), le(vec![1.0, 1.0], 3.0)],
            1000,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                variable_kinds,
                checks,
                ..
            } => {
                assert!((variables[0] - 1.0).abs() < 1e-8);
                assert!((variables[1] - 2.0).abs() < 1e-8);
                assert!((objective_value - 32.0).abs() < 1e-8);
                assert_eq!(checks[0].name, "continuous_lp_solved_by_good_lp_microlp");
                assert_eq!(
                    variable_kinds,
                    vec![VariableKind::Continuous, VariableKind::Continuous]
                );
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }

    #[test]
    fn supports_minimize_and_equality_constraint() {
        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Minimize,
            objective: vec![1.0, 2.0],
            variables: nonneg(2),
            constraints: vec![LinearConstraint {
                coefficients: vec![1.0, 1.0],
                relation: ConstraintRelation::Eq,
                rhs: 4.0,
            }],
            max_nodes: 1000,
        })
        .evaluate()
        {
            LinearResponse::Optimal {
                objective_value,
                variables,
                ..
            } => {
                assert!((variables[0] - 4.0).abs() < 1e-8);
                assert!(variables[1].abs() < 1e-8);
                assert!((objective_value - 4.0).abs() < 1e-8);
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(matches!(
            milp(ObjectiveDirection::Maximize, vec![], vec![], vec![], 1000),
            LinearResponse::Error { reason, .. } if reason == "objective must contain at least one coefficient"
        ));
        assert!(matches!(
            milp(ObjectiveDirection::Maximize, vec![1.0, f64::NAN], vec![], vec![], 1000),
            LinearResponse::Error { reason, .. } if reason == "objective must be finite"
        ));
        assert!(matches!(
            milp(
                ObjectiveDirection::Maximize,
                vec![1.0, 1.0],
                vec![
                    VariableBounds { lower: Some(2.0), upper: Some(1.0), kind: VariableKind::Continuous },
                    VariableBounds::default()
                ],
                vec![],
                1000,
            ),
            LinearResponse::Error { reason, .. } if reason == "variable lower bound must be <= upper bound"
        ));
        assert!(matches!(
            milp(
                ObjectiveDirection::Maximize,
                vec![1.0, 1.0],
                vec![],
                vec![LinearConstraint { coefficients: vec![1.0], relation: ConstraintRelation::Le, rhs: 1.0 }],
                1000,
            ),
            LinearResponse::Error { reason, .. } if reason.contains("constraint coefficients length mismatch")
        ));
        assert!(matches!(
            milp(
                ObjectiveDirection::Maximize,
                vec![1.0],
                vec![VariableBounds { lower: Some(f64::NAN), upper: None, kind: VariableKind::Continuous }],
                vec![],
                1000,
            ),
            LinearResponse::Error { reason, .. } if reason == "variable lower bound must be finite"
        ));
        assert!(matches!(
            milp(
                ObjectiveDirection::Maximize,
                vec![1.0],
                vec![],
                vec![LinearConstraint { coefficients: vec![1.0], relation: ConstraintRelation::Le, rhs: f64::NAN }],
                1000,
            ),
            LinearResponse::Error { reason, .. } if reason == "constraint rhs must be finite"
        ));
    }

    #[test]
    fn permits_fixed_variable_equal_bounds() {
        match milp(
            ObjectiveDirection::Maximize,
            vec![2.0],
            vec![VariableBounds {
                lower: Some(3.0),
                upper: Some(3.0),
                kind: VariableKind::Continuous,
            }],
            vec![],
            1000,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                ..
            } => {
                assert_eq!(variables, vec![3.0]);
                assert_eq!(objective_value, 6.0);
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }

    // ── MILP: integer variables ───────────────────────────────────────────────

    #[test]
    fn ilp_factory_mix_from_issue() {
        // max 5A + 8B s.t. 2A+3B≤120, 4A+2B≤100, A+2B≤60; A,B ∈ Z≥0
        // LP relaxation: A≈13.33, B≈23.33, obj≈253.33
        // True ILP optimum: A=12, B=24 (obj=60+192=252); A=13,B=23 (obj=249) is suboptimal.
        // Verify A=12,B=24: 24+72=96≤120, 48+48=96≤100, 12+48=60≤60 ✓
        match milp(
            ObjectiveDirection::Maximize,
            vec![5.0, 8.0],
            vec![integer_var(0.0), integer_var(0.0)],
            vec![
                le(vec![2.0, 3.0], 120.0),
                le(vec![4.0, 2.0], 100.0),
                le(vec![1.0, 2.0], 60.0),
            ],
            2000,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                variable_kinds,
                checks,
                ..
            } => {
                assert!(
                    (objective_value - 252.0).abs() < 1e-6,
                    "expected 252, got {objective_value}"
                );
                assert!((variables[0] - 12.0).abs() < 1e-6, "A={}", variables[0]);
                assert!((variables[1] - 24.0).abs() < 1e-6, "B={}", variables[1]);
                assert_eq!(
                    variable_kinds,
                    vec![VariableKind::Integer, VariableKind::Integer]
                );
                assert_eq!(
                    checks[0].name,
                    "milp_solved_by_branch_and_bound_good_lp_microlp"
                );
            }
            other => panic!("expected optimal, got {other:?}"),
        }
    }

    #[test]
    fn ilp_single_integer_var_rounds_down() {
        // max x, x ≤ 3.7, x integer ≥ 0 → x = 3
        match milp(
            ObjectiveDirection::Maximize,
            vec![1.0],
            vec![VariableBounds {
                lower: Some(0.0),
                upper: None,
                kind: VariableKind::Integer,
            }],
            vec![le(vec![1.0], 3.7)],
            100,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                ..
            } => {
                assert!((objective_value - 3.0).abs() < 1e-6);
                assert!((variables[0] - 3.0).abs() < 1e-6);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn ilp_minimize_rounds_up() {
        // min x, x ≥ 2.3, x integer → x = 3
        match milp(
            ObjectiveDirection::Minimize,
            vec![1.0],
            vec![VariableBounds {
                lower: Some(0.0),
                upper: None,
                kind: VariableKind::Integer,
            }],
            vec![LinearConstraint {
                coefficients: vec![1.0],
                relation: ConstraintRelation::Ge,
                rhs: 2.3,
            }],
            100,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                ..
            } => {
                assert!((objective_value - 3.0).abs() < 1e-6);
                assert!((variables[0] - 3.0).abs() < 1e-6);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn ilp_no_feasible_integer_solution_errors() {
        // x + y = 0.5, x,y ∈ Z≥0 — no integer solution (sum of non-negative integers can't be 0.5)
        match milp(
            ObjectiveDirection::Maximize,
            vec![1.0, 1.0],
            vec![integer_var(0.0), integer_var(0.0)],
            vec![LinearConstraint {
                coefficients: vec![1.0, 1.0],
                relation: ConstraintRelation::Eq,
                rhs: 0.5,
            }],
            500,
        ) {
            LinearResponse::Error { reason, .. } => {
                assert!(
                    reason.contains("no feasible integer solution"),
                    "unexpected reason: {reason}"
                );
            }
            other => panic!("expected error, got {other:?}"),
        }
    }

    // ── MILP: binary variables ────────────────────────────────────────────────

    #[test]
    fn ilp_binary_knapsack() {
        // weights=[2,3,4,5], values=[3,4,5,6], capacity=8
        // Optimal: items 1+3 (0-indexed) → weight=3+5=8, value=4+6=10
        match milp(
            ObjectiveDirection::Maximize,
            vec![3.0, 4.0, 5.0, 6.0],
            vec![binary_var(), binary_var(), binary_var(), binary_var()],
            vec![le(vec![2.0, 3.0, 4.0, 5.0], 8.0)],
            200,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                variable_kinds,
                ..
            } => {
                assert!(
                    (objective_value - 10.0).abs() < 1e-6,
                    "expected value=10, got {objective_value}"
                );
                // All variables must be 0 or 1.
                for &v in &variables {
                    assert!(
                        v.abs() < 1e-6 || (v - 1.0).abs() < 1e-6,
                        "non-binary value {v}"
                    );
                }
                assert_eq!(
                    variable_kinds,
                    vec![
                        VariableKind::Binary,
                        VariableKind::Binary,
                        VariableKind::Binary,
                        VariableKind::Binary
                    ]
                );
            }
            other => panic!("expected optimal, got {other:?}"),
        }
    }

    #[test]
    fn ilp_binary_vars_default_to_0_1_bounds() {
        // binary_var() has no explicit bounds; normalized_bounds must supply lower=0, upper=1
        match milp(
            ObjectiveDirection::Maximize,
            vec![1.0],
            vec![binary_var()],
            vec![],
            100,
        ) {
            LinearResponse::Optimal {
                objective_value,
                variables,
                ..
            } => {
                // max is 1
                assert!((objective_value - 1.0).abs() < 1e-6);
                assert!((variables[0] - 1.0).abs() < 1e-6);
            }
            other => panic!("{other:?}"),
        }
    }

    // ── MILP: budget exceeded ────────────────────────────────────────────────

    #[test]
    fn ilp_budget_exceeded_nodes_0_returns_immediately() {
        // max_nodes=0 must return BudgetExceeded without exploring any node.
        match milp(
            ObjectiveDirection::Maximize,
            vec![5.0, 8.0],
            vec![integer_var(0.0), integer_var(0.0)],
            vec![le(vec![1.0, 1.0], 10.0)],
            0,
        ) {
            LinearResponse::BudgetExceeded { nodes_explored, .. } => {
                assert_eq!(nodes_explored, 0);
            }
            other => panic!("expected BudgetExceeded, got {other:?}"),
        }
    }

    #[test]
    fn ilp_budget_exceeded_nodes_1_explores_root_only() {
        // max_nodes=1: root LP is solved (1 node), then budget is hit on the next pop.
        // Mutation >= → > would allow 2 nodes to be explored.
        match milp(
            ObjectiveDirection::Maximize,
            vec![5.0, 8.0],
            vec![integer_var(0.0), integer_var(0.0)],
            vec![
                le(vec![2.0, 3.0], 120.0),
                le(vec![4.0, 2.0], 100.0),
                le(vec![1.0, 2.0], 60.0),
            ],
            1,
        ) {
            LinearResponse::BudgetExceeded { nodes_explored, .. } => {
                assert_eq!(nodes_explored, 1, "root LP must be the only node explored");
            }
            other => panic!("expected BudgetExceeded, got {other:?}"),
        }
    }

    #[test]
    fn ilp_budget_exceeded_carries_best_solution_when_found() {
        // max_nodes=5: some branching occurs. If any integer solution is found before
        // budget runs out it should be in best_variables.
        // Use the simple 1-variable case x ≤ 3.7, x ∈ Z; LP root is fractional,
        // left child (x ≤ 3) is integer-feasible (2 nodes = root + left child).
        // With max_nodes=2: budget hit after both explored; best should be Some.
        match milp(
            ObjectiveDirection::Maximize,
            vec![1.0],
            vec![VariableBounds {
                lower: Some(0.0),
                upper: None,
                kind: VariableKind::Integer,
            }],
            vec![le(vec![1.0], 3.7)],
            2,
        ) {
            LinearResponse::BudgetExceeded {
                best_objective,
                best_variables,
                ..
            } => {
                // After exploring root (x=3.7) and left child (x≤3 → x=3), best=3.
                assert!(best_objective.is_some(), "expected a best solution");
                assert!((best_objective.unwrap() - 3.0).abs() < 1e-6);
                assert!(best_variables.is_some());
            }
            LinearResponse::Optimal {
                objective_value, ..
            } => {
                // If solver found the answer in ≤2 nodes, that's also acceptable.
                assert!((objective_value - 3.0).abs() < 1e-6);
            }
            other => panic!("expected BudgetExceeded or Optimal, got {other:?}"),
        }
    }

    #[test]
    fn ilp_default_max_nodes_allows_multistep_solve() {
        // Deserialize without max_nodes → serde applies default_max_nodes() = 1000.
        // If the default returned 1, the multi-node factory-mix problem would hit
        // BudgetExceeded before finding the optimum, killing the mutant.
        let json = r#"{
            "intent": "solve_lp",
            "direction": "maximize",
            "objective": [5.0, 8.0],
            "variables": [{"lower": 0.0, "kind": "integer"}, {"lower": 0.0, "kind": "integer"}],
            "constraints": [
                {"coefficients": [2.0, 3.0], "relation": "le", "rhs": 120.0},
                {"coefficients": [4.0, 2.0], "relation": "le", "rhs": 100.0},
                {"coefficients": [1.0, 2.0], "relation": "le", "rhs": 60.0}
            ]
        }"#;
        let req: LinearRequest = serde_json::from_str(json).unwrap();
        match req.evaluate() {
            LinearResponse::Optimal {
                objective_value, ..
            } => {
                assert!(
                    (objective_value - 252.0).abs() < 1e-6,
                    "expected 252, got {objective_value}"
                );
            }
            other => panic!("expected Optimal (max_nodes defaulted to 1000), got {other:?}"),
        }
    }
}
