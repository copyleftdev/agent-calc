use agent_calc::{StatsRequest, StatsResponse};
use proptest::prelude::*;

fn probability(response: StatsResponse) -> f64 {
    match response {
        StatsResponse::Probability { value, .. } => value,
        StatsResponse::Error { reason, .. } => panic!("expected probability response: {reason}"),
        other => panic!("expected probability response, got {other:?}"),
    }
}

fn quantile(response: StatsResponse) -> f64 {
    match response {
        StatsResponse::Quantile { value, .. } => value,
        StatsResponse::Error { reason, .. } => panic!("expected quantile response: {reason}"),
        other => panic!("expected quantile response, got {other:?}"),
    }
}

proptest! {
    #[test]
    fn normal_cdf_is_monotonic(
        mean in -1000.0f64..1000.0,
        std_dev in 0.001f64..1000.0,
        a in -1000.0f64..1000.0,
        b in -1000.0f64..1000.0,
    ) {
        prop_assume!(mean.is_finite() && std_dev.is_finite() && a.is_finite() && b.is_finite());
        let lo = a.min(b);
        let hi = a.max(b);

        let left = probability(StatsRequest::NormalCdf { mean, std_dev, x: lo }.evaluate());
        let right = probability(StatsRequest::NormalCdf { mean, std_dev, x: hi }.evaluate());

        prop_assert!(left <= right);
        prop_assert!((0.0..=1.0).contains(&left));
        prop_assert!((0.0..=1.0).contains(&right));
    }

    #[test]
    fn normal_quantile_round_trips_through_cdf(
        mean in -1000.0f64..1000.0,
        std_dev in 0.001f64..1000.0,
        p in 0.001f64..0.999,
    ) {
        prop_assume!(mean.is_finite() && std_dev.is_finite() && p.is_finite());

        let x = quantile(StatsRequest::NormalQuantile { mean, std_dev, p }.evaluate());
        let p2 = probability(StatsRequest::NormalCdf { mean, std_dev, x }.evaluate());

        prop_assert!((p2 - p).abs() <= 1e-10);
    }

    #[test]
    fn sample_variance_is_non_negative(values in proptest::collection::vec(-1.0e6f64..1.0e6, 1..32)) {
        prop_assume!(values.iter().all(|v| v.is_finite()));

        match (StatsRequest::DescribeSample { values }).evaluate() {
            StatsResponse::SampleSummary { variance, std_dev, min, max, .. } => {
                prop_assert!(variance >= 0.0);
                prop_assert!(std_dev >= 0.0);
                prop_assert!((std_dev * std_dev - variance).abs() <= variance.max(1.0) * 1e-12);
                prop_assert!(min <= max);
            }
            other => panic!("expected sample summary, got {other:?}"),
        }
    }

    #[test]
    fn binomial_cdf_is_monotonic(n in 1u64..100, p in 0.0f64..1.0) {
        prop_assume!(p.is_finite());
        let low_k = n / 3;
        let high_k = (2 * n) / 3;

        let low = probability(StatsRequest::BinomialCdf { n, p, k: low_k }.evaluate());
        let high = probability(StatsRequest::BinomialCdf { n, p, k: high_k }.evaluate());

        prop_assert!(low <= high);
        prop_assert!((0.0..=1.0).contains(&low));
        prop_assert!((0.0..=1.0).contains(&high));
    }
}
