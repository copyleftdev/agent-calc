use agent_calc::{StatsRequest, StatsResponse};
use rug::Float;

const REFERENCE_PRECISION_BITS: u32 = 256;

fn high_precision_mean_and_sample_variance(values: &[f64]) -> (f64, f64) {
    let mut sum = Float::with_val(REFERENCE_PRECISION_BITS, 0);
    for &value in values {
        sum += Float::with_val(REFERENCE_PRECISION_BITS, value);
    }

    let mut mean = sum;
    mean /= values.len() as u32;

    let mut sum_squares = Float::with_val(REFERENCE_PRECISION_BITS, 0);
    for &value in values {
        let mut delta = Float::with_val(REFERENCE_PRECISION_BITS, value);
        delta -= &mean;
        sum_squares += Float::with_val(REFERENCE_PRECISION_BITS, &delta * &delta);
    }

    let mut variance = sum_squares;
    variance /= (values.len() - 1) as u32;
    (mean.to_f64(), variance.to_f64())
}

fn sample_mean_and_variance(values: Vec<f64>) -> (f64, f64) {
    match (StatsRequest::DescribeSample { values }).evaluate() {
        StatsResponse::SampleSummary { mean, variance, .. } => (mean, variance),
        other => panic!("expected sample summary, got {other:?}"),
    }
}

fn assert_close_in_ulps(actual: f64, reference: f64, ulps: f64) {
    let tolerance = ulps * f64::EPSILON * reference.abs().max(1.0);
    assert!(
        (actual - reference).abs() <= tolerance,
        "actual={actual:.17e}, reference={reference:.17e}, tolerance={tolerance:.3e}"
    );
}

#[test]
fn cancellation_heavy_sample_matches_256_bit_reference() {
    let values = vec![1.0e16, 1.0, -1.0e16, 2.0];
    let reference = high_precision_mean_and_sample_variance(&values);
    let actual = sample_mean_and_variance(values);

    assert_close_in_ulps(actual.0, reference.0, 2.0);
    assert_close_in_ulps(actual.1, reference.1, 4.0);
}

#[test]
fn translated_sample_matches_256_bit_reference() {
    let offset = (1_u64 << 40) as f64;
    let values = [-997.0, -503.0, -1.0, 2.0, 499.0, 1000.0]
        .into_iter()
        .map(|value| value + offset)
        .collect::<Vec<_>>();
    let reference = high_precision_mean_and_sample_variance(&values);
    let actual = sample_mean_and_variance(values);

    assert_close_in_ulps(actual.0, reference.0, 2.0);
    assert_close_in_ulps(actual.1, reference.1, 4.0);
}
