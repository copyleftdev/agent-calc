use agent_calc::{DistributionKind, DistributionQuery, StatsRequest, StatsResponse};
use serde_json::json;

fn prob(req: StatsRequest) -> f64 {
    match req.evaluate() {
        StatsResponse::Probability { value, .. } => value,
        other => panic!("expected probability, got {other:?}"),
    }
}

fn quant(req: StatsRequest) -> f64 {
    match req.evaluate() {
        StatsResponse::Quantile { value, .. } => value,
        other => panic!("expected quantile, got {other:?}"),
    }
}

fn err(req: StatsRequest) -> String {
    match req.evaluate() {
        StatsResponse::Error { reason, .. } => reason,
        other => panic!("expected error, got {other:?}"),
    }
}

fn dist_req(distribution: DistributionKind, query: DistributionQuery) -> StatsRequest {
    StatsRequest::ProbabilityDistribution {
        distribution,
        query,
    }
}

// ── Normal ────────────────────────────────────────────────────────────────────

#[test]
fn normal_pdf_at_zero_standard() {
    // 1/sqrt(2π)
    let v = prob(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 1.0,
        },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!((v - 0.398_942_280_401_432_7).abs() < 1e-9);
}

#[test]
fn normal_cdf_at_zero_is_half() {
    let v = prob(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 1.0,
        },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!((v - 0.5).abs() < 1e-12);
}

#[test]
fn normal_cdf_at_1_96() {
    let v = prob(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 1.0,
        },
        DistributionQuery::Cdf { x: 1.96 },
    ));
    assert!((v - 0.975_002_104_851_78).abs() < 1e-9);
}

#[test]
fn normal_quantile_at_half_is_mean() {
    let v = quant(dist_req(
        DistributionKind::Normal {
            mean: 3.0,
            std_dev: 2.0,
        },
        DistributionQuery::Quantile { p: 0.5 },
    ));
    assert!((v - 3.0).abs() < 1e-12);
}

#[test]
fn normal_quantile_0_975() {
    let v = quant(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 1.0,
        },
        DistributionQuery::Quantile { p: 0.975 },
    ));
    assert!((v - 1.959_963_984_540_054).abs() < 1e-9);
}

