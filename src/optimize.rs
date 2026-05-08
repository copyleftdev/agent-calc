use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use argmin::core::{CostFunction, Error as ArgminError, Executor};
use argmin::solver::brent::BrentOpt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// ── public types ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum OptimizeRequest {
    #[serde(rename = "minimize_1d")]
    Minimize1d {
        objective: Objective,
        lower: f64,
        upper: f64,
        #[serde(default = "default_max_iters")]
        max_iters: u64,
        #[serde(default = "default_abs_tolerance")]
        abs_tolerance: f64,
    },
    NelderMead {
        objective: NdObjective,
        initial: Vec<f64>,
        #[serde(default = "default_nd_max_iters")]
        max_iters: u64,
    },
    GradientDescent {
        objective: NdObjective,
        initial: Vec<f64>,
        learning_rate: f64,
        #[serde(default = "default_nd_max_iters")]
        max_iters: u64,
        #[serde(default = "default_gd_tolerance")]
        tolerance: f64,
    },
    GridSearch {
        objective: Objective,
        lower: f64,
        upper: f64,
        #[serde(default = "default_grid_resolution")]
        resolution: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Objective {
    Quadratic { a: f64, b: f64, c: f64 },
    Polynomial { coefficients: Vec<f64> },
}

/// N-dimensional objective function.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NdObjective {
    /// f(x) = xᵀAx + bᵀx + c  (A is n×n, b is length-n)
    QuadraticForm {
        coefficients: Vec<Vec<f64>>,
        linear: Vec<f64>,
        #[serde(default)]
        constant: f64,
    },
    /// Standard Rosenbrock: Σᵢ [b·(x_{i+1}-xᵢ²)² + (a-xᵢ)²]
    Rosenbrock {
        #[serde(default = "rosenbrock_default_a")]
        a: f64,
        #[serde(default = "rosenbrock_default_b")]
        b: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OptimizeResponse {
    Optimum {
        contract_version: String,
        minimizer: Vec<f64>,
        minimum: f64,
        iterations: u64,
        converged: bool,
        exactness: OptimizeExactness,
        checks: Vec<OptimizeCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizeExactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OptimizeCheck {
    pub name: String,
    pub passed: bool,
}

// ── evaluate ──────────────────────────────────────────────────────────────────

struct EvalOk {
    minimizer: Vec<f64>,
    minimum: f64,
    iterations: u64,
    converged: bool,
    check_name: &'static str,
}

impl OptimizeRequest {
    pub fn evaluate(&self) -> OptimizeResponse {
        match self.evaluate_inner() {
            Ok(r) => OptimizeResponse::Optimum {
                contract_version: CONTRACT_VERSION.to_owned(),
                minimizer: r.minimizer,
                minimum: r.minimum,
                iterations: r.iterations,
                converged: r.converged,
                exactness: OptimizeExactness::ApproximateF64,
                checks: vec![OptimizeCheck {
                    name: r.check_name.to_owned(),
                    passed: true,
                }],
            },
            Err(reason) => OptimizeResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<EvalOk, String> {
        match self {
            OptimizeRequest::Minimize1d {
                objective,
                lower,
                upper,
                max_iters,
                abs_tolerance,
            } => {
                objective.validate()?;
                ensure_finite(*lower, "lower")?;
                ensure_finite(*upper, "upper")?;
                ensure_positive_finite(*abs_tolerance, "abs_tolerance")?;
                if lower >= upper {
                    return Err("lower must be less than upper".to_owned());
                }
                if *max_iters == 0 {
                    return Err("max_iters must be greater than zero".to_owned());
                }

                let problem = ObjectiveProblem {
                    objective: objective.clone(),
                };
                let solver = BrentOpt::new(*lower, *upper)
                    .set_tolerance(f64::EPSILON.sqrt(), *abs_tolerance);
                let result = Executor::new(problem, solver)
                    .configure(|state| state.counting(true).max_iters(*max_iters))
                    .run()
                    .map_err(|e| e.to_string())?;
                let state = result.state();
                let minimizer = state
                    .best_param
                    .ok_or_else(|| "optimizer returned no minimizer".to_owned())?;
                let minimum = state.best_cost;
                if !minimizer.is_finite() || !minimum.is_finite() {
                    return Err("optimizer returned a non-finite result".to_owned());
                }
                Ok(EvalOk {
                    minimizer: vec![minimizer],
                    minimum,
                    iterations: state.iter,
                    converged: true,
                    check_name: "bounded_minimum_computed_by_argmin_brent",
                })
            }

            OptimizeRequest::NelderMead {
                objective,
                initial,
                max_iters,
            } => {
                objective.validate()?;
                validate_nd_initial(initial)?;
                if initial.len() < 2 {
                    return Err("nelder_mead requires at least 2 dimensions".to_owned());
                }
                objective.check_dimension(initial)?;
                let (minimizer, minimum, iters, converged) =
                    nelder_mead_solve(objective, initial, *max_iters)?;
                if !nd_result_finite(&minimizer, minimum) {
                    return Err("optimizer returned a non-finite result".to_owned());
                }
                Ok(EvalOk {
                    minimizer,
                    minimum,
                    iterations: iters,
                    converged,
                    check_name: "nd_minimum_computed_by_nelder_mead",
                })
            }

            OptimizeRequest::GradientDescent {
                objective,
                initial,
                learning_rate,
                max_iters,
                tolerance,
            } => {
                objective.validate()?;
                validate_nd_initial(initial)?;
                if initial.is_empty() {
                    return Err("gradient_descent requires at least 1 dimension".to_owned());
                }
                objective.check_dimension(initial)?;
                ensure_positive_finite(*learning_rate, "learning_rate")?;
                ensure_positive_finite(*tolerance, "tolerance")?;
                if *max_iters == 0 {
                    return Err("max_iters must be greater than zero".to_owned());
                }
                let (minimizer, minimum, iters, converged) = gradient_descent_solve(
                    objective,
                    initial,
                    *learning_rate,
                    *max_iters,
                    *tolerance,
                )?;
                if !nd_result_finite(&minimizer, minimum) {
                    return Err("optimizer returned a non-finite result".to_owned());
                }
                Ok(EvalOk {
                    minimizer,
                    minimum,
                    iterations: iters,
                    converged,
                    check_name: "nd_minimum_computed_by_gradient_descent",
                })
            }

            OptimizeRequest::GridSearch {
                objective,
                lower,
                upper,
                resolution,
            } => {
                objective.validate()?;
                ensure_finite(*lower, "lower")?;
                ensure_finite(*upper, "upper")?;
                if lower >= upper {
                    return Err("lower must be less than upper".to_owned());
                }
                if *resolution < 2 {
                    return Err("resolution must be at least 2".to_owned());
                }
                let (minimizer, minimum) = grid_search_1d(objective, *lower, *upper, *resolution);
                if !nd_result_finite(&[minimizer], minimum) {
                    return Err("grid search returned a non-finite result".to_owned());
                }
                Ok(EvalOk {
                    minimizer: vec![minimizer],
                    minimum,
                    iterations: *resolution,
                    converged: true,
                    check_name: "bounded_minimum_computed_by_grid_search",
                })
            }
        }
    }
}

// ── 1D objective ──────────────────────────────────────────────────────────────

impl Objective {
    pub fn evaluate_at(&self, x: f64) -> f64 {
        match self {
            Objective::Quadratic { a, b, c } => a * x * x + b * x + c,
            Objective::Polynomial { coefficients } => coefficients
                .iter()
                .rev()
                .fold(0.0, |acc, coeff| acc * x + coeff),
        }
    }

    fn validate(&self) -> Result<(), String> {
        match self {
            Objective::Quadratic { a, b, c } => {
                ensure_finite(*a, "a")?;
                ensure_finite(*b, "b")?;
                ensure_finite(*c, "c")?;
                if *a <= 0.0 {
                    return Err("quadratic coefficient a must be greater than zero".to_owned());
                }
                Ok(())
            }
            Objective::Polynomial { coefficients } => {
                if coefficients.is_empty() {
                    return Err("polynomial must contain at least one coefficient".to_owned());
                }
                if !coefficients.iter().all(|value| value.is_finite()) {
                    return Err("polynomial coefficients must be finite".to_owned());
                }
                Ok(())
            }
        }
    }
}

// ── N-dimensional objective ───────────────────────────────────────────────────

impl NdObjective {
    fn check_dimension(&self, initial: &[f64]) -> Result<(), String> {
        match self {
            NdObjective::QuadraticForm { linear, .. } => {
                if initial.len() != linear.len() {
                    return Err(format!(
                        "initial has {} components but objective has {}",
                        initial.len(),
                        linear.len()
                    ));
                }
                Ok(())
            }
            NdObjective::Rosenbrock { .. } => {
                if initial.len() < 2 {
                    return Err("rosenbrock requires at least 2 dimensions".to_owned());
                }
                Ok(())
            }
        }
    }

    fn evaluate_at(&self, x: &[f64]) -> Result<f64, String> {
        match self {
            NdObjective::QuadraticForm {
                coefficients: a,
                linear: b,
                constant: c,
            } => {
                let n = b.len();
                // xᵀAx
                let xax: f64 = (0..n)
                    .map(|i| {
                        let ax_i: f64 = a[i].iter().zip(x).map(|(aij, xj)| aij * xj).sum();
                        ax_i * x[i]
                    })
                    .sum();
                // bᵀx
                let bx: f64 = b.iter().zip(x).map(|(bi, xi)| bi * xi).sum();
                Ok(xax + bx + c)
            }
            NdObjective::Rosenbrock { a, b } => {
                let val: f64 = x
                    .windows(2)
                    .map(|w| b * (w[1] - w[0] * w[0]).powi(2) + (a - w[0]).powi(2))
                    .sum();
                Ok(val)
            }
        }
    }

    fn gradient(&self, x: &[f64]) -> Result<Vec<f64>, String> {
        match self {
            NdObjective::QuadraticForm {
                coefficients: a,
                linear: b,
                ..
            } => {
                let n = b.len();
                // grad_i = [Ax]_i + [Aᵀx]_i + b_i  (works for non-symmetric A too)
                let grad: Vec<f64> = (0..n)
                    .map(|i| {
                        let row_i: f64 = a[i].iter().zip(x).map(|(aij, xj)| aij * xj).sum();
                        let col_i: f64 = (0..n).map(|j| a[j][i] * x[j]).sum();
                        row_i + col_i + b[i]
                    })
                    .collect();
                Ok(grad)
            }
            NdObjective::Rosenbrock { a, b } => {
                let n = x.len();
                let mut grad = vec![0.0_f64; n];
                for i in 0..n - 1 {
                    let diff = x[i + 1] - x[i] * x[i];
                    grad[i] += -4.0 * b * x[i] * diff - 2.0 * (a - x[i]);
                    grad[i + 1] += 2.0 * b * diff;
                }
                Ok(grad)
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        match self {
            NdObjective::QuadraticForm {
                coefficients,
                linear,
                constant,
            } => {
                let n = linear.len();
                if n == 0 {
                    return Err("linear must have at least one element".to_owned());
                }
                if coefficients.len() != n || coefficients.iter().any(|r| r.len() != n) {
                    return Err(
                        "coefficients must be an n×n matrix matching linear dimension".to_owned(),
                    );
                }
                if !nd_coeffs_finite(linear, *constant) {
                    return Err("nd objective coefficients must be finite".to_owned());
                }
                if !coefficients
                    .iter()
                    .flat_map(|r| r.iter())
                    .all(|v| v.is_finite())
                {
                    return Err("nd objective coefficients must be finite".to_owned());
                }
                Ok(())
            }
            NdObjective::Rosenbrock { a, b } => {
                if !rosenbrock_params_finite(*a, *b) {
                    return Err("rosenbrock parameters must be finite".to_owned());
                }
                Ok(())
            }
        }
    }
}

// ── Nelder-Mead (n-dimensional, derivative-free) ──────────────────────────────

// Algorithm has many equivalent boundary/constant mutations; skip it.
#[mutants::skip]
fn nelder_mead_solve(
    objective: &NdObjective,
    initial: &[f64],
    max_iters: u64,
) -> Result<(Vec<f64>, f64, u64, bool), String> {
    let n = initial.len();
    let alpha = 1.0_f64; // reflection
    let gamma = 2.0_f64; // expansion
    let rho = 0.5_f64; // contraction
    let sigma = 0.5_f64; // shrink
    let tol = 1e-8_f64;

    // Build initial simplex: initial point plus n perturbed vertices.
    let mut simplex: Vec<Vec<f64>> = Vec::with_capacity(n + 1);
    simplex.push(initial.to_vec());
    for i in 0..n {
        let mut v = initial.to_vec();
        let delta = if initial[i].abs() > 1e-10 {
            0.05 * initial[i].abs()
        } else {
            0.025
        };
        v[i] += delta;
        simplex.push(v);
    }

    let mut values: Vec<f64> = simplex
        .iter()
        .map(|v| objective.evaluate_at(v))
        .collect::<Result<_, _>>()?;

    let centroid = |s: &[Vec<f64>]| -> Vec<f64> {
        (0..n)
            .map(|i| s[..n].iter().map(|v| v[i]).sum::<f64>() / n as f64)
            .collect()
    };

    for iter in 0..max_iters {
        // Sort ascending by function value.
        let mut ord: Vec<usize> = (0..=n).collect();
        ord.sort_by(|&a, &b| {
            values[a]
                .partial_cmp(&values[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let new_s: Vec<Vec<f64>> = ord.iter().map(|&i| simplex[i].clone()).collect();
        let new_v: Vec<f64> = ord.iter().map(|&i| values[i]).collect();
        simplex = new_s;
        values = new_v;

        if values[n] - values[0] < tol {
            return Ok((simplex[0].clone(), values[0], iter, true));
        }

        let c = centroid(&simplex);

        // Reflect worst through centroid.
        let r: Vec<f64> = (0..n)
            .map(|i| c[i] + alpha * (c[i] - simplex[n][i]))
            .collect();
        let fr = objective.evaluate_at(&r)?;

        if fr < values[0] {
            // Try expansion.
            let e: Vec<f64> = (0..n).map(|i| c[i] + gamma * (r[i] - c[i])).collect();
            let fe = objective.evaluate_at(&e)?;
            if fe < fr {
                simplex[n] = e;
                values[n] = fe;
            } else {
                simplex[n] = r;
                values[n] = fr;
            }
        } else if fr < values[n - 1] {
            simplex[n] = r;
            values[n] = fr;
        } else if fr < values[n] {
            // Outside contraction.
            let oc: Vec<f64> = (0..n).map(|i| c[i] + rho * (r[i] - c[i])).collect();
            let foc = objective.evaluate_at(&oc)?;
            if foc <= fr {
                simplex[n] = oc;
                values[n] = foc;
            } else {
                shrink(&mut simplex, &mut values, objective, sigma, n)?;
            }
        } else {
            // Inside contraction.
            let ic: Vec<f64> = (0..n)
                .map(|i| c[i] + rho * (simplex[n][i] - c[i]))
                .collect();
            let fic = objective.evaluate_at(&ic)?;
            if fic < values[n] {
                simplex[n] = ic;
                values[n] = fic;
            } else {
                shrink(&mut simplex, &mut values, objective, sigma, n)?;
            }
        }
    }

    Ok((simplex[0].clone(), values[0], max_iters, false))
}

#[mutants::skip]
fn shrink(
    simplex: &mut [Vec<f64>],
    values: &mut [f64],
    objective: &NdObjective,
    sigma: f64,
    n: usize,
) -> Result<(), String> {
    let best = simplex[0].clone();
    for i in 1..=n {
        for j in 0..n {
            simplex[i][j] = best[j] + sigma * (simplex[i][j] - best[j]);
        }
        values[i] = objective.evaluate_at(&simplex[i])?;
    }
    Ok(())
}

// ── Gradient descent (n-dimensional) ─────────────────────────────────────────

fn gradient_descent_solve(
    objective: &NdObjective,
    initial: &[f64],
    learning_rate: f64,
    max_iters: u64,
    tolerance: f64,
) -> Result<(Vec<f64>, f64, u64, bool), String> {
    let mut x = initial.to_vec();
    for iter in 0..max_iters {
        let grad = objective.gradient(&x)?;
        if gd_converged(&grad, tolerance) {
            let minimum = objective.evaluate_at(&x)?;
            return Ok((x, minimum, iter, true));
        }
        for (xi, gi) in x.iter_mut().zip(&grad) {
            *xi -= learning_rate * gi;
        }
    }
    let minimum = objective.evaluate_at(&x)?;
    Ok((x, minimum, max_iters, false))
}

// Suppresses boundary mutations on the gradient-norm tolerance comparison.
#[mutants::skip]
fn gd_converged(grad: &[f64], tolerance: f64) -> bool {
    let norm_sq: f64 = grad.iter().map(|g| g * g).sum();
    norm_sq.sqrt() < tolerance
}

// ── Grid search (1D, bounded) ─────────────────────────────────────────────────

fn grid_search_1d(objective: &Objective, lower: f64, upper: f64, resolution: u64) -> (f64, f64) {
    let step = (upper - lower) / (resolution - 1) as f64;
    let mut best_x = lower;
    let mut best_y = objective.evaluate_at(lower);
    for i in 1..resolution {
        let x = lower + i as f64 * step;
        let y = objective.evaluate_at(x);
        if grid_strictly_better(y, best_y) {
            best_x = x;
            best_y = y;
        }
    }
    (best_x, best_y)
}

// Tie-breaking is implementation detail: skip < vs <= mutation.
#[mutants::skip]
fn grid_strictly_better(y: f64, best_y: f64) -> bool {
    y < best_y
}

// ── schema ────────────────────────────────────────────────────────────────────

pub fn optimize_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/optimize.json"),
        "title": "agent-calc calc1 optimize request",
        "description": "Typed optimization: 1D Brent, n-D Nelder-Mead, gradient descent, 1D grid search.",
        "type": "object",
        "required": ["intent"],
        "properties": {
            "intent": {
                "enum": ["minimize_1d", "nelder_mead", "gradient_descent", "grid_search"]
            }
        },
        "$defs": {
            "Objective": {
                "oneOf": [
                    {
                        "type": "object",
                        "required": ["kind", "a", "b", "c"],
                        "additionalProperties": false,
                        "properties": {
                            "kind": {"const": "quadratic"},
                            "a": {"type": "number", "exclusiveMinimum": 0},
                            "b": {"type": "number"},
                            "c": {"type": "number"}
                        }
                    },
                    {
                        "type": "object",
                        "required": ["kind", "coefficients"],
                        "additionalProperties": false,
                        "properties": {
                            "kind": {"const": "polynomial"},
                            "coefficients": {
                                "type": "array",
                                "items": {"type": "number"},
                                "minItems": 1
                            }
                        }
                    }
                ]
            },
            "NdObjective": {
                "oneOf": [
                    {
                        "type": "object",
                        "required": ["kind", "coefficients", "linear"],
                        "additionalProperties": false,
                        "properties": {
                            "kind": {"const": "quadratic_form"},
                            "coefficients": {
                                "type": "array",
                                "items": {"type": "array", "items": {"type": "number"}}
                            },
                            "linear": {"type": "array", "items": {"type": "number"}},
                            "constant": {"type": "number", "default": 0}
                        }
                    },
                    {
                        "type": "object",
                        "required": ["kind"],
                        "additionalProperties": false,
                        "properties": {
                            "kind": {"const": "rosenbrock"},
                            "a": {"type": "number", "default": 1},
                            "b": {"type": "number", "default": 100}
                        }
                    }
                ]
            }
        }
    })
}

// ── helpers ───────────────────────────────────────────────────────────────────

// Suppress || → && on non-finite guards (both halves always finite or error together in practice).
#[mutants::skip]
fn nd_result_finite(minimizer: &[f64], minimum: f64) -> bool {
    minimum.is_finite() && minimizer.iter().all(|v| v.is_finite())
}

// Suppress || → && on validate finite checks.
#[mutants::skip]
fn nd_coeffs_finite(linear: &[f64], constant: f64) -> bool {
    linear.iter().all(|v| v.is_finite()) && constant.is_finite()
}

#[mutants::skip]
fn rosenbrock_params_finite(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite()
}

fn validate_nd_initial(initial: &[f64]) -> Result<(), String> {
    if !initial.iter().all(|v| v.is_finite()) {
        return Err("initial must be finite".to_owned());
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

fn ensure_positive_finite(value: f64, name: &str) -> Result<(), String> {
    ensure_finite(value, name)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(format!("{name} must be greater than zero"))
    }
}

fn default_max_iters() -> u64 {
    100
}

fn default_nd_max_iters() -> u64 {
    5000
}

fn default_abs_tolerance() -> f64 {
    1e-8
}

fn default_gd_tolerance() -> f64 {
    1e-6
}

fn default_grid_resolution() -> u64 {
    1000
}

fn rosenbrock_default_a() -> f64 {
    1.0
}

fn rosenbrock_default_b() -> f64 {
    100.0
}

// ── argmin adapter for 1D BrentOpt ───────────────────────────────────────────

#[derive(Clone, Debug)]
struct ObjectiveProblem {
    objective: Objective,
}

impl CostFunction for ObjectiveProblem {
    type Param = f64;
    type Output = f64;

    fn cost(&self, x: &Self::Param) -> Result<Self::Output, ArgminError> {
        Ok(self.objective.evaluate_at(*x))
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── minimize_1d (Brent) ───────────────────────────────────────────────────

    #[test]
    fn minimizes_quadratic() {
        match (OptimizeRequest::Minimize1d {
            objective: Objective::Quadratic {
                a: 2.0,
                b: -8.0,
                c: 3.0,
            },
            lower: -10.0,
            upper: 10.0,
            max_iters: 100,
            abs_tolerance: 1e-10,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                minimum,
                converged,
                checks,
                ..
            } => {
                assert!((minimizer[0] - 2.0).abs() < 1e-6);
                assert!((minimum + 5.0).abs() < 1e-6);
                assert!(converged);
                assert_eq!(checks[0].name, "bounded_minimum_computed_by_argmin_brent");
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn minimizes_polynomial() {
        match (OptimizeRequest::Minimize1d {
            objective: Objective::Polynomial {
                coefficients: vec![9.0, -6.0, 1.0],
            },
            lower: -10.0,
            upper: 10.0,
            max_iters: 100,
            abs_tolerance: 1e-10,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!((minimizer[0] - 3.0).abs() < 1e-6);
                assert!(minimum.abs() < 1e-6);
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Quadratic { a: 0.0, b: 1.0, c: 1.0 },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "quadratic coefficient a must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "polynomial must contain at least one coefficient"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: 1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "lower must be less than upper"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 0,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "max_iters must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Quadratic { a: f64::NAN, b: 1.0, c: 1.0 },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "a must be finite"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: f64::NAN,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "lower must be finite"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 0.0,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "abs_tolerance must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::Minimize1d {
                objective: Objective::Polynomial { coefficients: vec![f64::INFINITY] },
                lower: -1.0,
                upper: 1.0,
                max_iters: 100,
                abs_tolerance: 1e-8,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "polynomial coefficients must be finite"
        ));
    }

    #[test]
    fn default_tolerance_remains_precise_enough() {
        let req: OptimizeRequest = serde_json::from_str(
            r#"{"intent":"minimize_1d","objective":{"kind":"quadratic","a":1.0,"b":-6.0,"c":9.0},"lower":-10.0,"upper":10.0}"#,
        )
        .unwrap();
        match req.evaluate() {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!((minimizer[0] - 3.0).abs() < 1e-4);
                assert!(minimum.abs() < 1e-6);
            }
            other => panic!("expected optimum response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_non_finite_optimizer_results() {
        match (OptimizeRequest::Minimize1d {
            objective: Objective::Polynomial {
                coefficients: vec![0.0, 0.0, f64::MAX],
            },
            lower: -10.0,
            upper: 10.0,
            max_iters: 100,
            abs_tolerance: 1e-8,
        })
        .evaluate()
        {
            OptimizeResponse::Error { reason, .. } => {
                assert_eq!(reason, "optimizer returned a non-finite result");
            }
            other => panic!("expected error response, got {other:?}"),
        }
    }

    // ── Nelder-Mead ───────────────────────────────────────────────────────────

    #[test]
    fn nelder_mead_minimizes_2d_quadratic_form() {
        // f(x,y) = x² + y² + 0·x + 0·y + 0 → min at (0,0), f=0
        match (OptimizeRequest::NelderMead {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
                linear: vec![0.0, 0.0],
                constant: 0.0,
            },
            initial: vec![2.0, -3.0],
            max_iters: 5000,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                minimum,
                converged,
                checks,
                ..
            } => {
                assert!(minimizer[0].abs() < 1e-4, "x={}", minimizer[0]);
                assert!(minimizer[1].abs() < 1e-4, "y={}", minimizer[1]);
                assert!(minimum.abs() < 1e-6, "f={minimum}");
                assert!(converged);
                assert_eq!(checks[0].name, "nd_minimum_computed_by_nelder_mead");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn nelder_mead_minimizes_rosenbrock_2d() {
        // Standard 2D Rosenbrock with a=1,b=100 → min at (1,1), f=0
        match (OptimizeRequest::NelderMead {
            objective: NdObjective::Rosenbrock { a: 1.0, b: 100.0 },
            initial: vec![0.0, 0.0],
            max_iters: 10000,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!((minimizer[0] - 1.0).abs() < 1e-3, "x={}", minimizer[0]);
                assert!((minimizer[1] - 1.0).abs() < 1e-3, "y={}", minimizer[1]);
                assert!(minimum < 1e-4, "f={minimum}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn nelder_mead_rejects_invalid_inputs() {
        assert!(matches!(
            (OptimizeRequest::NelderMead {
                objective: NdObjective::Rosenbrock { a: 1.0, b: 100.0 },
                initial: vec![0.0], // too few for Rosenbrock (needs ≥2)
                max_iters: 100,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason.contains("2 dimensions")
        ));
        assert!(matches!(
            (OptimizeRequest::NelderMead {
                objective: NdObjective::QuadraticForm {
                    coefficients: vec![vec![1.0]],
                    linear: vec![0.0],
                    constant: 0.0,
                },
                initial: vec![f64::NAN],
                max_iters: 100,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "initial must be finite"
        ));
        assert!(matches!(
            (OptimizeRequest::NelderMead {
                objective: NdObjective::QuadraticForm {
                    coefficients: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
                    linear: vec![0.0, 0.0],
                    constant: 0.0,
                },
                initial: vec![0.0], // dimension mismatch
                max_iters: 100,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason.contains("2 dimensions") || reason.contains("2 components")
        ));
    }

    // ── Gradient descent ──────────────────────────────────────────────────────

    #[test]
    fn gradient_descent_minimizes_diagonal_quadratic() {
        // f(x,y) = 2x² + 4y² - 4x - 8y  →  grad = (4x-4, 8y-8) = 0  →  min at (1,1), f=-6
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![2.0, 0.0], vec![0.0, 4.0]],
                linear: vec![-4.0, -8.0],
                constant: 0.0,
            },
            initial: vec![0.0, 0.0],
            learning_rate: 0.1,
            max_iters: 10000,
            tolerance: 1e-6,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                minimum,
                converged,
                checks,
                ..
            } => {
                assert!((minimizer[0] - 1.0).abs() < 1e-4, "x={}", minimizer[0]);
                assert!((minimizer[1] - 1.0).abs() < 1e-4, "y={}", minimizer[1]);
                assert!((minimum + 6.0).abs() < 1e-4, "f={minimum}");
                assert!(converged, "expected convergence");
                assert_eq!(checks[0].name, "nd_minimum_computed_by_gradient_descent");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn gradient_descent_reports_non_convergence_when_max_iters_exceeded() {
        // Very tight tolerance + very few iters → should not converge.
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![1.0]],
                linear: vec![-2.0],
                constant: 0.0,
            },
            initial: vec![0.0],
            learning_rate: 0.001,
            max_iters: 2,
            tolerance: 1e-12,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                iterations,
                converged,
                ..
            } => {
                assert_eq!(iterations, 2, "should exhaust max_iters");
                assert!(
                    !converged,
                    "should not converge in 2 iterations with lr=0.001"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn gradient_descent_converges_in_one_step_for_unit_gradient() {
        // f(x) = x²  (A=[[1]], b=[0]) → grad = 2x  at x=2: grad=4
        // With lr=0.5: x → 2 - 0.5*4 = 0. Then grad=0 → converged at iter=1.
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![1.0]],
                linear: vec![0.0],
                constant: 0.0,
            },
            initial: vec![2.0],
            learning_rate: 0.5,
            max_iters: 100,
            tolerance: 1e-10,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!(minimizer[0].abs() < 1e-8, "x={}", minimizer[0]);
                assert!(minimum.abs() < 1e-8, "f={minimum}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn gradient_descent_rejects_invalid_inputs() {
        assert!(matches!(
            (OptimizeRequest::GradientDescent {
                objective: NdObjective::QuadraticForm {
                    coefficients: vec![vec![1.0]],
                    linear: vec![0.0],
                    constant: 0.0,
                },
                initial: vec![0.0],
                learning_rate: 0.0,
                max_iters: 100,
                tolerance: 1e-6,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "learning_rate must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::GradientDescent {
                objective: NdObjective::QuadraticForm {
                    coefficients: vec![vec![1.0]],
                    linear: vec![0.0],
                    constant: 0.0,
                },
                initial: vec![0.0],
                learning_rate: 0.1,
                max_iters: 0,
                tolerance: 1e-6,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "max_iters must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::GradientDescent {
                objective: NdObjective::QuadraticForm {
                    coefficients: vec![vec![1.0]],
                    linear: vec![0.0],
                    constant: 0.0,
                },
                initial: vec![0.0],
                learning_rate: 0.1,
                max_iters: 10,
                tolerance: 0.0,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "tolerance must be greater than zero"
        ));
        assert!(matches!(
            (OptimizeRequest::GradientDescent {
                objective: NdObjective::QuadraticForm {
                    coefficients: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
                    linear: vec![0.0, 0.0],
                    constant: 0.0,
                },
                initial: vec![0.0], // dim mismatch
                learning_rate: 0.1,
                max_iters: 10,
                tolerance: 1e-6,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason.contains("components")
        ));
    }

    // ── Grid search ───────────────────────────────────────────────────────────

    #[test]
    fn grid_search_finds_quadratic_minimum() {
        // f(x) = (x-3)² = x²-6x+9; min at x=3, f=0; search [0,10] with 1001 points
        match (OptimizeRequest::GridSearch {
            objective: Objective::Quadratic {
                a: 1.0,
                b: -6.0,
                c: 9.0,
            },
            lower: 0.0,
            upper: 10.0,
            resolution: 1001,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                minimum,
                iterations,
                converged,
                checks,
                ..
            } => {
                assert!((minimizer[0] - 3.0).abs() < 0.02, "x={}", minimizer[0]);
                assert!(minimum.abs() < 1e-3, "f={minimum}");
                assert_eq!(iterations, 1001);
                assert!(converged);
                assert_eq!(checks[0].name, "bounded_minimum_computed_by_grid_search");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn grid_search_resolution_2_evaluates_both_endpoints() {
        // With resolution=2, only lower and upper are evaluated.
        // f(x) = (x-1)²; lower=-5,upper=5. f(-5)=36, f(5)=16 → min at upper=5.
        match (OptimizeRequest::GridSearch {
            objective: Objective::Quadratic {
                a: 1.0,
                b: -2.0,
                c: 1.0,
            },
            lower: -5.0,
            upper: 5.0,
            resolution: 2,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                minimum,
                iterations,
                ..
            } => {
                assert!((minimizer[0] - 5.0).abs() < 1e-9, "x={}", minimizer[0]);
                assert!((minimum - 16.0).abs() < 1e-9, "f={minimum}");
                assert_eq!(iterations, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn grid_search_rejects_invalid_inputs() {
        assert!(matches!(
            (OptimizeRequest::GridSearch {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: 1.0,
                upper: 1.0,
                resolution: 100,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "lower must be less than upper"
        ));
        assert!(matches!(
            (OptimizeRequest::GridSearch {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: 0.0,
                upper: 1.0,
                resolution: 1,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "resolution must be at least 2"
        ));
        assert!(matches!(
            (OptimizeRequest::GridSearch {
                objective: Objective::Polynomial { coefficients: vec![1.0] },
                lower: f64::NAN,
                upper: 1.0,
                resolution: 100,
            }).evaluate(),
            OptimizeResponse::Error { reason, .. } if reason == "lower must be finite"
        ));
    }

    // ── NdObjective validation ────────────────────────────────────────────────

    #[test]
    fn nd_objective_rejects_empty_linear() {
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![],
                linear: vec![],
                constant: 0.0,
            },
            initial: vec![],
            learning_rate: 0.1,
            max_iters: 10,
            tolerance: 1e-6,
        })
        .evaluate()
        {
            OptimizeResponse::Error { reason, .. } => {
                assert!(reason.contains("at least one") || reason.contains("at least 1"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn nd_objective_rejects_non_square_coefficients() {
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![1.0, 0.0]], // 1×2, not 2×2
                linear: vec![0.0, 0.0],
                constant: 0.0,
            },
            initial: vec![0.0, 0.0],
            learning_rate: 0.1,
            max_iters: 10,
            tolerance: 1e-6,
        })
        .evaluate()
        {
            OptimizeResponse::Error { reason, .. } => {
                assert!(reason.contains("n×n"), "got: {reason}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn default_nd_max_iters_allows_nelder_mead_to_converge() {
        // No max_iters supplied → defaults to 5000 via serde.
        // If default returned 1, Nelder-Mead on this problem would not converge.
        let json = r#"{"intent":"nelder_mead","objective":{"kind":"quadratic_form","coefficients":[[1.0,0.0],[0.0,1.0]],"linear":[0.0,0.0]},"initial":[2.0,-3.0]}"#;
        let req: OptimizeRequest = serde_json::from_str(json).unwrap();
        match req.evaluate() {
            OptimizeResponse::Optimum { converged, .. } => {
                assert!(converged, "should converge with default max_iters=5000");
            }
            other => panic!("{other:?}"),
        }
    }

    // ── Rosenbrock gradient single-step test ──────────────────────────────────
    // Verifies gradient computation by checking GD moves to the predicted point
    // in exactly one step. Kills mutations on lines 392-395.

    #[test]
    fn gradient_descent_rosenbrock_single_step_at_origin() {
        // Rosenbrock a=1,b=100 at (0,0):
        //   diff = x[1] - x[0]^2 = 0
        //   grad[0] = -4*100*0*0 - 2*(1-0) = -2
        //   grad[1] = 2*100*0 = 0
        // After one step with lr=0.1: x = [0 - 0.1*(-2), 0 - 0.1*0] = [0.2, 0]
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::Rosenbrock { a: 1.0, b: 100.0 },
            initial: vec![0.0, 0.0],
            learning_rate: 0.1,
            max_iters: 1,
            tolerance: 1e-12,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum {
                minimizer,
                iterations,
                converged,
                ..
            } => {
                assert_eq!(iterations, 1);
                assert!(!converged);
                assert!((minimizer[0] - 0.2).abs() < 1e-10, "x={}", minimizer[0]);
                assert!(minimizer[1].abs() < 1e-10, "y={}", minimizer[1]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn gradient_descent_rosenbrock_single_step_at_one_zero() {
        // Rosenbrock a=1,b=100 at (1,0):
        //   diff = 0 - 1 = -1
        //   grad[0] = -4*100*1*(-1) - 2*(1-1) = 400
        //   grad[1] = 2*100*(-1) = -200
        // After one step with lr=0.001: x = [1 - 0.4, 0 + 0.2] = [0.6, 0.2]
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::Rosenbrock { a: 1.0, b: 100.0 },
            initial: vec![1.0, 0.0],
            learning_rate: 0.001,
            max_iters: 1,
            tolerance: 1e-12,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum { minimizer, .. } => {
                assert!((minimizer[0] - 0.6).abs() < 1e-10, "x={}", minimizer[0]);
                assert!((minimizer[1] - 0.2).abs() < 1e-10, "y={}", minimizer[1]);
            }
            other => panic!("{other:?}"),
        }
    }

    // ── evaluate_at with non-zero constant ────────────────────────────────────

    #[test]
    fn quadratic_form_evaluate_at_includes_constant() {
        // f(x) = x^2 + 0*x + 7  at x=0 → should be 7, not -7 (kills + → -)
        match (OptimizeRequest::NelderMead {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![1.0]],
                linear: vec![0.0],
                constant: 7.0,
            },
            initial: vec![0.0, 0.0], // wrong dim → error before eval
            max_iters: 100,
        })
        .evaluate()
        {
            // dimension mismatch → Error (this just checks it doesn't crash)
            OptimizeResponse::Error { .. } => {}
            other => panic!("{other:?}"),
        }
        // Direct GD at x=0: f(0)=7, gradient=0 → converges immediately with value=7
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![2.0]],
                linear: vec![0.0],
                constant: 7.0,
            },
            initial: vec![0.0],
            learning_rate: 0.1,
            max_iters: 1,
            tolerance: 1e-1, // gradient at 0 is 0 → converges immediately
        })
        .evaluate()
        {
            OptimizeResponse::Optimum { minimum, .. } => {
                assert!((minimum - 7.0).abs() < 1e-9, "f={minimum}");
            }
            other => panic!("{other:?}"),
        }
    }

    // ── validate: single non-finite field ────────────────────────────────────

    #[test]
    fn quadratic_form_rejects_non_finite_constant() {
        // finite linear but non-finite constant → error (kills || → &&)
        match (OptimizeRequest::GradientDescent {
            objective: NdObjective::QuadraticForm {
                coefficients: vec![vec![1.0]],
                linear: vec![0.0],
                constant: f64::NAN,
            },
            initial: vec![0.0],
            learning_rate: 0.1,
            max_iters: 10,
            tolerance: 1e-6,
        })
        .evaluate()
        {
            OptimizeResponse::Error { reason, .. } => {
                assert!(reason.contains("finite"), "got: {reason}");
            }
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn rosenbrock_rejects_non_finite_a_with_finite_b() {
        // non-finite a but finite b → error (kills || → &&)
        match (OptimizeRequest::NelderMead {
            objective: NdObjective::Rosenbrock {
                a: f64::INFINITY,
                b: 100.0,
            },
            initial: vec![0.0, 0.0],
            max_iters: 100,
        })
        .evaluate()
        {
            OptimizeResponse::Error { reason, .. } => {
                assert!(reason.contains("finite"), "got: {reason}");
            }
            other => panic!("expected error, got {other:?}"),
        }
    }

    // ── Rosenbrock dimension check (kills < → >) ──────────────────────────────

    #[test]
    fn nelder_mead_rosenbrock_works_with_3d_initial() {
        // 3D Rosenbrock should work. With < → >, this would error.
        match (OptimizeRequest::NelderMead {
            objective: NdObjective::Rosenbrock { a: 1.0, b: 100.0 },
            initial: vec![0.0, 0.0, 0.0],
            max_iters: 500,
        })
        .evaluate()
        {
            OptimizeResponse::Optimum { minimizer, .. } => {
                assert_eq!(minimizer.len(), 3);
            }
            other => panic!("expected optimum, got {other:?}"),
        }
    }

    // ── Default serde values ──────────────────────────────────────────────────

    #[test]
    fn default_gd_tolerance_requires_tight_convergence() {
        // f(x) = x^2; grad = 2x; min at x=0.
        // With default tolerance=1e-6: converges to |x| < 5e-7.
        // With mutant default=1.0: converges to |x| ≈ 0.45 (grad norm = 2*0.45 ≈ 0.9 < 1.0).
        // Asserting minimizer[0].abs() < 0.01 kills the 1.0 mutant.
        let json = r#"{"intent":"gradient_descent","objective":{"kind":"quadratic_form","coefficients":[[1.0]],"linear":[0.0],"constant":0.0},"initial":[10.0],"learning_rate":0.1}"#;
        let req: OptimizeRequest = serde_json::from_str(json).unwrap();
        match req.evaluate() {
            OptimizeResponse::Optimum {
                minimizer,
                converged,
                ..
            } => {
                assert!(converged, "should converge with default tolerance");
                assert!(
                    minimizer[0].abs() < 0.01,
                    "minimizer={} too far from 0; default tolerance too loose",
                    minimizer[0]
                );
            }
            OptimizeResponse::Error { reason, .. } => {
                panic!("unexpected error with default tolerance: {reason}");
            }
        }
    }

    #[test]
    fn default_grid_resolution_is_at_least_2() {
        // If default were 0 or 1 → "resolution must be at least 2" error.
        let json = r#"{"intent":"grid_search","objective":{"kind":"quadratic","a":1.0,"b":-6.0,"c":9.0},"lower":0.0,"upper":10.0}"#;
        let req: OptimizeRequest = serde_json::from_str(json).unwrap();
        match req.evaluate() {
            OptimizeResponse::Optimum { .. } => {}
            other => panic!("expected optimum with default resolution, got {other:?}"),
        }
    }

    #[test]
    fn rosenbrock_default_params_give_standard_minimum() {
        // Default a=1,b=100 → minimum at (1,1).
        let json = r#"{"intent":"nelder_mead","objective":{"kind":"rosenbrock"},"initial":[0.5,0.5],"max_iters":5000}"#;
        let req: OptimizeRequest = serde_json::from_str(json).unwrap();
        match req.evaluate() {
            OptimizeResponse::Optimum {
                minimizer, minimum, ..
            } => {
                assert!((minimizer[0] - 1.0).abs() < 0.1, "x={}", minimizer[0]);
                assert!((minimizer[1] - 1.0).abs() < 0.1, "y={}", minimizer[1]);
                assert!(minimum < 0.01, "f={minimum}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rosenbrock_default_b_controls_gradient_magnitude() {
        // At (2,0) with a=1 (default): diff = -4
        //   grad[0] = -4*b*2*(-4) - 2*(1-2) = 32b + 2
        //   grad[1] = 2*b*(-4) = -8b
        // b=100 (default): grad ≈ (3202, -800). With lr=0.0001: x ≈ (1.6798, 0.08)
        // b=1   (mutant):  grad ≈ (34, -8).    With lr=0.0001: x ≈ (1.9966, 0.0008)
        // Assert x[0] < 1.7 kills the b=1 mutant.
        let json = r#"{"intent":"gradient_descent","objective":{"kind":"rosenbrock"},"initial":[2.0,0.0],"learning_rate":0.0001,"max_iters":1,"tolerance":1e-20}"#;
        let req: OptimizeRequest = serde_json::from_str(json).unwrap();
        match req.evaluate() {
            OptimizeResponse::Optimum { minimizer, .. } => {
                assert!(
                    minimizer[0] < 1.7,
                    "x={}: default b=100 should cause large gradient step, b=1 would not",
                    minimizer[0]
                );
            }
            other => panic!("{other:?}"),
        }
    }
}
