use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use good_lp::{Expression, Solution, SolverModel, Variable, default_solver, variable, variables};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveDirection {
    Minimize,
    Maximize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VariableBounds {
    #[serde(default)]
    pub lower: Option<f64>,
    #[serde(default)]
    pub upper: Option<f64>,
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
        exactness: LinearExactness,
        checks: Vec<LinearCheck>,
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

impl LinearRequest {
    pub fn evaluate(&self) -> LinearResponse {
        match self.evaluate_inner() {
            Ok((objective_value, values)) => LinearResponse::Optimal {
                contract_version: CONTRACT_VERSION.to_owned(),
                objective_value,
                variables: values,
                exactness: LinearExactness::ApproximateF64,
                checks: vec![LinearCheck {
                    name: "continuous_lp_solved_by_good_lp_microlp".to_owned(),
                    passed: true,
                }],
            },
            Err(reason) => LinearResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<(f64, Vec<f64>), String> {
        match self {
            LinearRequest::SolveLp {
                direction,
                objective,
                variables: bounds,
                constraints,
            } => {
                validate_coefficients("objective", objective)?;
                if objective.is_empty() {
                    return Err("objective must contain at least one coefficient".to_owned());
                }
                let variable_count = objective.len();
                let bounds = normalized_bounds(bounds, variable_count)?;
                for constraint in constraints {
                    validate_constraint(constraint, variable_count)?;
                }

                let mut problem_vars = variables!();
                let variables: Vec<Variable> = bounds
                    .iter()
                    .map(|bound| {
                        let mut definition = variable();
                        if let Some(lower) = bound.lower {
                            definition = definition.min(lower);
                        }
                        if let Some(upper) = bound.upper {
                            definition = definition.max(upper);
                        }
                        problem_vars.add(definition)
                    })
                    .collect();

                let objective_expr = linear_expression(objective, &variables);
                let mut model = match direction {
                    ObjectiveDirection::Maximize => problem_vars
                        .maximise(objective_expr.clone())
                        .using(default_solver),
                    ObjectiveDirection::Minimize => problem_vars
                        .minimise(objective_expr.clone())
                        .using(default_solver),
                };
                for constraint in constraints {
                    let expr = linear_expression(&constraint.coefficients, &variables);
                    model = match constraint.relation {
                        ConstraintRelation::Le => model.with(expr.leq(constraint.rhs)),
                        ConstraintRelation::Ge => model.with(expr.geq(constraint.rhs)),
                        ConstraintRelation::Eq => model.with(expr.eq(constraint.rhs)),
                    };
                }

                let solution = model
                    .solve()
                    .map_err(|e| format!("linear solve failed: {e}"))?;
                let values: Vec<f64> = variables.iter().map(|&v| solution.value(v)).collect();
                if !values.iter().all(|value| value.is_finite()) {
                    return Err("linear solver returned non-finite variable values".to_owned());
                }
                let objective_value = solution.eval(objective_expr);
                if !objective_value.is_finite() {
                    return Err("linear solver returned a non-finite objective value".to_owned());
                }
                Ok((objective_value, values))
            }
        }
    }
}

pub fn linear_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/linear.json"),
        "title": "agent-calc calc1 linear request",
        "description": "Typed continuous linear programming request backed by good_lp with the microlp solver.",
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
            }
        },
        "$defs": {
            "VariableBounds": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "lower": {"type": "number"},
                    "upper": {"type": "number"}
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
    for bound in bounds {
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
    Ok(bounds.to_vec())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn nonnegative_vars(n: usize) -> Vec<VariableBounds> {
        vec![
            VariableBounds {
                lower: Some(0.0),
                upper: None,
            };
            n
        ]
    }

    #[test]
    fn solves_resource_allocation_lp() {
        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Maximize,
            objective: vec![10.0, 11.0],
            variables: nonnegative_vars(2),
            constraints: vec![
                LinearConstraint {
                    coefficients: vec![1.0, 2.0],
                    relation: ConstraintRelation::Le,
                    rhs: 5.0,
                },
                LinearConstraint {
                    coefficients: vec![1.0, 1.0],
                    relation: ConstraintRelation::Le,
                    rhs: 3.0,
                },
            ],
        })
        .evaluate()
        {
            LinearResponse::Optimal {
                objective_value,
                variables,
                checks,
                ..
            } => {
                assert!((variables[0] - 1.0).abs() < 1e-8);
                assert!((variables[1] - 2.0).abs() < 1e-8);
                assert!((objective_value - 32.0).abs() < 1e-8);
                assert_eq!(checks[0].name, "continuous_lp_solved_by_good_lp_microlp");
            }
            other => panic!("expected optimal response, got {other:?}"),
        }
    }

    #[test]
    fn supports_minimize_and_equality_constraint() {
        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Minimize,
            objective: vec![1.0, 2.0],
            variables: nonnegative_vars(2),
            constraints: vec![LinearConstraint {
                coefficients: vec![1.0, 1.0],
                relation: ConstraintRelation::Eq,
                rhs: 4.0,
            }],
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
            (LinearRequest::SolveLp {
                direction: ObjectiveDirection::Maximize,
                objective: vec![],
                variables: vec![],
                constraints: vec![],
            }).evaluate(),
            LinearResponse::Error { reason, .. } if reason == "objective must contain at least one coefficient"
        ));
        assert!(matches!(
            (LinearRequest::SolveLp {
                direction: ObjectiveDirection::Maximize,
                objective: vec![1.0, f64::NAN],
                variables: vec![],
                constraints: vec![],
            }).evaluate(),
            LinearResponse::Error { reason, .. } if reason == "objective must be finite"
        ));
        assert!(matches!(
            (LinearRequest::SolveLp {
                direction: ObjectiveDirection::Maximize,
                objective: vec![1.0, 1.0],
                variables: vec![VariableBounds { lower: Some(2.0), upper: Some(1.0) }, VariableBounds::default()],
                constraints: vec![],
            }).evaluate(),
            LinearResponse::Error { reason, .. } if reason == "variable lower bound must be <= upper bound"
        ));
        assert!(matches!(
            (LinearRequest::SolveLp {
                direction: ObjectiveDirection::Maximize,
                objective: vec![1.0, 1.0],
                variables: vec![],
                constraints: vec![LinearConstraint {
                    coefficients: vec![1.0],
                    relation: ConstraintRelation::Le,
                    rhs: 1.0,
                }],
            }).evaluate(),
            LinearResponse::Error { reason, .. } if reason.contains("constraint coefficients length mismatch")
        ));
        assert!(matches!(
            (LinearRequest::SolveLp {
                direction: ObjectiveDirection::Maximize,
                objective: vec![1.0],
                variables: vec![VariableBounds { lower: Some(f64::NAN), upper: None }],
                constraints: vec![],
            }).evaluate(),
            LinearResponse::Error { reason, .. } if reason == "variable lower bound must be finite"
        ));
        assert!(matches!(
            (LinearRequest::SolveLp {
                direction: ObjectiveDirection::Maximize,
                objective: vec![1.0],
                variables: vec![],
                constraints: vec![LinearConstraint {
                    coefficients: vec![1.0],
                    relation: ConstraintRelation::Le,
                    rhs: f64::NAN,
                }],
            }).evaluate(),
            LinearResponse::Error { reason, .. } if reason == "constraint rhs must be finite"
        ));
    }

    #[test]
    fn permits_fixed_variable_equal_bounds() {
        match (LinearRequest::SolveLp {
            direction: ObjectiveDirection::Maximize,
            objective: vec![2.0],
            variables: vec![VariableBounds {
                lower: Some(3.0),
                upper: Some(3.0),
            }],
            constraints: vec![],
        })
        .evaluate()
        {
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
}
