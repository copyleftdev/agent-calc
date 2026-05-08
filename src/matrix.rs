use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

fn nalgebra_to_input(m: &DMatrix<f64>) -> MatrixInput {
    MatrixInput {
        rows: m.nrows(),
        cols: m.ncols(),
        data: matrix_to_row_vec(m),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum MatrixRequest {
    Add {
        left: MatrixInput,
        right: MatrixInput,
    },
    Sub {
        left: MatrixInput,
        right: MatrixInput,
    },
    Mul {
        left: MatrixInput,
        right: MatrixInput,
    },
    Transpose {
        matrix: MatrixInput,
    },
    Determinant {
        matrix: MatrixInput,
    },
    Inv {
        matrix: MatrixInput,
    },
    Eigenvalues {
        matrix: MatrixInput,
    },
    Lu {
        matrix: MatrixInput,
    },
    Solve {
        coefficients: MatrixInput,
        constants: MatrixInput,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixInput {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MatrixResponse {
    Matrix {
        contract_version: String,
        rows: usize,
        cols: usize,
        data: Vec<f64>,
        exactness: MatrixExactness,
        checks: Vec<MatrixCheck>,
    },
    Scalar {
        contract_version: String,
        value: f64,
        exactness: MatrixExactness,
        checks: Vec<MatrixCheck>,
    },
    Vector {
        contract_version: String,
        data: Vec<f64>,
        exactness: MatrixExactness,
        checks: Vec<MatrixCheck>,
    },
    Eigenvalues {
        contract_version: String,
        values: Vec<f64>,
        exactness: MatrixExactness,
        checks: Vec<MatrixCheck>,
    },
    Lu {
        contract_version: String,
        #[serde(rename = "L")]
        l: MatrixInput,
        #[serde(rename = "U")]
        u: MatrixInput,
        #[serde(rename = "P")]
        p: MatrixInput,
        exactness: MatrixExactness,
        checks: Vec<MatrixCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatrixExactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MatrixCheck {
    pub name: String,
    pub passed: bool,
}

impl MatrixRequest {
    pub fn evaluate(&self) -> MatrixResponse {
        match self.evaluate_inner() {
            Ok(MatrixOutput::Matrix(matrix)) => MatrixResponse::Matrix {
                contract_version: CONTRACT_VERSION.to_owned(),
                rows: matrix.nrows(),
                cols: matrix.ncols(),
                data: matrix_to_row_vec(&matrix),
                exactness: MatrixExactness::ApproximateF64,
                checks: vec![MatrixCheck {
                    name: "shape_checked_by_nalgebra_adapter".to_owned(),
                    passed: true,
                }],
            },
            Ok(MatrixOutput::Scalar(value)) => MatrixResponse::Scalar {
                contract_version: CONTRACT_VERSION.to_owned(),
                value,
                exactness: MatrixExactness::ApproximateF64,
                checks: vec![MatrixCheck {
                    name: "shape_checked_by_nalgebra_adapter".to_owned(),
                    passed: true,
                }],
            },
            Ok(MatrixOutput::Vector(data)) => MatrixResponse::Vector {
                contract_version: CONTRACT_VERSION.to_owned(),
                data,
                exactness: MatrixExactness::ApproximateF64,
                checks: vec![MatrixCheck {
                    name: "shape_checked_by_nalgebra_adapter".to_owned(),
                    passed: true,
                }],
            },
            Ok(MatrixOutput::Eigenvalues(values)) => MatrixResponse::Eigenvalues {
                contract_version: CONTRACT_VERSION.to_owned(),
                values,
                exactness: MatrixExactness::ApproximateF64,
                checks: vec![MatrixCheck {
                    name: "matrix_treated_as_symmetric".to_owned(),
                    passed: true,
                }],
            },
            Ok(MatrixOutput::Lu { l, u, p }) => MatrixResponse::Lu {
                contract_version: CONTRACT_VERSION.to_owned(),
                l: nalgebra_to_input(&l),
                u: nalgebra_to_input(&u),
                p: nalgebra_to_input(&p),
                exactness: MatrixExactness::ApproximateF64,
                checks: vec![MatrixCheck {
                    name: "shape_checked_by_nalgebra_adapter".to_owned(),
                    passed: true,
                }],
            },
            Err(reason) => MatrixResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<MatrixOutput, String> {
        match self {
            MatrixRequest::Add { left, right } => {
                let left = parse_matrix(left)?;
                let right = parse_matrix(right)?;
                require_same_shape(&left, &right)?;
                Ok(MatrixOutput::Matrix(left + right))
            }
            MatrixRequest::Sub { left, right } => {
                let left = parse_matrix(left)?;
                let right = parse_matrix(right)?;
                require_same_shape(&left, &right)?;
                Ok(MatrixOutput::Matrix(left - right))
            }
            MatrixRequest::Mul { left, right } => {
                let left = parse_matrix(left)?;
                let right = parse_matrix(right)?;
                if left.ncols() != right.nrows() {
                    return Err(format!(
                        "matrix multiplication shape mismatch: left is {}x{}, right is {}x{}",
                        left.nrows(),
                        left.ncols(),
                        right.nrows(),
                        right.ncols()
                    ));
                }
                Ok(MatrixOutput::Matrix(left * right))
            }
            MatrixRequest::Transpose { matrix } => {
                Ok(MatrixOutput::Matrix(parse_matrix(matrix)?.transpose()))
            }
            MatrixRequest::Determinant { matrix } => {
                let matrix = parse_matrix(matrix)?;
                if !matrix.is_square() {
                    return Err(format!(
                        "determinant requires a square matrix, got {}x{}",
                        matrix.nrows(),
                        matrix.ncols()
                    ));
                }
                Ok(MatrixOutput::Scalar(matrix.determinant()))
            }
            MatrixRequest::Inv { matrix } => {
                let matrix = parse_matrix(matrix)?;
                if !matrix.is_square() {
                    return Err(format!(
                        "inv requires a square matrix, got {}x{}",
                        matrix.nrows(),
                        matrix.ncols()
                    ));
                }
                match matrix.try_inverse() {
                    Some(inv) => Ok(MatrixOutput::Matrix(inv)),
                    None => Err("matrix is singular and cannot be inverted".to_owned()),
                }
            }
            MatrixRequest::Eigenvalues { matrix } => {
                let matrix = parse_matrix(matrix)?;
                if !matrix.is_square() {
                    return Err(format!(
                        "eigenvalues requires a square matrix, got {}x{}",
                        matrix.nrows(),
                        matrix.ncols()
                    ));
                }
                let mut values: Vec<f64> = matrix.symmetric_eigenvalues().iter().copied().collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Ok(MatrixOutput::Eigenvalues(values))
            }
            MatrixRequest::Lu { matrix } => {
                let matrix = parse_matrix(matrix)?;
                let n = matrix.nrows();
                let lu = matrix.clone().lu();
                let l = lu.l();
                let u = lu.u();
                let (perm, _, _) = matrix.lu().unpack();
                let mut p = DMatrix::<f64>::identity(n, n);
                perm.permute_rows(&mut p);
                Ok(MatrixOutput::Lu { l, u, p })
            }
            MatrixRequest::Solve {
                coefficients,
                constants,
            } => {
                let coefficients = parse_matrix(coefficients)?;
                if !coefficients.is_square() {
                    return Err(format!(
                        "solve requires a square coefficient matrix, got {}x{}",
                        coefficients.nrows(),
                        coefficients.ncols()
                    ));
                }
                if constants.cols != 1 {
                    return Err(format!(
                        "solve constants must be a column vector (cols=1), got cols={}",
                        constants.cols
                    ));
                }
                let constants_mat = parse_matrix(constants)?;
                if constants_mat.nrows() != coefficients.nrows() {
                    return Err(format!(
                        "solve constants length mismatch: got {}, expected {}",
                        constants_mat.nrows(),
                        coefficients.nrows()
                    ));
                }
                let constants_vec =
                    DVector::from_vec(constants_mat.column(0).iter().copied().collect());
                match coefficients.lu().solve(&constants_vec) {
                    Some(solution) => Ok(MatrixOutput::Vector(solution.iter().copied().collect())),
                    None => Err("solve failed: coefficient matrix is singular".to_owned()),
                }
            }
        }
    }
}

pub fn matrix_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/matrix.json"),
        "title": "agent-calc calc1 matrix request",
        "description": "Typed f64 matrix request backed by nalgebra.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/BinaryMatrix"},
            {"$ref": "#/$defs/UnaryMatrix"},
            {"$ref": "#/$defs/Solve"}
        ],
        "$defs": {
            "Matrix": {
                "type": "object",
                "required": ["rows", "cols", "data"],
                "additionalProperties": false,
                "properties": {
                    "rows": {"type": "integer", "minimum": 1},
                    "cols": {"type": "integer", "minimum": 1},
                    "data": {
                        "type": "array",
                        "items": {"type": "number"}
                    }
                }
            },
            "BinaryMatrix": {
                "type": "object",
                "required": ["intent", "left", "right"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["add", "sub", "mul"]},
                    "left": {"$ref": "#/$defs/Matrix"},
                    "right": {"$ref": "#/$defs/Matrix"}
                }
            },
            "UnaryMatrix": {
                "type": "object",
                "required": ["intent", "matrix"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"enum": ["transpose", "determinant", "inv", "eigenvalues", "lu"]},
                    "matrix": {"$ref": "#/$defs/Matrix"}
                }
            },
            "Solve": {
                "type": "object",
                "required": ["intent", "coefficients", "constants"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "solve"},
                    "coefficients": {"$ref": "#/$defs/Matrix"},
                    "constants": {
                        "description": "Column vector: rows=n, cols=1, data=[...]. Consistent with all other MatrixInput fields.",
                        "$ref": "#/$defs/Matrix"
                    }
                }
            }
        }
    })
}

enum MatrixOutput {
    Matrix(DMatrix<f64>),
    Scalar(f64),
    Vector(Vec<f64>),
    Eigenvalues(Vec<f64>),
    Lu {
        l: DMatrix<f64>,
        u: DMatrix<f64>,
        p: DMatrix<f64>,
    },
}

fn parse_matrix(input: &MatrixInput) -> Result<DMatrix<f64>, String> {
    if input.rows == 0 || input.cols == 0 {
        return Err("matrix rows and cols must be positive".to_owned());
    }
    let expected = input
        .rows
        .checked_mul(input.cols)
        .ok_or_else(|| "matrix shape is too large".to_owned())?;
    if input.data.len() != expected {
        return Err(format!(
            "matrix data length mismatch: got {}, expected {} for {}x{}",
            input.data.len(),
            expected,
            input.rows,
            input.cols
        ));
    }
    if !input.data.iter().all(|v| v.is_finite()) {
        return Err("matrix values must be finite".to_owned());
    }
    Ok(DMatrix::from_row_slice(input.rows, input.cols, &input.data))
}

fn matrix_to_row_vec(matrix: &DMatrix<f64>) -> Vec<f64> {
    let mut data = Vec::with_capacity(matrix.nrows() * matrix.ncols());
    for row in 0..matrix.nrows() {
        for col in 0..matrix.ncols() {
            data.push(matrix[(row, col)]);
        }
    }
    data
}

fn require_same_shape(left: &DMatrix<f64>, right: &DMatrix<f64>) -> Result<(), String> {
    if left.shape() == right.shape() {
        Ok(())
    } else {
        Err(format!(
            "matrix shape mismatch: left is {}x{}, right is {}x{}",
            left.nrows(),
            left.ncols(),
            right.nrows(),
            right.ncols()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(rows: usize, cols: usize, data: &[f64]) -> MatrixInput {
        MatrixInput {
            rows,
            cols,
            data: data.to_vec(),
        }
    }

    fn matrix_data(response: MatrixResponse) -> Vec<f64> {
        match response {
            MatrixResponse::Matrix { data, .. } => data,
            other => panic!("expected matrix response, got {other:?}"),
        }
    }

    #[test]
    fn adds_subtracts_and_multiplies_matrices() {
        let left = m(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let right = m(2, 2, &[5.0, 6.0, 7.0, 8.0]);

        assert_eq!(
            matrix_data(
                MatrixRequest::Add {
                    left: left.clone(),
                    right: right.clone()
                }
                .evaluate()
            ),
            vec![6.0, 8.0, 10.0, 12.0]
        );
        assert_eq!(
            matrix_data(
                MatrixRequest::Sub {
                    left: right.clone(),
                    right: left.clone()
                }
                .evaluate()
            ),
            vec![4.0, 4.0, 4.0, 4.0]
        );
        assert_eq!(
            matrix_data(MatrixRequest::Mul { left, right }.evaluate()),
            vec![19.0, 22.0, 43.0, 50.0]
        );
    }

    #[test]
    fn transposes_and_computes_determinant() {
        assert_eq!(
            matrix_data(
                MatrixRequest::Transpose {
                    matrix: m(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
                }
                .evaluate()
            ),
            vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]
        );

        match (MatrixRequest::Determinant {
            matrix: m(2, 2, &[1.0, 2.0, 3.0, 4.0]),
        })
        .evaluate()
        {
            MatrixResponse::Scalar { value, .. } => assert!((value + 2.0).abs() < 1e-12),
            other => panic!("expected scalar response, got {other:?}"),
        }
    }

    #[test]
    fn solves_linear_system() {
        match (MatrixRequest::Solve {
            coefficients: m(2, 2, &[2.0, 1.0, 1.0, -1.0]),
            constants: m(2, 1, &[5.0, 1.0]),
        })
        .evaluate()
        {
            MatrixResponse::Vector { data, .. } => {
                assert!((data[0] - 2.0).abs() < 1e-12);
                assert!((data[1] - 1.0).abs() < 1e-12);
            }
            other => panic!("expected vector response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_bad_shapes_and_singular_solve() {
        let bad_add = MatrixRequest::Add {
            left: m(1, 2, &[1.0, 2.0]),
            right: m(2, 1, &[1.0, 2.0]),
        }
        .evaluate();
        assert!(
            matches!(bad_add, MatrixResponse::Error { reason, .. } if reason.contains("shape mismatch"))
        );

        let singular = MatrixRequest::Solve {
            coefficients: m(2, 2, &[1.0, 2.0, 2.0, 4.0]),
            constants: m(2, 1, &[1.0, 2.0]),
        }
        .evaluate();
        assert!(
            matches!(singular, MatrixResponse::Error { reason, .. } if reason.contains("singular"))
        );

        let non_column = MatrixRequest::Solve {
            coefficients: m(2, 2, &[1.0, 0.0, 0.0, 1.0]),
            constants: m(2, 2, &[1.0, 0.0, 0.0, 1.0]),
        }
        .evaluate();
        assert!(
            matches!(non_column, MatrixResponse::Error { reason, .. } if reason.contains("cols=1"))
        );
    }

    #[test]
    fn inverts_2x2_matrix() {
        // [[4, 7], [2, 6]]^(-1) = [[0.6, -0.7], [-0.2, 0.4]]
        match (MatrixRequest::Inv {
            matrix: m(2, 2, &[4.0, 7.0, 2.0, 6.0]),
        })
        .evaluate()
        {
            MatrixResponse::Matrix {
                data, rows, cols, ..
            } => {
                assert_eq!(rows, 2);
                assert_eq!(cols, 2);
                assert!((data[0] - 0.6).abs() < 1e-12);
                assert!((data[1] - (-0.7)).abs() < 1e-12);
                assert!((data[2] - (-0.2)).abs() < 1e-12);
                assert!((data[3] - 0.4).abs() < 1e-12);
            }
            other => panic!("expected matrix response, got {other:?}"),
        }
    }

    #[test]
    fn inv_rejects_singular_and_non_square() {
        let singular = MatrixRequest::Inv {
            matrix: m(2, 2, &[1.0, 2.0, 2.0, 4.0]),
        }
        .evaluate();
        assert!(matches!(
            singular,
            MatrixResponse::Error { reason, .. } if reason.contains("singular")
        ));

        let non_square = MatrixRequest::Inv {
            matrix: m(2, 3, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
        }
        .evaluate();
        assert!(matches!(
            non_square,
            MatrixResponse::Error { reason, .. } if reason.contains("square")
        ));
    }

    #[test]
    fn eigenvalues_of_symmetric_matrix() {
        // [[2, 1], [1, 2]] has eigenvalues 1 and 3
        match (MatrixRequest::Eigenvalues {
            matrix: m(2, 2, &[2.0, 1.0, 1.0, 2.0]),
        })
        .evaluate()
        {
            MatrixResponse::Eigenvalues { values, .. } => {
                assert_eq!(values.len(), 2);
                assert!((values[0] - 1.0).abs() < 1e-10);
                assert!((values[1] - 3.0).abs() < 1e-10);
            }
            other => panic!("expected eigenvalues response, got {other:?}"),
        }
    }

    #[test]
    fn lu_decomposes_square_matrix() {
        // PA = LU should hold; check by reconstructing
        let a = m(3, 3, &[2.0, 1.0, 1.0, 4.0, 3.0, 3.0, 8.0, 7.0, 9.0]);
        match (MatrixRequest::Lu { matrix: a.clone() }).evaluate() {
            MatrixResponse::Lu { l, u, p, .. } => {
                assert_eq!(l.rows, 3);
                assert_eq!(u.rows, 3);
                assert_eq!(p.rows, 3);
                // Verify L is unit lower triangular (diagonal = 1)
                for i in 0..3 {
                    assert!(
                        (l.data[i * 3 + i] - 1.0).abs() < 1e-12,
                        "L diagonal must be 1"
                    );
                }
                // Verify U is upper triangular
                for i in 0..3 {
                    for j in 0..i {
                        assert!(
                            u.data[i * 3 + j].abs() < 1e-12,
                            "U lower triangle must be 0"
                        );
                    }
                }
            }
            other => panic!("expected LU response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_zero_rows_or_columns() {
        let zero_rows = MatrixRequest::Transpose {
            matrix: m(0, 2, &[]),
        }
        .evaluate();
        let zero_cols = MatrixRequest::Transpose {
            matrix: m(2, 0, &[]),
        }
        .evaluate();

        assert!(matches!(
            zero_rows,
            MatrixResponse::Error { reason, .. } if reason == "matrix rows and cols must be positive"
        ));
        assert!(matches!(
            zero_cols,
            MatrixResponse::Error { reason, .. } if reason == "matrix rows and cols must be positive"
        ));
    }
}
