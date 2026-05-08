use agent_calc::{MatrixInput, MatrixRequest, MatrixResponse};
use proptest::prelude::*;

fn data_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(-1.0e6f64..1.0e6, 4).prop_filter("finite data", |values| {
        values.iter().all(|value| value.is_finite())
    })
}

fn m(data: Vec<f64>) -> MatrixInput {
    MatrixInput {
        rows: 2,
        cols: 2,
        data,
    }
}

fn matrix_data(response: MatrixResponse) -> Vec<f64> {
    match response {
        MatrixResponse::Matrix { data, .. } => data,
        MatrixResponse::Error { reason, .. } => panic!("expected matrix response: {reason}"),
        other => panic!("expected matrix response, got {other:?}"),
    }
}

fn assert_close(left: &[f64], right: &[f64]) {
    assert_eq!(left.len(), right.len());
    for (a, b) in left.iter().zip(right) {
        assert!((a - b).abs() <= a.abs().max(b.abs()).max(1.0) * 1e-9);
    }
}

proptest! {
    #[test]
    fn adding_zero_preserves_matrix(data in data_strategy()) {
        let zero = m(vec![0.0, 0.0, 0.0, 0.0]);
        let result = matrix_data(MatrixRequest::Add {
            left: m(data.clone()),
            right: zero,
        }.evaluate());

        assert_close(&result, &data);
    }

    #[test]
    fn transpose_twice_preserves_matrix(data in data_strategy()) {
        let first = match (MatrixRequest::Transpose { matrix: m(data.clone()) }).evaluate() {
            MatrixResponse::Matrix { rows, cols, data, .. } => MatrixInput { rows, cols, data },
            other => panic!("expected matrix response, got {other:?}"),
        };
        let second = matrix_data(MatrixRequest::Transpose { matrix: first }.evaluate());

        assert_close(&second, &data);
    }

    #[test]
    fn multiplying_by_identity_preserves_matrix(data in data_strategy()) {
        let identity = m(vec![1.0, 0.0, 0.0, 1.0]);
        let result = matrix_data(MatrixRequest::Mul {
            left: m(data.clone()),
            right: identity,
        }.evaluate());

        assert_close(&result, &data);
    }
}
