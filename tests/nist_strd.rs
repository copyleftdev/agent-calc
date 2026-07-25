use agent_calc::{StatsRequest, StatsResponse, stats::XInput};

fn sample_summary(values: Vec<f64>) -> (usize, f64, f64) {
    match (StatsRequest::DescribeSample { values }).evaluate() {
        StatsResponse::SampleSummary {
            n, mean, std_dev, ..
        } => (n, mean, std_dev),
        other => panic!("expected sample summary, got {other:?}"),
    }
}

fn assert_close(actual: f64, certified: f64, relative_tolerance: f64) {
    let relative_error = (actual - certified).abs() / certified.abs().max(f64::MIN_POSITIVE);
    assert!(
        relative_error <= relative_tolerance,
        "relative error {relative_error:e} exceeds {relative_tolerance:e}: \
         actual={actual:.17e}, certified={certified:.17e}"
    );
}

fn nist_numacc(base: f64) -> Vec<f64> {
    let mut values = Vec::with_capacity(1001);
    values.push(base);
    for _ in 0..500 {
        values.push(base - 0.1);
        values.push(base + 0.1);
    }
    values
}

#[test]
fn nist_numacc1_certified_mean_and_standard_deviation() {
    // NIST StRD NumAcc1:
    // https://www.itl.nist.gov/div898/strd/univ/data/NumAcc1.dat
    let (n, mean, std_dev) = sample_summary(vec![10_000_001.0, 10_000_003.0, 10_000_002.0]);

    assert_eq!(n, 3);
    assert_eq!(mean, 10_000_002.0);
    assert_eq!(std_dev, 1.0);
}

#[test]
fn nist_numacc2_through_4_certified_univariate_statistics() {
    // These generated NIST datasets contain one `base` value followed by 500
    // alternating pairs at base ± 0.1. Their certified standard deviation is
    // exactly 0.1. Decimal JSON inputs become binary f64 values, so the
    // standard-deviation budget includes their unavoidable representation
    // error and grows with the common leading offset.
    //
    // Certified values:
    // https://www.itl.nist.gov/div898/strd/univ/certvalues/numacc2.html
    // https://www.itl.nist.gov/div898/strd/univ/certvalues/numacc3.html
    // https://www.itl.nist.gov/div898/strd/univ/certvalues/numacc4.html
    let cases = [
        (1.2, 5.0e-15),
        (1_000_000.2, 4.0e-10),
        (10_000_000.2, 6.0e-9),
    ];

    for (certified_mean, std_dev_tolerance) in cases {
        let (n, mean, std_dev) = sample_summary(nist_numacc(certified_mean));
        assert_eq!(n, 1001);
        assert_close(mean, certified_mean, 2.0 * f64::EPSILON);
        assert_close(std_dev, 0.1, std_dev_tolerance);
    }
}

#[test]
fn nist_norris_linear_regression_matches_certified_values() {
    // NIST StRD Norris:
    // https://www.itl.nist.gov/div898/strd/lls/data/LINKS/DATA/Norris.dat
    let y = vec![
        0.1, 338.8, 118.1, 888.0, 9.2, 228.1, 668.5, 998.5, 449.1, 778.9, 559.2, 0.3, 0.1, 778.1,
        668.8, 339.3, 448.9, 10.8, 557.7, 228.3, 998.0, 888.8, 119.6, 0.3, 0.6, 557.6, 339.3,
        888.0, 998.5, 778.9, 10.2, 117.6, 228.9, 668.4, 449.2, 0.2,
    ];
    let x = vec![
        0.2, 337.4, 118.2, 884.6, 10.1, 226.5, 666.3, 996.3, 448.6, 777.0, 558.2, 0.4, 0.6, 775.5,
        666.9, 338.0, 447.5, 11.6, 556.0, 228.1, 995.8, 887.6, 120.2, 0.3, 0.3, 556.8, 339.1,
        887.2, 999.0, 779.0, 11.1, 118.3, 229.2, 669.1, 448.9, 0.5,
    ];

    match (StatsRequest::LinearRegression {
        x: XInput::Single(x),
        y,
    })
    .evaluate()
    {
        StatsResponse::Regression {
            intercept,
            slope,
            r_squared,
            ..
        } => {
            assert_close(intercept, -0.262_323_073_774_029, 2.0e-12);
            assert_close(slope, 1.002_116_818_020_45, 2.0e-14);
            assert_close(r_squared, 0.999_993_745_883_712, 2.0e-14);
        }
        other => panic!("expected regression response, got {other:?}"),
    }
}

#[test]
fn finite_mean_does_not_overflow_with_a_nonrepresentable_raw_sum() {
    let (_, mean, std_dev) = sample_summary(vec![f64::MAX, f64::MAX]);

    assert_eq!(mean, f64::MAX);
    assert_eq!(std_dev, 0.0);
}