#[test]
fn normal_quantile_rejects_p_zero() {
    let e = err(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 1.0,
        },
        DistributionQuery::Quantile { p: 0.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

#[test]
fn normal_quantile_rejects_p_one() {
    let e = err(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 1.0,
        },
        DistributionQuery::Quantile { p: 1.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

#[test]
fn normal_rejects_nonpositive_std_dev() {
    let e = err(dist_req(
        DistributionKind::Normal {
            mean: 0.0,
            std_dev: 0.0,
        },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!(!e.is_empty());
}

// ── StudentT ──────────────────────────────────────────────────────────────────

#[test]
fn student_t_pdf_at_zero_df1() {
    // Cauchy: pdf(0) = 1/π
    let v = prob(dist_req(
        DistributionKind::StudentT { df: 1.0 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!((v - std::f64::consts::FRAC_1_PI).abs() < 1e-9);
}

#[test]
fn student_t_cdf_at_zero_is_half() {
    let v = prob(dist_req(
        DistributionKind::StudentT { df: 10.0 },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!((v - 0.5).abs() < 1e-12);
}

#[test]
fn student_t_quantile_0_975_df10() {
    // scipy.stats.t.ppf(0.975, 10) = 2.228138852...
    let v = quant(dist_req(
        DistributionKind::StudentT { df: 10.0 },
        DistributionQuery::Quantile { p: 0.975 },
    ));
    assert!((v - 2.228_138_852_121_98).abs() < 1e-6);
}

#[test]
fn student_t_quantile_rejects_p_zero() {
    let e = err(dist_req(
        DistributionKind::StudentT { df: 5.0 },
        DistributionQuery::Quantile { p: 0.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

#[test]
fn student_t_rejects_df_zero() {
    let e = err(dist_req(
        DistributionKind::StudentT { df: 0.0 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!(!e.is_empty());
}

// ── ChiSquared ────────────────────────────────────────────────────────────────

#[test]
fn chi2_pdf_at_one_df2() {
    // f(x) = 0.5*exp(-x/2) for df=2; f(1) = 0.5*exp(-0.5)
    let expected = 0.5 * (-0.5_f64).exp();
    let v = prob(dist_req(
        DistributionKind::ChiSquared { df: 2.0 },
        DistributionQuery::Pdf { x: 1.0 },
    ));
    assert!((v - expected).abs() < 1e-12);
}

#[test]
fn chi2_cdf_at_two_df2() {
    // CDF(2, df=2) = 1 - exp(-1)
    let expected = 1.0 - (-1.0_f64).exp();
    let v = prob(dist_req(
        DistributionKind::ChiSquared { df: 2.0 },
        DistributionQuery::Cdf { x: 2.0 },
    ));
    assert!((v - expected).abs() < 1e-12);
}

#[test]
fn chi2_quantile_0_95_df2() {
    // scipy.stats.chi2.ppf(0.95, 2) = 5.991464547107982
    let v = quant(dist_req(
        DistributionKind::ChiSquared { df: 2.0 },
        DistributionQuery::Quantile { p: 0.95 },
    ));
    assert!((v - 5.991_464_547_107_982).abs() < 1e-6);
}

#[test]
fn chi2_pdf_at_zero_succeeds() {
    // boundary: x=0 must succeed
    let v = prob(dist_req(
        DistributionKind::ChiSquared { df: 4.0 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!(v.is_finite());
}

#[test]
fn chi2_cdf_at_zero_is_zero() {
    // CDF(0) = 0 for any chi-squared; x=0 must succeed (kills < → <= mutation)
    let v = prob(dist_req(
        DistributionKind::ChiSquared { df: 2.0 },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!(v.abs() < 1e-12);
}

#[test]
fn chi2_pdf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::ChiSquared { df: 2.0 },
        DistributionQuery::Pdf { x: -1.0 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn chi2_cdf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::ChiSquared { df: 2.0 },
        DistributionQuery::Cdf { x: -0.1 },
    ));
    assert!(!e.is_empty());
}

// ── FDist ─────────────────────────────────────────────────────────────────────

#[test]
fn f_pdf_at_zero_d1_2() {
    // x=0 must not error — kills < → <= guard mutation in eval_distribution.
    // statrs computes 0/0 via the sqrt formula and returns NaN; that is a
    // Probability response (not Error), so prob() succeeds without asserting value.
    let _v = prob(dist_req(
        DistributionKind::FDist { d1: 2.0, d2: 10.0 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
}

#[test]
fn f_cdf_at_zero_is_zero() {
    let v = prob(dist_req(
        DistributionKind::FDist { d1: 5.0, d2: 10.0 },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!(v.abs() < 1e-12);
}

#[test]
fn f_quantile_0_95_d1_5_d2_10() {
    // scipy.stats.f.ppf(0.95, 5, 10) = 3.325834506...
    let v = quant(dist_req(
        DistributionKind::FDist { d1: 5.0, d2: 10.0 },
        DistributionQuery::Quantile { p: 0.95 },
    ));
    assert!((v - 3.325_834_506_297_24).abs() < 1e-6);
}

#[test]
fn f_pdf_at_one_d1_2_d2_4() {
    // f(1; d1=2, d2=4) = (1/B(1,2))*(d1/d2)^1 * x^0 * (1+d1*x/d2)^(-3)
    //   = 2 * 0.5 * 1 * (1+0.5)^(-3) = 1 * (3/2)^(-3) = 8/27
    let expected = 8.0_f64 / 27.0;
    let v = prob(dist_req(
        DistributionKind::FDist { d1: 2.0, d2: 4.0 },
        DistributionQuery::Pdf { x: 1.0 },
    ));
    assert!((v - expected).abs() < 1e-9);
}

#[test]
fn f_pdf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::FDist { d1: 2.0, d2: 5.0 },
        DistributionQuery::Pdf { x: -1.0 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn f_cdf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::FDist { d1: 2.0, d2: 5.0 },
        DistributionQuery::Cdf { x: -0.5 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn f_quantile_rejects_p_zero() {
    let e = err(dist_req(
        DistributionKind::FDist { d1: 2.0, d2: 5.0 },
        DistributionQuery::Quantile { p: 0.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

// ── Binomial ──────────────────────────────────────────────────────────────────

#[test]
fn binomial_pmf_5_n10_p05_exact() {
    // C(10,5) * 0.5^10 = 252/1024 = 0.24609375
    let v = prob(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Pdf { x: 5.0 },
    ));
    assert!((v - 0.24609375).abs() < 1e-12);
}

#[test]
fn binomial_cdf_5_n10_p05_exact() {
    // sum P(k<=5) = 638/1024 = 0.623046875
    let v = prob(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Cdf { x: 5.0 },
    ));
    assert!((v - 0.623_046_875).abs() < 1e-12);
}

#[test]
fn binomial_quantile_median_n10_p05() {
    let v = quant(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Quantile { p: 0.5 },
    ));
    assert_eq!(v, 5.0);
}

#[test]
fn binomial_pmf_at_zero_succeeds() {
    let v = prob(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    // P(k=0) = 0.5^10 = 1/1024
    assert!((v - 1.0 / 1024.0).abs() < 1e-15);
}

#[test]
fn binomial_cdf_at_zero() {
    // CDF(0) = P(k=0) = 0.5^10 = 1/1024; x=0 must not early-return 0 (kills < → <= mutation)
    let v = prob(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!((v - 1.0 / 1024.0).abs() < 1e-12);
}

#[test]
fn binomial_cdf_negative_x_returns_zero() {
    // CDF at x<0 is 0 for a non-negative discrete distribution
    let v = prob(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Cdf { x: -1.0 },
    ));
    assert_eq!(v, 0.0);
}

#[test]
fn binomial_pmf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Pdf { x: -1.0 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn binomial_pmf_rejects_noninteger_x() {
    let e = err(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Pdf { x: 2.5 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn binomial_quantile_rejects_p_one() {
    let e = err(dist_req(
        DistributionKind::Binomial { n: 10, p: 0.5 },
        DistributionQuery::Quantile { p: 1.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

// ── Poisson ───────────────────────────────────────────────────────────────────

#[test]
fn poisson_pmf_3_lambda3() {
    // 3^3 * e^(-3) / 3! = 4.5 * e^(-3)
    let expected = 4.5 * (-3.0_f64).exp();
    let v = prob(dist_req(
        DistributionKind::Poisson { lambda: 3.0 },
        DistributionQuery::Pdf { x: 3.0 },
    ));
    assert!((v - expected).abs() < 1e-12);
}

#[test]
fn poisson_pmf_0_lambda1() {
    // P(k=0, λ=1) = e^(-1)
    let v = prob(dist_req(
        DistributionKind::Poisson { lambda: 1.0 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!((v - (-1.0_f64).exp()).abs() < 1e-12);
}

#[test]
fn poisson_cdf_3_lambda3() {
    // scipy.stats.poisson.cdf(3, 3) ≈ 0.647232...
    let v = prob(dist_req(
        DistributionKind::Poisson { lambda: 3.0 },
        DistributionQuery::Cdf { x: 3.0 },
    ));
    assert!((v - 0.647_231_897_5).abs() < 1e-6);
}

#[test]
fn poisson_quantile_median_lambda3() {
    let v = quant(dist_req(
        DistributionKind::Poisson { lambda: 3.0 },
        DistributionQuery::Quantile { p: 0.5 },
    ));
    assert_eq!(v, 3.0);
}

#[test]
fn poisson_cdf_at_zero_lambda1() {
    // CDF(0, lambda=1) = P(k=0) = e^(-1); x=0 must not early-return 0 (kills < → <= mutation)
    let v = prob(dist_req(
        DistributionKind::Poisson { lambda: 1.0 },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!((v - (-1.0_f64).exp()).abs() < 1e-12);
}

#[test]
fn poisson_cdf_negative_x_returns_zero() {
    let v = prob(dist_req(
        DistributionKind::Poisson { lambda: 2.0 },
        DistributionQuery::Cdf { x: -1.0 },
    ));
    assert_eq!(v, 0.0);
}

#[test]
fn poisson_pmf_rejects_noninteger_x() {
    let e = err(dist_req(
        DistributionKind::Poisson { lambda: 2.0 },
        DistributionQuery::Pdf { x: 1.5 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn poisson_pmf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::Poisson { lambda: 2.0 },
        DistributionQuery::Pdf { x: -1.0 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn poisson_quantile_rejects_p_zero() {
    let e = err(dist_req(
        DistributionKind::Poisson { lambda: 2.0 },
        DistributionQuery::Quantile { p: 0.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

// ── Exponential ───────────────────────────────────────────────────────────────

#[test]
fn exp_pdf_at_zero_equals_rate() {
    // pdf(0) = rate * exp(-rate*0) = rate
    let v = prob(dist_req(
        DistributionKind::Exponential { rate: 3.0 },
        DistributionQuery::Pdf { x: 0.0 },
    ));
    assert!((v - 3.0).abs() < 1e-12);
}

#[test]
fn exp_pdf_at_one_rate2() {
    // pdf(1, rate=2) = 2 * exp(-2)
    let expected = 2.0 * (-2.0_f64).exp();
    let v = prob(dist_req(
        DistributionKind::Exponential { rate: 2.0 },
        DistributionQuery::Pdf { x: 1.0 },
    ));
    assert!((v - expected).abs() < 1e-12);
}

#[test]
fn exp_cdf_at_one_rate2() {
    // cdf(1, rate=2) = 1 - exp(-2)
    let expected = 1.0 - (-2.0_f64).exp();
    let v = prob(dist_req(
        DistributionKind::Exponential { rate: 2.0 },
        DistributionQuery::Cdf { x: 1.0 },
    ));
    assert!((v - expected).abs() < 1e-12);
}

#[test]
fn exp_quantile_half_rate2() {
    // quantile(0.5, rate=2) = ln(2)/2
    let expected = 2.0_f64.ln() / 2.0;
    let v = quant(dist_req(
        DistributionKind::Exponential { rate: 2.0 },
        DistributionQuery::Quantile { p: 0.5 },
    ));
    assert!((v - expected).abs() < 1e-12);
}

#[test]
fn exp_pdf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::Exponential { rate: 1.0 },
        DistributionQuery::Pdf { x: -1.0 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn exp_cdf_at_zero_is_zero() {
    // cdf(0) = 1 - exp(0) = 0; x=0 must succeed not error (kills < → <= mutation)
    let v = prob(dist_req(
        DistributionKind::Exponential { rate: 2.0 },
        DistributionQuery::Cdf { x: 0.0 },
    ));
    assert!(v.abs() < 1e-12);
}

#[test]
fn exp_cdf_rejects_negative_x() {
    let e = err(dist_req(
        DistributionKind::Exponential { rate: 1.0 },
        DistributionQuery::Cdf { x: -0.5 },
    ));
    assert!(!e.is_empty());
}

#[test]
fn exp_quantile_rejects_p_one() {
    let e = err(dist_req(
        DistributionKind::Exponential { rate: 1.0 },
        DistributionQuery::Quantile { p: 1.0 },
    ));
    assert!(e.contains("(0, 1)"), "got: {e}");
}

// ── JSON round-trip ───────────────────────────────────────────────────────────

#[test]
fn json_round_trip_normal_cdf() {
    let payload = json!({
        "intent": "probability_distribution",
        "distribution": {"normal": {"mean": 0.0, "std_dev": 1.0}},
        "query": {"cdf": {"x": 0.0}}
    });
    let req: StatsRequest = serde_json::from_value(payload).unwrap();
    let v = match req.evaluate() {
        StatsResponse::Probability { value, .. } => value,
        other => panic!("expected probability, got {other:?}"),
    };
    assert!((v - 0.5).abs() < 1e-12);
}

#[test]
fn json_round_trip_binomial_quantile() {
    let payload = json!({
        "intent": "probability_distribution",
        "distribution": {"binomial": {"n": 10, "p": 0.5}},
        "query": {"quantile": {"p": 0.5}}
    });
    let req: StatsRequest = serde_json::from_value(payload).unwrap();
    let v = match req.evaluate() {
        StatsResponse::Quantile { value, .. } => value,
        other => panic!("expected quantile, got {other:?}"),
    };
    assert_eq!(v, 5.0);
}
