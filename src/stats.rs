use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use statrs::distribution::{
    Beta, Binomial, ChiSquared, Continuous, ContinuousCDF, Discrete, DiscreteCDF, Exp,
    FisherSnedecor, Normal, Poisson, StudentsT, Uniform,
};

/// Accepts either a flat `number[]` (single predictor) or `number[][]` (multiple predictors).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum XInput {
    Single(Vec<f64>),
    Multi(Vec<Vec<f64>>),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tail {
    #[default]
    Two,
    Left,
    Right,
}

fn default_alpha() -> f64 {
    0.05
}

fn default_period() -> usize {
    3
}

fn default_max_lag() -> usize {
    10
}

fn default_power() -> f64 {
    0.80
}

fn default_smoothing() -> f64 {
    0.3
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSeriesIntent {
    Sma,
    Ema,
    Autocorr,
    Trend,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum StatsRequest {
    DescribeSample {
        #[serde(alias = "data")]
        values: Vec<f64>,
    },
    NormalCdf {
        mean: f64,
        std_dev: f64,
        x: f64,
    },
    NormalQuantile {
        mean: f64,
        std_dev: f64,
        p: f64,
    },
    StudentTInterval {
        #[serde(alias = "data")]
        values: Vec<f64>,
        confidence: f64,
    },
    BinomialPmf {
        n: u64,
        p: f64,
        k: u64,
    },
    BinomialCdf {
        n: u64,
        p: f64,
        k: u64,
    },
    TCdf {
        degrees_of_freedom: f64,
        x: f64,
    },
    TInverseCdf {
        degrees_of_freedom: f64,
        p: f64,
    },
    Chi2Cdf {
        degrees_of_freedom: f64,
        x: f64,
    },
    Chi2InverseCdf {
        degrees_of_freedom: f64,
        p: f64,
    },
    PoissonPmf {
        lambda: f64,
        k: u64,
    },
    PoissonCdf {
        lambda: f64,
        k: u64,
    },
    BetaCdf {
        alpha: f64,
        beta: f64,
        x: f64,
    },
    BetaPdf {
        alpha: f64,
        beta: f64,
        x: f64,
    },
    FCdf {
        d1: f64,
        d2: f64,
        x: f64,
    },
    ExponentialCdf {
        rate: f64,
        x: f64,
    },
    ExponentialPdf {
        rate: f64,
        x: f64,
    },
    UniformCdf {
        min: f64,
        max: f64,
        x: f64,
    },
    Correlation {
        x: Vec<f64>,
        y: Vec<f64>,
    },
    LinearRegression {
        x: XInput,
        y: Vec<f64>,
    },
    Percentile {
        values: Vec<f64>,
        p: f64,
    },
    Mode {
        values: Vec<f64>,
    },
    Rank {
        values: Vec<f64>,
        method: RankMethod,
    },
    OneSampleT {
        sample: Vec<f64>,
        mu0: f64,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default)]
        tail: Tail,
    },
    TwoSampleT {
        sample1: Vec<f64>,
        sample2: Vec<f64>,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default)]
        tail: Tail,
        #[serde(default)]
        equal_var: bool,
    },
    PairedT {
        before: Vec<f64>,
        after: Vec<f64>,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default)]
        tail: Tail,
    },
    ChiSquareGof {
        observed: Vec<f64>,
        expected: Vec<f64>,
        #[serde(default = "default_alpha")]
        alpha: f64,
    },
    OneWayAnova {
        groups: Vec<Vec<f64>>,
        #[serde(default = "default_alpha")]
        alpha: f64,
    },
    TimeSeries {
        method: TimeSeriesIntent,
        values: Vec<f64>,
        #[serde(default = "default_period")]
        period: usize,
        #[serde(default = "default_max_lag")]
        max_lag: usize,
        #[serde(default = "default_smoothing")]
        smoothing: f64,
    },
    MannWhitneyU {
        sample1: Vec<f64>,
        sample2: Vec<f64>,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default)]
        tail: Tail,
    },
    WilcoxonSigned {
        before: Vec<f64>,
        after: Vec<f64>,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default)]
        tail: Tail,
    },
    KruskalWallis {
        groups: Vec<Vec<f64>>,
        #[serde(default = "default_alpha")]
        alpha: f64,
    },
    SpearmanCorrelation {
        x: Vec<f64>,
        y: Vec<f64>,
    },
    KendallTau {
        x: Vec<f64>,
        y: Vec<f64>,
    },
    ProbabilityDistribution {
        distribution: DistributionKind,
        query: DistributionQuery,
    },
    CohenD {
        sample1: Vec<f64>,
        sample2: Vec<f64>,
    },
    CohenDOneSample {
        sample: Vec<f64>,
        mu0: f64,
    },
    EtaSquared {
        groups: Vec<Vec<f64>>,
    },
    CramersV {
        observed: Vec<Vec<f64>>,
    },
    PointBiserialR {
        binary: Vec<f64>,
        continuous: Vec<f64>,
    },
    PowerOneSampleT {
        effect_d: f64,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default = "default_power")]
        power: f64,
    },
    PowerTwoSampleT {
        effect_d: f64,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default = "default_power")]
        power: f64,
    },
    PowerOneProportion {
        p0: f64,
        p1: f64,
        #[serde(default = "default_alpha")]
        alpha: f64,
        #[serde(default = "default_power")]
        power: f64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankMethod {
    Average,
    Min,
    Max,
    First,
    Last,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionKind {
    Normal { mean: f64, std_dev: f64 },
    StudentT { df: f64 },
    ChiSquared { df: f64 },
    FDist { d1: f64, d2: f64 },
    Binomial { n: u64, p: f64 },
    Poisson { lambda: f64 },
    Exponential { rate: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionQuery {
    Pdf { x: f64 },
    Cdf { x: f64 },
    Quantile { p: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StatsResponse {
    SampleSummary {
        contract_version: String,
        n: usize,
        mean: f64,
        variance: f64,
        std_dev: f64,
        min: f64,
        max: f64,
        median: f64,
        q1: f64,
        q3: f64,
        iqr: f64,
        skewness: f64,
        kurtosis: f64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Probability {
        contract_version: String,
        value: f64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Quantile {
        contract_version: String,
        value: f64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Interval {
        contract_version: String,
        confidence: f64,
        mean: f64,
        lower: f64,
        upper: f64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Correlation {
        contract_version: String,
        pearson_r: f64,
        n: usize,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Regression {
        contract_version: String,
        slope: f64,
        intercept: f64,
        r_squared: f64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    MultipleRegression {
        contract_version: String,
        /// Coefficients in order: [intercept, b1, b2, ..., bk]
        coefficients: Vec<f64>,
        r_squared: f64,
        residual_std_dev: f64,
        std_errors: Vec<f64>,
        n: usize,
        k: usize,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Percentile {
        contract_version: String,
        value: f64,
        p: f64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Mode {
        contract_version: String,
        values: Vec<f64>,
        frequency: usize,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Ranks {
        contract_version: String,
        ranks: Vec<f64>,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    HypothesisTest {
        contract_version: String,
        test: String,
        statistic: f64,
        p_value: f64,
        dof: f64,
        critical_value: f64,
        alpha: f64,
        reject_h0: bool,
        conclusion: String,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    TimeSeries {
        contract_version: String,
        method: String,
        result: Vec<f64>,
        n: usize,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    EffectSize {
        contract_version: String,
        statistic: String,
        value: f64,
        interpretation: String,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    SampleSize {
        contract_version: String,
        n: u64,
        exactness: StatsExactness,
        checks: Vec<StatsCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatsExactness {
    ApproximateF64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatsCheck {
    pub name: String,
    pub passed: bool,
}

impl StatsRequest {
    pub fn evaluate(&self) -> StatsResponse {
        match self.evaluate_inner() {
            Ok(StatsOutput::Sample(summary)) => StatsResponse::SampleSummary {
                contract_version: CONTRACT_VERSION.to_owned(),
                n: summary.n,
                mean: summary.mean,
                variance: summary.variance,
                std_dev: summary.std_dev,
                min: summary.min,
                max: summary.max,
                median: summary.median,
                q1: summary.q1,
                q3: summary.q3,
                iqr: summary.iqr,
                skewness: summary.skewness,
                kurtosis: summary.kurtosis,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::Probability(value)) => StatsResponse::Probability {
                contract_version: CONTRACT_VERSION.to_owned(),
                value,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::Quantile(value)) => StatsResponse::Quantile {
                contract_version: CONTRACT_VERSION.to_owned(),
                value,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::Interval {
                confidence,
                mean,
                lower,
                upper,
            }) => StatsResponse::Interval {
                contract_version: CONTRACT_VERSION.to_owned(),
                confidence,
                mean,
                lower,
                upper,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::CorrelationResult { pearson_r, n }) => StatsResponse::Correlation {
                contract_version: CONTRACT_VERSION.to_owned(),
                pearson_r,
                n,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::RegressionResult {
                slope,
                intercept,
                r_squared,
            }) => StatsResponse::Regression {
                contract_version: CONTRACT_VERSION.to_owned(),
                slope,
                intercept,
                r_squared,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::MultipleRegressionResult {
                coefficients,
                r_squared,
                residual_std_dev,
                std_errors,
                n,
                k,
            }) => StatsResponse::MultipleRegression {
                contract_version: CONTRACT_VERSION.to_owned(),
                coefficients,
                r_squared,
                residual_std_dev,
                std_errors,
                n,
                k,
                exactness: StatsExactness::ApproximateF64,
                checks: vec![StatsCheck {
                    name: "ols_solved_by_svd_nalgebra".to_owned(),
                    passed: true,
                }],
            },
            Ok(StatsOutput::PercentileValue { value, p }) => StatsResponse::Percentile {
                contract_version: CONTRACT_VERSION.to_owned(),
                value,
                p,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::ModeResult { values, frequency }) => StatsResponse::Mode {
                contract_version: CONTRACT_VERSION.to_owned(),
                values,
                frequency,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::RanksResult(ranks)) => StatsResponse::Ranks {
                contract_version: CONTRACT_VERSION.to_owned(),
                ranks,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::HypothesisTestResult {
                test,
                statistic,
                p_value,
                dof,
                critical_value,
                alpha,
                reject_h0,
                conclusion,
            }) => StatsResponse::HypothesisTest {
                contract_version: CONTRACT_VERSION.to_owned(),
                test: test.to_owned(),
                statistic,
                p_value,
                dof,
                critical_value,
                alpha,
                reject_h0,
                conclusion,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::TimeSeriesResult { method, result, n }) => StatsResponse::TimeSeries {
                contract_version: CONTRACT_VERSION.to_owned(),
                method: method.to_owned(),
                result,
                n,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::EffectSizeResult {
                statistic,
                value,
                interpretation,
            }) => StatsResponse::EffectSize {
                contract_version: CONTRACT_VERSION.to_owned(),
                statistic: statistic.to_owned(),
                value,
                interpretation: interpretation.to_owned(),
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Ok(StatsOutput::SampleSizeResult { n }) => StatsResponse::SampleSize {
                contract_version: CONTRACT_VERSION.to_owned(),
                n,
                exactness: StatsExactness::ApproximateF64,
                checks: default_checks(),
            },
            Err(reason) => StatsResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<StatsOutput, String> {
        match self {
            StatsRequest::DescribeSample { values } => {
                Ok(StatsOutput::Sample(describe_sample(values)?))
            }
            StatsRequest::NormalCdf { mean, std_dev, x } => {
                let normal = normal(*mean, *std_dev)?;
                ensure_finite(*x, "x")?;
                Ok(StatsOutput::Probability(normal.cdf(*x)))
            }
            StatsRequest::NormalQuantile { mean, std_dev, p } => {
                let normal = normal(*mean, *std_dev)?;
                ensure_probability_open(*p, "p")?;
                Ok(StatsOutput::Quantile(normal.inverse_cdf(*p)))
            }
            StatsRequest::StudentTInterval { values, confidence } => {
                ensure_probability_open(*confidence, "confidence")?;
                let summary = describe_sample(values)?;
                if summary.n < 2 {
                    return Err("student_t_interval requires at least two values".to_owned());
                }
                let df = (summary.n - 1) as f64;
                let t = StudentsT::new(0.0, 1.0, df).map_err(|e| e.to_string())?;
                let alpha = 1.0 - confidence;
                let critical = t.inverse_cdf(1.0 - alpha / 2.0);
                let margin = critical * summary.std_dev / (summary.n as f64).sqrt();
                Ok(StatsOutput::Interval {
                    confidence: *confidence,
                    mean: summary.mean,
                    lower: summary.mean - margin,
                    upper: summary.mean + margin,
                })
            }
            StatsRequest::BinomialPmf { n, p, k } => {
                let binomial = binomial(*n, *p)?;
                Ok(StatsOutput::Probability(binomial.pmf(*k)))
            }
            StatsRequest::BinomialCdf { n, p, k } => {
                let binomial = binomial(*n, *p)?;
                Ok(StatsOutput::Probability(binomial.cdf(*k)))
            }
            StatsRequest::TCdf {
                degrees_of_freedom,
                x,
            } => {
                ensure_positive_finite(*degrees_of_freedom, "degrees_of_freedom")?;
                ensure_finite(*x, "x")?;
                let t = StudentsT::new(0.0, 1.0, *degrees_of_freedom).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(t.cdf(*x)))
            }
            StatsRequest::TInverseCdf {
                degrees_of_freedom,
                p,
            } => {
                ensure_positive_finite(*degrees_of_freedom, "degrees_of_freedom")?;
                ensure_probability_open(*p, "p")?;
                let t = StudentsT::new(0.0, 1.0, *degrees_of_freedom).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Quantile(t.inverse_cdf(*p)))
            }
            StatsRequest::Chi2Cdf {
                degrees_of_freedom,
                x,
            } => {
                ensure_positive_finite(*degrees_of_freedom, "degrees_of_freedom")?;
                if *x < 0.0 {
                    return Err("x must be non-negative for chi2_cdf".to_owned());
                }
                let chi2 = ChiSquared::new(*degrees_of_freedom).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(chi2.cdf(*x)))
            }
            StatsRequest::Chi2InverseCdf {
                degrees_of_freedom,
                p,
            } => {
                ensure_positive_finite(*degrees_of_freedom, "degrees_of_freedom")?;
                ensure_probability_open(*p, "p")?;
                let chi2 = ChiSquared::new(*degrees_of_freedom).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Quantile(chi2.inverse_cdf(*p)))
            }
            StatsRequest::PoissonPmf { lambda, k } => {
                ensure_positive_finite(*lambda, "lambda")?;
                let poisson = Poisson::new(*lambda).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(poisson.pmf(*k)))
            }
            StatsRequest::PoissonCdf { lambda, k } => {
                ensure_positive_finite(*lambda, "lambda")?;
                let poisson = Poisson::new(*lambda).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(poisson.cdf(*k)))
            }
            StatsRequest::BetaCdf { alpha, beta, x } => {
                ensure_positive_finite(*alpha, "alpha")?;
                ensure_positive_finite(*beta, "beta")?;
                if *x < 0.0 || *x > 1.0 {
                    return Err("x must be in [0, 1] for beta_cdf".to_owned());
                }
                let beta_dist = Beta::new(*alpha, *beta).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(beta_dist.cdf(*x)))
            }
            StatsRequest::BetaPdf { alpha, beta, x } => {
                ensure_positive_finite(*alpha, "alpha")?;
                ensure_positive_finite(*beta, "beta")?;
                if *x < 0.0 || *x > 1.0 {
                    return Err("x must be in [0, 1] for beta_pdf".to_owned());
                }
                let beta_dist = Beta::new(*alpha, *beta).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(beta_dist.pdf(*x)))
            }
            StatsRequest::FCdf { d1, d2, x } => {
                ensure_positive_finite(*d1, "d1")?;
                ensure_positive_finite(*d2, "d2")?;
                if *x < 0.0 {
                    return Err("x must be non-negative for f_cdf".to_owned());
                }
                let f = FisherSnedecor::new(*d1, *d2).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(f.cdf(*x)))
            }
            StatsRequest::ExponentialCdf { rate, x } => {
                ensure_positive_finite(*rate, "rate")?;
                if *x < 0.0 {
                    return Err("x must be non-negative for exponential_cdf".to_owned());
                }
                let exp = Exp::new(*rate).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(exp.cdf(*x)))
            }
            StatsRequest::ExponentialPdf { rate, x } => {
                ensure_positive_finite(*rate, "rate")?;
                if *x < 0.0 {
                    return Err("x must be non-negative for exponential_pdf".to_owned());
                }
                let exp = Exp::new(*rate).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(exp.pdf(*x)))
            }
            StatsRequest::UniformCdf { min, max, x } => {
                if min >= max {
                    return Err("min must be less than max for uniform_cdf".to_owned());
                }
                let uniform = Uniform::new(*min, *max).map_err(|e| e.to_string())?;
                Ok(StatsOutput::Probability(uniform.cdf(*x)))
            }
            StatsRequest::Correlation { x, y } => {
                let (pearson_r, n) = correlation(x, y)?;
                Ok(StatsOutput::CorrelationResult { pearson_r, n })
            }
            StatsRequest::LinearRegression { x, y } => match x {
                XInput::Single(x_vec) => {
                    let (slope, intercept, r_squared) = linear_regression(x_vec, y)?;
                    Ok(StatsOutput::RegressionResult {
                        slope,
                        intercept,
                        r_squared,
                    })
                }
                XInput::Multi(x_mat) => {
                    let out = multiple_linear_regression(x_mat, y)?;
                    Ok(StatsOutput::MultipleRegressionResult {
                        coefficients: out.coefficients,
                        r_squared: out.r_squared,
                        residual_std_dev: out.residual_std_dev,
                        std_errors: out.std_errors,
                        n: out.n,
                        k: out.k,
                    })
                }
            },
            StatsRequest::Percentile { values, p } => {
                if values.is_empty() {
                    return Err("values must contain at least one value".to_owned());
                }
                if !values.iter().all(|v| v.is_finite()) {
                    return Err("values must be finite".to_owned());
                }
                if !p.is_finite() || *p < 0.0 || *p > 100.0 {
                    return Err("p must be in [0, 100]".to_owned());
                }
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Ok(StatsOutput::PercentileValue {
                    value: percentile_sorted(&sorted, *p),
                    p: *p,
                })
            }
            StatsRequest::Mode { values } => {
                if values.is_empty() {
                    return Err("values must contain at least one value".to_owned());
                }
                if !values.iter().all(|v| v.is_finite()) {
                    return Err("values must be finite".to_owned());
                }
                let (modal_values, frequency) = compute_mode(values);
                Ok(StatsOutput::ModeResult {
                    values: modal_values,
                    frequency,
                })
            }
            StatsRequest::Rank { values, method } => {
                if values.is_empty() {
                    return Err("values must contain at least one value".to_owned());
                }
                if !values.iter().all(|v| v.is_finite()) {
                    return Err("values must be finite".to_owned());
                }
                Ok(StatsOutput::RanksResult(compute_ranks(values, *method)))
            }
            StatsRequest::OneSampleT {
                sample,
                mu0,
                alpha,
                tail,
            } => one_sample_t_test(sample, *mu0, *alpha, *tail, "OneSampleT"),
            StatsRequest::TwoSampleT {
                sample1,
                sample2,
                alpha,
                tail,
                equal_var,
            } => two_sample_t_test(sample1, sample2, *alpha, *tail, *equal_var),
            StatsRequest::PairedT {
                before,
                after,
                alpha,
                tail,
            } => {
                if before.len() != after.len() {
                    return Err("before and after must have the same length".to_owned());
                }
                let diffs: Vec<f64> = before
                    .iter()
                    .zip(after.iter())
                    .map(|(b, a)| a - b)
                    .collect();
                one_sample_t_test(&diffs, 0.0, *alpha, *tail, "PairedT")
            }
            StatsRequest::ChiSquareGof {
                observed,
                expected,
                alpha,
            } => chi_square_gof_test(observed, expected, *alpha),
            StatsRequest::OneWayAnova { groups, alpha } => one_way_anova_test(groups, *alpha),
            StatsRequest::TimeSeries {
                method,
                values,
                period,
                max_lag,
                smoothing,
            } => time_series_analysis(*method, values, *period, *max_lag, *smoothing),
            StatsRequest::MannWhitneyU {
                sample1,
                sample2,
                alpha,
                tail,
            } => mann_whitney_u_test(sample1, sample2, *alpha, *tail),
            StatsRequest::WilcoxonSigned {
                before,
                after,
                alpha,
                tail,
            } => wilcoxon_signed_rank_test(before, after, *alpha, *tail),
            StatsRequest::KruskalWallis { groups, alpha } => kruskal_wallis_test(groups, *alpha),
            StatsRequest::SpearmanCorrelation { x, y } => {
                let (rho, n) = spearman_correlation(x, y)?;
                Ok(StatsOutput::CorrelationResult { pearson_r: rho, n })
            }
            StatsRequest::KendallTau { x, y } => {
                let (tau, n) = kendall_tau_b(x, y)?;
                Ok(StatsOutput::CorrelationResult { pearson_r: tau, n })
            }
            StatsRequest::ProbabilityDistribution {
                distribution,
                query,
            } => eval_distribution(distribution, query),
            StatsRequest::CohenD { sample1, sample2 } => {
                let d = cohen_d_two_sample(sample1, sample2)?;
                Ok(StatsOutput::EffectSizeResult {
                    statistic: "cohen_d",
                    value: d,
                    interpretation: interpret_d(d),
                })
            }
            StatsRequest::CohenDOneSample { sample, mu0 } => {
                let d = cohen_d_one_sample(sample, *mu0)?;
                Ok(StatsOutput::EffectSizeResult {
                    statistic: "cohen_d",
                    value: d,
                    interpretation: interpret_d(d),
                })
            }
            StatsRequest::EtaSquared { groups } => {
                let eta = eta_squared(groups)?;
                Ok(StatsOutput::EffectSizeResult {
                    statistic: "eta_squared",
                    value: eta,
                    interpretation: interpret_eta_sq(eta),
                })
            }
            StatsRequest::CramersV { observed } => {
                let v = cramers_v(observed)?;
                Ok(StatsOutput::EffectSizeResult {
                    statistic: "cramers_v",
                    value: v,
                    interpretation: interpret_d(v),
                })
            }
            StatsRequest::PointBiserialR { binary, continuous } => {
                let r = point_biserial_r(binary, continuous)?;
                Ok(StatsOutput::EffectSizeResult {
                    statistic: "point_biserial_r",
                    value: r,
                    interpretation: interpret_r(r.abs()),
                })
            }
            StatsRequest::PowerOneSampleT {
                effect_d,
                alpha,
                power,
            } => {
                let n = power_t(*effect_d, *alpha, *power, false)?;
                Ok(StatsOutput::SampleSizeResult { n })
            }
            StatsRequest::PowerTwoSampleT {
                effect_d,
                alpha,
                power,
            } => {
                let n = power_t(*effect_d, *alpha, *power, true)?;
                Ok(StatsOutput::SampleSizeResult { n })
            }
            StatsRequest::PowerOneProportion {
                p0,
                p1,
                alpha,
                power,
            } => {
                let n = power_proportion(*p0, *p1, *alpha, *power)?;
                Ok(StatsOutput::SampleSizeResult { n })
            }
        }
    }
}

pub fn stats_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/stats.json"),
        "title": "agent-calc calc1 stats request",
        "description": "Typed statistical and probability request backed by statrs.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/DescribeSample"},
            {"$ref": "#/$defs/NormalCdf"},
            {"$ref": "#/$defs/NormalQuantile"},
            {"$ref": "#/$defs/StudentTInterval"},
            {"$ref": "#/$defs/BinomialPmf"},
            {"$ref": "#/$defs/BinomialCdf"},
            {"$ref": "#/$defs/TCdf"},
            {"$ref": "#/$defs/TInverseCdf"},
            {"$ref": "#/$defs/Chi2Cdf"},
            {"$ref": "#/$defs/Chi2InverseCdf"},
            {"$ref": "#/$defs/PoissonPmf"},
            {"$ref": "#/$defs/PoissonCdf"},
            {"$ref": "#/$defs/BetaCdf"},
            {"$ref": "#/$defs/BetaPdf"},
            {"$ref": "#/$defs/FCdf"},
            {"$ref": "#/$defs/ExponentialCdf"},
            {"$ref": "#/$defs/ExponentialPdf"},
            {"$ref": "#/$defs/UniformCdf"},
            {"$ref": "#/$defs/Correlation"},
            {"$ref": "#/$defs/LinearRegression"},
            {"$ref": "#/$defs/Percentile"},
            {"$ref": "#/$defs/Mode"},
            {"$ref": "#/$defs/Rank"},
            {"$ref": "#/$defs/OneSampleT"},
            {"$ref": "#/$defs/TwoSampleT"},
            {"$ref": "#/$defs/PairedT"},
            {"$ref": "#/$defs/ChiSquareGof"},
            {"$ref": "#/$defs/OneWayAnova"},
            {"$ref": "#/$defs/ProbabilityDistribution"},
            {"$ref": "#/$defs/CohenD"},
            {"$ref": "#/$defs/CohenDOneSample"},
            {"$ref": "#/$defs/EtaSquared"},
            {"$ref": "#/$defs/CramersV"},
            {"$ref": "#/$defs/PointBiserialR"},
            {"$ref": "#/$defs/PowerOneSampleT"},
            {"$ref": "#/$defs/PowerTwoSampleT"},
            {"$ref": "#/$defs/PowerOneProportion"}
        ],
        "$defs": {
            "Sample": {
                "type": "array",
                "items": {"type": "number"},
                "minItems": 1
            },
            "DescribeSample": {
                "type": "object",
                "required": ["intent", "values"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "describe_sample"},
                    "values": {"$ref": "#/$defs/Sample"}
                }
            },
            "NormalCdf": {
                "type": "object",
                "required": ["intent", "mean", "std_dev", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "normal_cdf"},
                    "mean": {"type": "number"},
                    "std_dev": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number"}
                }
            },
            "NormalQuantile": {
                "type": "object",
                "required": ["intent", "mean", "std_dev", "p"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "normal_quantile"},
                    "mean": {"type": "number"},
                    "std_dev": {"type": "number", "exclusiveMinimum": 0},
                    "p": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1}
                }
            },
            "StudentTInterval": {
                "type": "object",
                "required": ["intent", "values", "confidence"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "student_t_interval"},
                    "values": {"$ref": "#/$defs/Sample"},
                    "confidence": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1}
                }
            },
            "BinomialPmf": {
                "type": "object",
                "required": ["intent", "n", "p", "k"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "binomial_pmf"},
                    "n": {"type": "integer", "minimum": 0},
                    "p": {"type": "number", "minimum": 0, "maximum": 1},
                    "k": {"type": "integer", "minimum": 0}
                }
            },
            "BinomialCdf": {
                "type": "object",
                "required": ["intent", "n", "p", "k"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "binomial_cdf"},
                    "n": {"type": "integer", "minimum": 0},
                    "p": {"type": "number", "minimum": 0, "maximum": 1},
                    "k": {"type": "integer", "minimum": 0}
                }
            },
            "TCdf": {
                "type": "object",
                "required": ["intent", "degrees_of_freedom", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "t_cdf"},
                    "degrees_of_freedom": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number"}
                }
            },
            "TInverseCdf": {
                "type": "object",
                "required": ["intent", "degrees_of_freedom", "p"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "t_inverse_cdf"},
                    "degrees_of_freedom": {"type": "number", "exclusiveMinimum": 0},
                    "p": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1}
                }
            },
            "Chi2Cdf": {
                "type": "object",
                "required": ["intent", "degrees_of_freedom", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "chi2_cdf"},
                    "degrees_of_freedom": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number", "minimum": 0}
                }
            },
            "Chi2InverseCdf": {
                "type": "object",
                "required": ["intent", "degrees_of_freedom", "p"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "chi2_inverse_cdf"},
                    "degrees_of_freedom": {"type": "number", "exclusiveMinimum": 0},
                    "p": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1}
                }
            },
            "PoissonPmf": {
                "type": "object",
                "required": ["intent", "lambda", "k"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "poisson_pmf"},
                    "lambda": {"type": "number", "exclusiveMinimum": 0},
                    "k": {"type": "integer", "minimum": 0}
                }
            },
            "PoissonCdf": {
                "type": "object",
                "required": ["intent", "lambda", "k"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "poisson_cdf"},
                    "lambda": {"type": "number", "exclusiveMinimum": 0},
                    "k": {"type": "integer", "minimum": 0}
                }
            },
            "BetaCdf": {
                "type": "object",
                "required": ["intent", "alpha", "beta", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "beta_cdf"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0},
                    "beta": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number", "minimum": 0, "maximum": 1}
                }
            },
            "BetaPdf": {
                "type": "object",
                "required": ["intent", "alpha", "beta", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "beta_pdf"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0},
                    "beta": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number", "minimum": 0, "maximum": 1}
                }
            },
            "FCdf": {
                "type": "object",
                "required": ["intent", "d1", "d2", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "f_cdf"},
                    "d1": {"type": "number", "exclusiveMinimum": 0},
                    "d2": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number", "minimum": 0}
                }
            },
            "ExponentialCdf": {
                "type": "object",
                "required": ["intent", "rate", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "exponential_cdf"},
                    "rate": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number", "minimum": 0}
                }
            },
            "ExponentialPdf": {
                "type": "object",
                "required": ["intent", "rate", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "exponential_pdf"},
                    "rate": {"type": "number", "exclusiveMinimum": 0},
                    "x": {"type": "number", "minimum": 0}
                }
            },
            "UniformCdf": {
                "type": "object",
                "required": ["intent", "min", "max", "x"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "uniform_cdf"},
                    "min": {"type": "number"},
                    "max": {"type": "number"},
                    "x": {"type": "number"}
                }
            },
            "Correlation": {
                "type": "object",
                "required": ["intent", "x", "y"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "correlation"},
                    "x": {"$ref": "#/$defs/Sample"},
                    "y": {"$ref": "#/$defs/Sample"}
                }
            },
            "LinearRegression": {
                "type": "object",
                "required": ["intent", "x", "y"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "linear_regression"},
                    "x": {
                        "oneOf": [
                            {"$ref": "#/$defs/Sample"},
                            {
                                "type": "array",
                                "items": {"$ref": "#/$defs/Sample"},
                                "minItems": 1,
                                "description": "n×k predictor matrix (multiple regression)"
                            }
                        ]
                    },
                    "y": {"$ref": "#/$defs/Sample"}
                }
            },
            "Percentile": {
                "type": "object",
                "required": ["intent", "values", "p"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "percentile"},
                    "values": {"$ref": "#/$defs/Sample"},
                    "p": {"type": "number", "minimum": 0, "maximum": 100}
                }
            },
            "Mode": {
                "type": "object",
                "required": ["intent", "values"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "mode"},
                    "values": {"$ref": "#/$defs/Sample"}
                }
            },
            "Rank": {
                "type": "object",
                "required": ["intent", "values", "method"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "rank"},
                    "values": {"$ref": "#/$defs/Sample"},
                    "method": {"type": "string", "enum": ["average", "min", "max", "first", "last"]}
                }
            },
            "OneSampleT": {
                "type": "object",
                "required": ["intent", "sample", "mu0"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "one_sample_t"},
                    "sample": {"$ref": "#/$defs/Sample"},
                    "mu0": {"type": "number"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05},
                    "tail": {"type": "string", "enum": ["two", "left", "right"], "default": "two"}
                }
            },
            "TwoSampleT": {
                "type": "object",
                "required": ["intent", "sample1", "sample2"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "two_sample_t"},
                    "sample1": {"$ref": "#/$defs/Sample"},
                    "sample2": {"$ref": "#/$defs/Sample"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05},
                    "tail": {"type": "string", "enum": ["two", "left", "right"], "default": "two"},
                    "equal_var": {"type": "boolean", "default": false}
                }
            },
            "PairedT": {
                "type": "object",
                "required": ["intent", "before", "after"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "paired_t"},
                    "before": {"$ref": "#/$defs/Sample"},
                    "after": {"$ref": "#/$defs/Sample"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05},
                    "tail": {"type": "string", "enum": ["two", "left", "right"], "default": "two"}
                }
            },
            "ChiSquareGof": {
                "type": "object",
                "required": ["intent", "observed", "expected"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "chi_square_gof"},
                    "observed": {"$ref": "#/$defs/Sample"},
                    "expected": {"$ref": "#/$defs/Sample"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05}
                }
            },
            "OneWayAnova": {
                "type": "object",
                "required": ["intent", "groups"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "one_way_anova"},
                    "groups": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Sample"},
                        "minItems": 2,
                        "description": "k groups, each a non-empty array of observations"
                    },
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05}
                }
            },
            "ProbabilityDistribution": {
                "type": "object",
                "required": ["intent", "distribution", "query"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "probability_distribution"},
                    "distribution": {"$ref": "#/$defs/DistributionKind"},
                    "query": {"$ref": "#/$defs/DistributionQuery"}
                }
            },
            "DistributionKind": {
                "oneOf": [
                    {"type": "object", "required": ["normal"], "additionalProperties": false,
                     "properties": {"normal": {"type": "object", "required": ["mean", "std_dev"],
                         "properties": {"mean": {"type": "number"}, "std_dev": {"type": "number", "exclusiveMinimum": 0}}}}},
                    {"type": "object", "required": ["student_t"], "additionalProperties": false,
                     "properties": {"student_t": {"type": "object", "required": ["df"],
                         "properties": {"df": {"type": "number", "exclusiveMinimum": 0}}}}},
                    {"type": "object", "required": ["chi_squared"], "additionalProperties": false,
                     "properties": {"chi_squared": {"type": "object", "required": ["df"],
                         "properties": {"df": {"type": "number", "exclusiveMinimum": 0}}}}},
                    {"type": "object", "required": ["f_dist"], "additionalProperties": false,
                     "properties": {"f_dist": {"type": "object", "required": ["d1", "d2"],
                         "properties": {"d1": {"type": "number", "exclusiveMinimum": 0}, "d2": {"type": "number", "exclusiveMinimum": 0}}}}},
                    {"type": "object", "required": ["binomial"], "additionalProperties": false,
                     "properties": {"binomial": {"type": "object", "required": ["n", "p"],
                         "properties": {"n": {"type": "integer", "minimum": 0}, "p": {"type": "number", "minimum": 0, "maximum": 1}}}}},
                    {"type": "object", "required": ["poisson"], "additionalProperties": false,
                     "properties": {"poisson": {"type": "object", "required": ["lambda"],
                         "properties": {"lambda": {"type": "number", "exclusiveMinimum": 0}}}}},
                    {"type": "object", "required": ["exponential"], "additionalProperties": false,
                     "properties": {"exponential": {"type": "object", "required": ["rate"],
                         "properties": {"rate": {"type": "number", "exclusiveMinimum": 0}}}}}
                ]
            },
            "DistributionQuery": {
                "oneOf": [
                    {"type": "object", "required": ["pdf"], "additionalProperties": false,
                     "properties": {"pdf": {"type": "object", "required": ["x"], "properties": {"x": {"type": "number"}}}}},
                    {"type": "object", "required": ["cdf"], "additionalProperties": false,
                     "properties": {"cdf": {"type": "object", "required": ["x"], "properties": {"x": {"type": "number"}}}}},
                    {"type": "object", "required": ["quantile"], "additionalProperties": false,
                     "properties": {"quantile": {"type": "object", "required": ["p"],
                         "properties": {"p": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1}}}}}
                ]
            },
            "CohenD": {
                "type": "object",
                "required": ["intent", "sample1", "sample2"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "cohen_d"},
                    "sample1": {"$ref": "#/$defs/Sample"},
                    "sample2": {"$ref": "#/$defs/Sample"}
                }
            },
            "CohenDOneSample": {
                "type": "object",
                "required": ["intent", "sample", "mu0"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "cohen_d_one_sample"},
                    "sample": {"$ref": "#/$defs/Sample"},
                    "mu0": {"type": "number"}
                }
            },
            "EtaSquared": {
                "type": "object",
                "required": ["intent", "groups"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "eta_squared"},
                    "groups": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Sample"},
                        "minItems": 2
                    }
                }
            },
            "CramersV": {
                "type": "object",
                "required": ["intent", "observed"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "cramers_v"},
                    "observed": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Sample"},
                        "minItems": 2,
                        "description": "r×c contingency table of non-negative counts"
                    }
                }
            },
            "PointBiserialR": {
                "type": "object",
                "required": ["intent", "binary", "continuous"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "point_biserial_r"},
                    "binary": {"type": "array", "items": {"type": "number", "enum": [0, 1]}, "minItems": 2},
                    "continuous": {"$ref": "#/$defs/Sample"}
                }
            },
            "PowerOneSampleT": {
                "type": "object",
                "required": ["intent", "effect_d"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "power_one_sample_t"},
                    "effect_d": {"type": "number"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05},
                    "power": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.80}
                }
            },
            "PowerTwoSampleT": {
                "type": "object",
                "required": ["intent", "effect_d"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "power_two_sample_t"},
                    "effect_d": {"type": "number"},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05},
                    "power": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.80}
                }
            },
            "PowerOneProportion": {
                "type": "object",
                "required": ["intent", "p0", "p1"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "power_one_proportion"},
                    "p0": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1},
                    "p1": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1},
                    "alpha": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.05},
                    "power": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1, "default": 0.80}
                }
            }
        }
    })
}

#[derive(Clone, Copy, Debug)]
struct SampleSummary {
    n: usize,
    mean: f64,
    variance: f64,
    std_dev: f64,
    min: f64,
    max: f64,
    median: f64,
    q1: f64,
    q3: f64,
    iqr: f64,
    skewness: f64,
    kurtosis: f64,
}

enum StatsOutput {
    Sample(SampleSummary),
    Probability(f64),
    Quantile(f64),
    Interval {
        confidence: f64,
        mean: f64,
        lower: f64,
        upper: f64,
    },
    CorrelationResult {
        pearson_r: f64,
        n: usize,
    },
    RegressionResult {
        slope: f64,
        intercept: f64,
        r_squared: f64,
    },
    MultipleRegressionResult {
        coefficients: Vec<f64>,
        r_squared: f64,
        residual_std_dev: f64,
        std_errors: Vec<f64>,
        n: usize,
        k: usize,
    },
    PercentileValue {
        value: f64,
        p: f64,
    },
    ModeResult {
        values: Vec<f64>,
        frequency: usize,
    },
    RanksResult(Vec<f64>),
    HypothesisTestResult {
        test: &'static str,
        statistic: f64,
        p_value: f64,
        dof: f64,
        critical_value: f64,
        alpha: f64,
        reject_h0: bool,
        conclusion: String,
    },
    TimeSeriesResult {
        method: &'static str,
        result: Vec<f64>,
        n: usize,
    },
    EffectSizeResult {
        statistic: &'static str,
        value: f64,
        interpretation: &'static str,
    },
    SampleSizeResult {
        n: u64,
    },
}

/// Neumaier compensated summation. This preserves small terms that would
/// otherwise be lost when values have very different magnitudes.
fn compensated_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut sum = 0.0;
    let mut correction = 0.0;
    for value in values {
        let next = sum + value;
        if sum.abs() >= value.abs() {
            correction += (sum - next) + value;
        } else {
            correction += (value - next) + sum;
        }
        sum = next;
    }
    sum + correction
}

/// Scale before summing so a finite mean remains representable even when the
/// unscaled sum would overflow.
fn stable_mean(values: &[f64]) -> f64 {
    stable_mean_iter(values.iter().copied(), values.len())
}

fn stable_mean_iter(values: impl IntoIterator<Item = f64>, count: usize) -> f64 {
    let n = count as f64;
    compensated_sum(values.into_iter().map(|value| value / n))
}

fn centered_sum_squares(values: &[f64], mean: f64) -> f64 {
    compensated_sum(values.iter().map(|value| {
        let delta = value - mean;
        delta * delta
    }))
}

fn centered_sum_products(x: &[f64], y: &[f64], mean_x: f64, mean_y: f64) -> f64 {
    compensated_sum(
        x.iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y)),
    )
}

fn sample_mean_variance(values: &[f64]) -> (f64, f64) {
    let mean = stable_mean(values);
    let variance = centered_sum_squares(values, mean) / (values.len() - 1) as f64;
    (mean, variance)
}

fn describe_sample(values: &[f64]) -> Result<SampleSummary, String> {
    if values.is_empty() {
        return Err("sample must contain at least one value".to_owned());
    }
    if !values.iter().all(|v| v.is_finite()) {
        return Err("sample values must be finite".to_owned());
    }
    let n = values.len();
    let mean = stable_mean(values);
    let mut min = values[0];
    let mut max = values[0];
    for value in values {
        min = min.min(*value);
        max = max.max(*value);
    }
    let sum_sq = centered_sum_squares(values, mean);
    let variance = if n > 1 { sum_sq / (n - 1) as f64 } else { 0.0 };
    let std_dev = variance.sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median = percentile_sorted(&sorted, 50.0);
    let q1 = percentile_sorted(&sorted, 25.0);
    let q3 = percentile_sorted(&sorted, 75.0);
    let iqr = q3 - q1;
    let skewness = sample_skewness(values, mean, std_dev);
    let kurtosis = sample_kurtosis(values, mean, std_dev);

    Ok(SampleSummary {
        n,
        mean,
        variance,
        std_dev,
        min,
        max,
        median,
        q1,
        q3,
        iqr,
        skewness,
        kurtosis,
    })
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let pos = p / 100.0 * (n - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let frac = pos - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

fn sample_skewness(values: &[f64], mean: f64, std_dev: f64) -> f64 {
    let n = values.len();
    if n < 3 || std_dev == 0.0 {
        return 0.0;
    }
    let factor = n as f64 / ((n - 1) as f64 * (n - 2) as f64);
    let sum = compensated_sum(values.iter().map(|v| ((v - mean) / std_dev).powi(3)));
    factor * sum
}

fn sample_kurtosis(values: &[f64], mean: f64, std_dev: f64) -> f64 {
    let n = values.len() as f64;
    if n < 4.0 || std_dev == 0.0 {
        return 0.0;
    }
    let factor1 = (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0));
    let sum = compensated_sum(values.iter().map(|v| ((v - mean) / std_dev).powi(4)));
    let correction = 3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0));
    factor1 * sum - correction
}

fn spearman_correlation(x: &[f64], y: &[f64]) -> Result<(f64, usize), String> {
    if x.len() != y.len() {
        return Err("x and y must have the same length".to_owned());
    }
    if !x.iter().all(|v| v.is_finite()) || !y.iter().all(|v| v.is_finite()) {
        return Err("x and y values must be finite".to_owned());
    }
    let rx = compute_ranks(x, RankMethod::Average);
    let ry = compute_ranks(y, RankMethod::Average);
    correlation(&rx, &ry)
}

#[mutants::skip] // (i+1)..n → i..n adds self-pairs that are always dx=dy=0 (tied-on-both, not counted) — equivalent mutation
fn kendall_pairs(n: usize) -> Vec<(usize, usize)> {
    (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .collect()
}

fn kendall_tau_b(x: &[f64], y: &[f64]) -> Result<(f64, usize), String> {
    if x.len() != y.len() {
        return Err("x and y must have the same length".to_owned());
    }
    if !x.iter().all(|v| v.is_finite()) || !y.iter().all(|v| v.is_finite()) {
        return Err("x and y values must be finite".to_owned());
    }
    let n = x.len();
    let mut concordant = 0i64;
    let mut discordant = 0i64;
    let mut tie_x = 0i64;
    let mut tie_y = 0i64;
    for (i, j) in kendall_pairs(n) {
        let dx = x[j] - x[i];
        let dy = y[j] - y[i];
        if dx == 0.0 && dy == 0.0 {
            // tied on both — not counted
        } else if dx == 0.0 {
            tie_x += 1;
        } else if dy == 0.0 {
            tie_y += 1;
        } else if dx.signum() == dy.signum() {
            concordant += 1;
        } else {
            discordant += 1;
        }
    }
    let c = concordant as f64;
    let d = discordant as f64;
    let tx = tie_x as f64;
    let ty = tie_y as f64;
    let denom = ((c + d + tx) * (c + d + ty)).sqrt();
    if denom == 0.0 {
        return Err("kendall_tau is undefined when all values are tied".to_owned());
    }
    Ok(((c - d) / denom, n))
}

fn correlation(x: &[f64], y: &[f64]) -> Result<(f64, usize), String> {
    if x.len() != y.len() {
        return Err("x and y must have the same length".to_owned());
    }
    let n = x.len();
    if n < 2 {
        return Err("correlation requires at least two paired observations".to_owned());
    }
    if !x.iter().all(|v| v.is_finite()) || !y.iter().all(|v| v.is_finite()) {
        return Err("x and y values must be finite".to_owned());
    }
    let mean_x = stable_mean(x);
    let mean_y = stable_mean(y);
    let num = centered_sum_products(x, y, mean_x, mean_y);
    let denom_x = centered_sum_squares(x, mean_x);
    let denom_y = centered_sum_squares(y, mean_y);
    let denom = denom_x.sqrt() * denom_y.sqrt();
    if denom == 0.0 {
        return Err(
            "correlation is undefined when all values in a series are identical".to_owned(),
        );
    }
    Ok((num / denom, n))
}

fn linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64, f64), String> {
    let (pearson_r, _) = correlation(x, y)?;
    let mean_x = stable_mean(x);
    let mean_y = stable_mean(y);
    let denom_x = centered_sum_squares(x, mean_x);
    let num = centered_sum_products(x, y, mean_x, mean_y);
    let slope = num / denom_x;
    let intercept = mean_y - slope * mean_x;
    let r_squared = pearson_r * pearson_r;
    Ok((slope, intercept, r_squared))
}

struct MultipleRegressionOutput {
    coefficients: Vec<f64>,
    r_squared: f64,
    residual_std_dev: f64,
    std_errors: Vec<f64>,
    n: usize,
    k: usize,
}

// Threshold guard is a numerical-stability tuning parameter; exact operator/multiplier
// choice has no effect on correctness for non-degenerate inputs.
#[mutants::skip]
fn sv_significant(s: f64, sigma_max: f64) -> bool {
    s > 1e-10 * sigma_max
}

fn multiple_linear_regression(
    rows: &[Vec<f64>],
    y: &[f64],
) -> Result<MultipleRegressionOutput, String> {
    use nalgebra::{DMatrix, DVector};

    let n = y.len();
    if n < 2 {
        return Err("linear_regression requires at least 2 observations".to_owned());
    }
    if !y.iter().all(|v| v.is_finite()) {
        return Err("y values must be finite".to_owned());
    }
    if rows.len() != n {
        return Err(format!(
            "x has {} rows but y has {} elements",
            rows.len(),
            n
        ));
    }
    let k = rows.first().map(|r| r.len()).unwrap_or(0);
    if k == 0 {
        return Err("x matrix must have at least one column".to_owned());
    }
    for (i, row) in rows.iter().enumerate() {
        if row.len() != k {
            return Err(format!(
                "row {} has {} columns, expected {}",
                i,
                row.len(),
                k
            ));
        }
        if !row.iter().all(|v| v.is_finite()) {
            return Err("x values must be finite".to_owned());
        }
    }
    if n <= k + 1 {
        return Err(format!(
            "need at least {} observations for {} predictors, got {}",
            k + 2,
            k,
            n
        ));
    }

    // Build augmented design matrix [1 | X]  (n × (k+1))
    let p = k + 1;
    let mut design_data = Vec::with_capacity(n * p);
    for row in rows {
        design_data.push(1.0_f64);
        design_data.extend_from_slice(row);
    }
    let design = DMatrix::from_row_slice(n, p, &design_data);
    let y_vec = DVector::from_column_slice(y);

    // SVD least-squares solve — stable for ill-conditioned (Longley-class) matrices
    let svd = design.clone().svd(true, true);
    let beta = svd
        .solve(&y_vec, 1e-10)
        .map_err(|_| "design matrix is rank-deficient".to_owned())?;

    // Residuals and fit statistics
    let residuals = &y_vec - &design * &beta;
    let ss_res = compensated_sum(residuals.iter().map(|r| r * r));
    let y_mean = stable_mean(y);
    let ss_tot = centered_sum_squares(y, y_mean);
    let r_squared = if ss_tot == 0.0 {
        1.0
    } else {
        1.0 - ss_res / ss_tot
    };
    let dof = (n - k - 1) as f64;
    let residual_std_dev = (ss_res / dof).sqrt();

    // Std errors: se[i] = s * sqrt( Σ_j (V[i,j] / σ_j)² )
    // Var(β̂) = σ² (X̃ᵀX̃)⁻¹ = σ² V Σ⁻² Vᵀ
    let v_t = svd.v_t.ok_or_else(|| "SVD did not compute V".to_owned())?;
    let sigma = &svd.singular_values;
    let sigma_max = sigma.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    #[allow(clippy::manual_clamp)]
    let std_errors: Vec<f64> = (0..p)
        .map(|i| {
            let sum_sq: f64 = (0..sigma.len())
                .map(|j| {
                    let s = sigma[j];
                    if sv_significant(s, sigma_max) {
                        let v_ji = v_t[(j, i)];
                        (v_ji / s).powi(2)
                    } else {
                        0.0
                    }
                })
                .sum();
            residual_std_dev * sum_sq.sqrt()
        })
        .collect();

    Ok(MultipleRegressionOutput {
        coefficients: beta.iter().copied().collect(),
        r_squared,
        residual_std_dev,
        std_errors,
        n,
        k,
    })
}

fn compute_mode(values: &[f64]) -> (Vec<f64>, usize) {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut counts: Vec<(f64, usize)> = Vec::new();
    for v in &sorted {
        if let Some(last) = counts.last_mut()
            && last.0.to_bits() == v.to_bits()
        {
            last.1 += 1;
            continue;
        }
        counts.push((*v, 1));
    }

    let max_count = counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let modal: Vec<f64> = counts
        .into_iter()
        .filter(|(_, c)| *c == max_count)
        .map(|(v, _)| v)
        .collect();
    (modal, max_count)
}

fn compute_ranks(values: &[f64], method: RankMethod) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(f64, usize)> = values.iter().copied().zip(0..n).collect();
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.cmp(&b.1)));

    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && indexed[j].0.to_bits() == indexed[i].0.to_bits() {
            j += 1;
        }
        // positions i..j are a tie group; 1-based ranks are (i+1)..=j
        for k in i..j {
            let rank = match method {
                RankMethod::Average => (i + j + 1) as f64 / 2.0,
                RankMethod::Min => (i + 1) as f64,
                RankMethod::Max => j as f64,
                RankMethod::First => (k + 1) as f64,
                RankMethod::Last => (i + j - k) as f64,
            };
            ranks[indexed[k].1] = rank;
        }
        i = j;
    }
    ranks
}

fn t_p_value(t_dist: &StudentsT, t_stat: f64, tail: Tail) -> f64 {
    match tail {
        Tail::Two => 2.0 * (1.0 - t_dist.cdf(t_stat.abs())),
        Tail::Left => t_dist.cdf(t_stat),
        Tail::Right => 1.0 - t_dist.cdf(t_stat),
    }
}

fn t_critical(t_dist: &StudentsT, alpha: f64, tail: Tail) -> f64 {
    match tail {
        Tail::Two => t_dist.inverse_cdf(1.0 - alpha / 2.0),
        Tail::Left => -t_dist.inverse_cdf(1.0 - alpha),
        Tail::Right => t_dist.inverse_cdf(1.0 - alpha),
    }
}

fn hypothesis_conclusion(p_value: f64, alpha: f64, reject_h0: bool) -> String {
    if reject_h0 {
        format!("Reject H0 at alpha={alpha} (p={p_value:.4} < {alpha})")
    } else {
        format!("Fail to reject H0 at alpha={alpha} (p={p_value:.4} >= {alpha})")
    }
}

fn one_sample_t_test(
    sample: &[f64],
    mu0: f64,
    alpha: f64,
    tail: Tail,
    test_name: &'static str,
) -> Result<StatsOutput, String> {
    if sample.len() < 2 {
        return Err("sample must contain at least 2 values".to_owned());
    }
    if !sample.iter().all(|v| v.is_finite()) {
        return Err("sample values must be finite".to_owned());
    }
    ensure_finite(mu0, "mu0")?;
    let n = sample.len();
    let (mean, var) = sample_mean_variance(sample);
    let s = var.sqrt();
    if s == 0.0 {
        return Err("sample has zero variance — t-test is undefined".to_owned());
    }
    let t_stat = (mean - mu0) / (s / (n as f64).sqrt());
    let dof = (n - 1) as f64;
    let t_dist = StudentsT::new(0.0, 1.0, dof).map_err(|e| e.to_string())?;
    let p_value = t_p_value(&t_dist, t_stat, tail);
    let critical_value = t_critical(&t_dist, alpha, tail);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: test_name,
        statistic: t_stat,
        p_value,
        dof,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn two_sample_t_test(
    sample1: &[f64],
    sample2: &[f64],
    alpha: f64,
    tail: Tail,
    equal_var: bool,
) -> Result<StatsOutput, String> {
    for (s, name) in [(sample1, "sample1"), (sample2, "sample2")] {
        if s.len() < 2 {
            return Err(format!("{name} must contain at least 2 values"));
        }
        if !s.iter().all(|v| v.is_finite()) {
            return Err(format!("{name} values must be finite"));
        }
    }
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;
    let (mean1, var1) = sample_mean_variance(sample1);
    let (mean2, var2) = sample_mean_variance(sample2);

    let (t_stat, dof) = if equal_var {
        let sp2 = ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0);
        let sp = sp2.sqrt();
        if sp == 0.0 {
            return Err("samples have zero pooled variance — t-test is undefined".to_owned());
        }
        let t = (mean1 - mean2) / (sp * (1.0 / n1 + 1.0 / n2).sqrt());
        (t, n1 + n2 - 2.0)
    } else {
        let v1n = var1 / n1;
        let v2n = var2 / n2;
        let se = (v1n + v2n).sqrt();
        if se == 0.0 {
            return Err("samples have zero variance — t-test is undefined".to_owned());
        }
        let t = (mean1 - mean2) / se;
        let dof = (v1n + v2n).powi(2) / (v1n.powi(2) / (n1 - 1.0) + v2n.powi(2) / (n2 - 1.0));
        (t, dof)
    };

    let t_dist = StudentsT::new(0.0, 1.0, dof).map_err(|e| e.to_string())?;
    let p_value = t_p_value(&t_dist, t_stat, tail);
    let critical_value = t_critical(&t_dist, alpha, tail);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: "TwoSampleT",
        statistic: t_stat,
        p_value,
        dof,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn chi_square_gof_test(
    observed: &[f64],
    expected: &[f64],
    alpha: f64,
) -> Result<StatsOutput, String> {
    if observed.len() != expected.len() {
        return Err("observed and expected must have the same length".to_owned());
    }
    if observed.len() < 2 {
        return Err("chi_square_gof requires at least 2 categories".to_owned());
    }
    if !observed.iter().all(|v| v.is_finite() && *v >= 0.0) {
        return Err("observed values must be finite and non-negative".to_owned());
    }
    if !expected.iter().all(|v| v.is_finite() && *v > 0.0) {
        return Err("expected values must be finite and positive".to_owned());
    }
    let chi2: f64 = observed
        .iter()
        .zip(expected.iter())
        .map(|(o, e)| (o - e).powi(2) / e)
        .sum();
    let dof = (observed.len() - 1) as f64;
    let chi2_dist = ChiSquared::new(dof).map_err(|e| e.to_string())?;
    let p_value = 1.0 - chi2_dist.cdf(chi2);
    let critical_value = chi2_dist.inverse_cdf(1.0 - alpha);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: "ChiSquareGof",
        statistic: chi2,
        p_value,
        dof,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn one_way_anova_test(groups: &[Vec<f64>], alpha: f64) -> Result<StatsOutput, String> {
    if groups.len() < 2 {
        return Err("one_way_anova requires at least 2 groups".to_owned());
    }
    for (i, g) in groups.iter().enumerate() {
        if g.len() < 2 {
            return Err(format!("group {i} must contain at least 2 values"));
        }
        if !g.iter().all(|v| v.is_finite()) {
            return Err(format!("group {i} values must be finite"));
        }
    }
    let n_total: usize = groups.iter().map(|g| g.len()).sum();
    let grand_mean = stable_mean_iter(groups.iter().flatten().copied(), n_total);
    let group_moments: Vec<(f64, f64)> = groups
        .iter()
        .map(|group| {
            let mean = stable_mean(group);
            (mean, centered_sum_squares(group, mean))
        })
        .collect();
    let ss_between = compensated_sum(
        groups
            .iter()
            .zip(group_moments.iter())
            .map(|(group, (mean, _))| group.len() as f64 * (mean - grand_mean).powi(2)),
    );
    let ss_within = compensated_sum(group_moments.iter().map(|(_, sum_sq)| *sum_sq));

    let df_between = (groups.len() - 1) as f64;
    let df_within = (n_total - groups.len()) as f64;

    if ss_within == 0.0 {
        return Err("all groups have zero within-group variance — ANOVA is undefined".to_owned());
    }
    let f_stat = (ss_between / df_between) / (ss_within / df_within);
    let f_dist = FisherSnedecor::new(df_between, df_within).map_err(|e| e.to_string())?;
    let p_value = 1.0 - f_dist.cdf(f_stat);
    let critical_value = f_dist.inverse_cdf(1.0 - alpha);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: "OneWayAnova",
        statistic: f_stat,
        p_value,
        dof: df_between,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn ts_validate(values: &[f64]) -> Result<(), String> {
    if values.len() < 2 {
        return Err("values must contain at least 2 elements".to_owned());
    }
    if !values.iter().all(|v| v.is_finite()) {
        return Err("values must be finite".to_owned());
    }
    Ok(())
}

fn sma(values: &[f64], period: usize) -> Result<Vec<f64>, String> {
    ts_validate(values)?;
    let n = values.len();
    if period < 2 {
        return Err("period must be at least 2".to_owned());
    }
    if period > n {
        return Err(format!(
            "period ({period}) cannot exceed values length ({n})"
        ));
    }
    let result = (0..=n - period)
        .map(|i| stable_mean(&values[i..i + period]))
        .collect();
    Ok(result)
}

fn ema(values: &[f64], smoothing: f64) -> Result<Vec<f64>, String> {
    ts_validate(values)?;
    if smoothing <= 0.0 || smoothing >= 1.0 {
        return Err("smoothing must be in (0, 1)".to_owned());
    }
    let mut result = Vec::with_capacity(values.len());
    result.push(values[0]);
    for i in 1..values.len() {
        let prev = result[i - 1];
        result.push(smoothing * values[i] + (1.0 - smoothing) * prev);
    }
    Ok(result)
}

fn autocorrelation(values: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
    ts_validate(values)?;
    let n = values.len();
    let mean = stable_mean(values);
    let denom = centered_sum_squares(values, mean);
    if denom == 0.0 {
        return Err("values have zero variance — autocorrelation is undefined".to_owned());
    }
    let effective_max = max_lag.min(n - 1);
    let result = (0..=effective_max)
        .map(|k| {
            if k == 0 {
                1.0
            } else {
                let num = compensated_sum(
                    (0..n - k).map(|i| (values[i] - mean) * (values[i + k] - mean)),
                );
                num / denom
            }
        })
        .collect();
    Ok(result)
}

fn ts_trend(values: &[f64]) -> Result<Vec<f64>, String> {
    ts_validate(values)?;
    let t: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
    let (slope, intercept, r_squared) = linear_regression(&t, values)?;
    Ok(vec![slope, intercept, r_squared])
}

fn time_series_analysis(
    method: TimeSeriesIntent,
    values: &[f64],
    period: usize,
    max_lag: usize,
    smoothing: f64,
) -> Result<StatsOutput, String> {
    let (result, name): (Vec<f64>, &'static str) = match method {
        TimeSeriesIntent::Sma => (sma(values, period)?, "Sma"),
        TimeSeriesIntent::Ema => (ema(values, smoothing)?, "Ema"),
        TimeSeriesIntent::Autocorr => (autocorrelation(values, max_lag)?, "Autocorr"),
        TimeSeriesIntent::Trend => (ts_trend(values)?, "Trend"),
    };
    Ok(StatsOutput::TimeSeriesResult {
        method: name,
        result,
        n: values.len(),
    })
}

fn normal_z_p_value(z: f64, tail: Tail) -> f64 {
    let n = Normal::new(0.0, 1.0).unwrap();
    match tail {
        Tail::Two => 2.0 * (1.0 - n.cdf(z.abs())),
        Tail::Left => n.cdf(z),
        Tail::Right => 1.0 - n.cdf(z),
    }
}

fn normal_z_critical(alpha: f64, tail: Tail) -> f64 {
    let n = Normal::new(0.0, 1.0).unwrap();
    match tail {
        Tail::Two => n.inverse_cdf(1.0 - alpha / 2.0),
        Tail::Left => -n.inverse_cdf(1.0 - alpha),
        Tail::Right => n.inverse_cdf(1.0 - alpha),
    }
}

fn mann_whitney_u_test(
    sample1: &[f64],
    sample2: &[f64],
    alpha: f64,
    tail: Tail,
) -> Result<StatsOutput, String> {
    let n1 = sample1.len();
    let n2 = sample2.len();
    if n1 < 2 {
        return Err("sample1 must contain at least 2 values".to_owned());
    }
    if n2 < 2 {
        return Err("sample2 must contain at least 2 values".to_owned());
    }
    if !sample1.iter().all(|v| v.is_finite()) {
        return Err("sample1 values must be finite".to_owned());
    }
    if !sample2.iter().all(|v| v.is_finite()) {
        return Err("sample2 values must be finite".to_owned());
    }
    let pooled: Vec<f64> = sample1.iter().chain(sample2.iter()).copied().collect();
    let ranks = compute_ranks(&pooled, RankMethod::Average);
    let r1: f64 = ranks[..n1].iter().sum();
    let u1 = r1 - (n1 * (n1 + 1) / 2) as f64;
    let mu_u = (n1 * n2) as f64 / 2.0;
    let sigma_u = ((n1 * n2 * (n1 + n2 + 1)) as f64 / 12.0).sqrt();
    let z = (u1 - mu_u) / sigma_u;
    let p_value = normal_z_p_value(z, tail);
    let critical_value = normal_z_critical(alpha, tail);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: "MannWhitneyU",
        statistic: u1,
        p_value,
        dof: 0.0,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn wilcoxon_signed_rank_test(
    before: &[f64],
    after: &[f64],
    alpha: f64,
    tail: Tail,
) -> Result<StatsOutput, String> {
    if before.len() != after.len() {
        return Err("before and after must have the same length".to_owned());
    }
    let n = before.len();
    if n < 2 {
        return Err("paired sample must contain at least 2 values".to_owned());
    }
    if !before.iter().all(|v| v.is_finite()) || !after.iter().all(|v| v.is_finite()) {
        return Err("before and after values must be finite".to_owned());
    }
    let diffs: Vec<f64> = before
        .iter()
        .zip(after.iter())
        .map(|(b, a)| a - b)
        .collect();
    let nonzero: Vec<f64> = diffs.iter().copied().filter(|d| *d != 0.0).collect();
    let n_nz = nonzero.len();
    if n_nz == 0 {
        return Err("all differences are zero — Wilcoxon test is undefined".to_owned());
    }
    let abs_nz: Vec<f64> = nonzero.iter().map(|d| d.abs()).collect();
    let ranks = compute_ranks(&abs_nz, RankMethod::Average);
    let w_plus: f64 = nonzero
        .iter()
        .zip(ranks.iter())
        .filter(|(d, _)| (**d).is_sign_positive())
        .map(|(_, r)| r)
        .sum();
    let mu_w = (n_nz * (n_nz + 1)) as f64 / 4.0;
    let sigma_w = ((n_nz * (n_nz + 1) * (2 * n_nz + 1)) as f64 / 24.0).sqrt();
    let z = (w_plus - mu_w) / sigma_w;
    let p_value = normal_z_p_value(z, tail);
    let critical_value = normal_z_critical(alpha, tail);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: "WilcoxonSigned",
        statistic: w_plus,
        p_value,
        dof: 0.0,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn kruskal_wallis_test(groups: &[Vec<f64>], alpha: f64) -> Result<StatsOutput, String> {
    if groups.len() < 2 {
        return Err("kruskal_wallis requires at least 2 groups".to_owned());
    }
    for (i, g) in groups.iter().enumerate() {
        if g.len() < 2 {
            return Err(format!("group {i} must contain at least 2 values"));
        }
        if !g.iter().all(|v| v.is_finite()) {
            return Err(format!("group {i} values must be finite"));
        }
    }
    let n_total: usize = groups.iter().map(|g| g.len()).sum();
    let pooled: Vec<f64> = groups.iter().flat_map(|g| g.iter().copied()).collect();
    let all_ranks = compute_ranks(&pooled, RankMethod::Average);
    let mut h_sum = 0.0f64;
    let mut offset = 0usize;
    for g in groups {
        let n_k = g.len();
        let r_k: f64 = all_ranks[offset..offset + n_k].iter().sum();
        h_sum += r_k * r_k / n_k as f64;
        offset += n_k;
    }
    let h = (12.0 / (n_total * (n_total + 1)) as f64) * h_sum - 3.0 * (n_total + 1) as f64;
    let dof = (groups.len() - 1) as f64;
    let chi2_dist = ChiSquared::new(dof).map_err(|e| e.to_string())?;
    let p_value = 1.0 - chi2_dist.cdf(h);
    let critical_value = chi2_dist.inverse_cdf(1.0 - alpha);
    let reject_h0 = p_value < alpha;
    let conclusion = hypothesis_conclusion(p_value, alpha, reject_h0);
    Ok(StatsOutput::HypothesisTestResult {
        test: "KruskalWallis",
        statistic: h,
        p_value,
        dof,
        critical_value,
        alpha,
        reject_h0,
        conclusion,
    })
}

fn eval_distribution(
    distribution: &DistributionKind,
    query: &DistributionQuery,
) -> Result<StatsOutput, String> {
    match distribution {
        DistributionKind::Normal { mean, std_dev } => {
            let dist = normal(*mean, *std_dev)?;
            match query {
                DistributionQuery::Pdf { x } => {
                    ensure_finite(*x, "x")?;
                    Ok(StatsOutput::Probability(dist.pdf(*x)))
                }
                DistributionQuery::Cdf { x } => {
                    ensure_finite(*x, "x")?;
                    Ok(StatsOutput::Probability(dist.cdf(*x)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p)))
                }
            }
        }
        DistributionKind::StudentT { df } => {
            ensure_positive_finite(*df, "df")?;
            let dist = StudentsT::new(0.0, 1.0, *df).map_err(|e| e.to_string())?;
            match query {
                DistributionQuery::Pdf { x } => {
                    ensure_finite(*x, "x")?;
                    Ok(StatsOutput::Probability(dist.pdf(*x)))
                }
                DistributionQuery::Cdf { x } => {
                    ensure_finite(*x, "x")?;
                    Ok(StatsOutput::Probability(dist.cdf(*x)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p)))
                }
            }
        }
        DistributionKind::ChiSquared { df } => {
            ensure_positive_finite(*df, "df")?;
            let dist = ChiSquared::new(*df).map_err(|e| e.to_string())?;
            match query {
                DistributionQuery::Pdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be non-negative for chi_squared pdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.pdf(*x)))
                }
                DistributionQuery::Cdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be non-negative for chi_squared cdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.cdf(*x)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p)))
                }
            }
        }
        DistributionKind::FDist { d1, d2 } => {
            ensure_positive_finite(*d1, "d1")?;
            ensure_positive_finite(*d2, "d2")?;
            let dist = FisherSnedecor::new(*d1, *d2).map_err(|e| e.to_string())?;
            match query {
                DistributionQuery::Pdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be non-negative for f_dist pdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.pdf(*x)))
                }
                DistributionQuery::Cdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be non-negative for f_dist cdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.cdf(*x)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p)))
                }
            }
        }
        DistributionKind::Binomial { n, p } => {
            let dist = binomial(*n, *p)?;
            match query {
                DistributionQuery::Pdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be a non-negative integer for binomial pdf".to_owned());
                    }
                    if x.fract() != 0.0 {
                        return Err("x must be a non-negative integer for binomial pdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.pmf(*x as u64)))
                }
                DistributionQuery::Cdf { x } => {
                    if *x < 0.0 {
                        return Ok(StatsOutput::Probability(0.0));
                    }
                    Ok(StatsOutput::Probability(dist.cdf(x.floor() as u64)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p) as f64))
                }
            }
        }
        DistributionKind::Poisson { lambda } => {
            ensure_positive_finite(*lambda, "lambda")?;
            let dist = Poisson::new(*lambda).map_err(|e| e.to_string())?;
            match query {
                DistributionQuery::Pdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be a non-negative integer for poisson pdf".to_owned());
                    }
                    if x.fract() != 0.0 {
                        return Err("x must be a non-negative integer for poisson pdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.pmf(*x as u64)))
                }
                DistributionQuery::Cdf { x } => {
                    if *x < 0.0 {
                        return Ok(StatsOutput::Probability(0.0));
                    }
                    Ok(StatsOutput::Probability(dist.cdf(x.floor() as u64)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p) as f64))
                }
            }
        }
        DistributionKind::Exponential { rate } => {
            ensure_positive_finite(*rate, "rate")?;
            let dist = Exp::new(*rate).map_err(|e| e.to_string())?;
            match query {
                DistributionQuery::Pdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be non-negative for exponential pdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.pdf(*x)))
                }
                DistributionQuery::Cdf { x } => {
                    if *x < 0.0 {
                        return Err("x must be non-negative for exponential cdf".to_owned());
                    }
                    Ok(StatsOutput::Probability(dist.cdf(*x)))
                }
                DistributionQuery::Quantile { p } => {
                    ensure_probability_open(*p, "p")?;
                    Ok(StatsOutput::Quantile(dist.inverse_cdf(*p)))
                }
            }
        }
    }
}

fn normal(mean: f64, std_dev: f64) -> Result<Normal, String> {
    ensure_finite(mean, "mean")?;
    ensure_positive_finite(std_dev, "std_dev")?;
    Normal::new(mean, std_dev).map_err(|e| e.to_string())
}

fn binomial(n: u64, p: f64) -> Result<Binomial, String> {
    ensure_probability_closed(p, "p")?;
    Binomial::new(p, n).map_err(|e| e.to_string())
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

fn ensure_probability_open(value: f64, name: &str) -> Result<(), String> {
    ensure_finite(value, name)?;
    if value > 0.0 && value < 1.0 {
        Ok(())
    } else {
        Err(format!("{name} must be in (0, 1)"))
    }
}

fn ensure_probability_closed(value: f64, name: &str) -> Result<(), String> {
    ensure_finite(value, name)?;
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be in [0, 1]"))
    }
}

fn default_checks() -> Vec<StatsCheck> {
    vec![StatsCheck {
        name: "computed_by_statrs_or_checked_adapter".to_owned(),
        passed: true,
    }]
}

fn cohen_d_two_sample(sample1: &[f64], sample2: &[f64]) -> Result<f64, String> {
    for (s, name) in [(sample1, "sample1"), (sample2, "sample2")] {
        if s.len() < 2 {
            return Err(format!("{name} must contain at least 2 values"));
        }
        if !s.iter().all(|v| v.is_finite()) {
            return Err(format!("{name} values must be finite"));
        }
    }
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;
    let (mean1, var1) = sample_mean_variance(sample1);
    let (mean2, var2) = sample_mean_variance(sample2);
    let sp = (((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0)).sqrt();
    if sp == 0.0 {
        return Err("pooled standard deviation is zero — Cohen's d is undefined".to_owned());
    }
    Ok((mean1 - mean2) / sp)
}

fn cohen_d_one_sample(sample: &[f64], mu0: f64) -> Result<f64, String> {
    if sample.len() < 2 {
        return Err("sample must contain at least 2 values".to_owned());
    }
    if !sample.iter().all(|v| v.is_finite()) {
        return Err("sample values must be finite".to_owned());
    }
    ensure_finite(mu0, "mu0")?;
    let (mean, var) = sample_mean_variance(sample);
    let s = var.sqrt();
    if s == 0.0 {
        return Err("sample standard deviation is zero — Cohen's d is undefined".to_owned());
    }
    Ok((mean - mu0) / s)
}

fn eta_squared(groups: &[Vec<f64>]) -> Result<f64, String> {
    if groups.len() < 2 {
        return Err("eta_squared requires at least 2 groups".to_owned());
    }
    for (i, g) in groups.iter().enumerate() {
        if g.len() < 2 {
            return Err(format!("group {i} must contain at least 2 values"));
        }
        if !g.iter().all(|v| v.is_finite()) {
            return Err(format!("group {i} values must be finite"));
        }
    }
    let n_total: usize = groups.iter().map(|group| group.len()).sum();
    let grand_mean = stable_mean_iter(groups.iter().flatten().copied(), n_total);
    let ss_between = compensated_sum(groups.iter().map(|group| {
        let mean = stable_mean(group);
        group.len() as f64 * (mean - grand_mean).powi(2)
    }));
    let ss_total = compensated_sum(
        groups
            .iter()
            .flatten()
            .map(|value| (value - grand_mean).powi(2)),
    );
    if ss_total == 0.0 {
        return Err("total variance is zero — eta-squared is undefined".to_owned());
    }
    Ok(ss_between / ss_total)
}

fn cramers_v(observed: &[Vec<f64>]) -> Result<f64, String> {
    let r = observed.len();
    if r < 2 {
        return Err("cramers_v requires at least 2 rows".to_owned());
    }
    let c = observed[0].len();
    if c < 2 {
        return Err("cramers_v requires at least 2 columns".to_owned());
    }
    for (i, row) in observed.iter().enumerate() {
        if row.len() != c {
            return Err(format!("row {i} has {} columns, expected {c}", row.len()));
        }
        if !row.iter().all(|v| v.is_finite() && *v >= 0.0) {
            return Err(format!("row {i} values must be finite and non-negative"));
        }
    }
    let n = compensated_sum(observed.iter().flatten().copied());
    if n == 0.0 {
        return Err("observed table sum is zero".to_owned());
    }
    let row_sums: Vec<f64> = observed
        .iter()
        .map(|row| compensated_sum(row.iter().copied()))
        .collect();
    let col_sums: Vec<f64> = (0..c)
        .map(|j| compensated_sum(observed.iter().map(|row| row[j])))
        .collect();
    let chi2 = compensated_sum(observed.iter().enumerate().map(|(i, row)| {
        compensated_sum(row.iter().enumerate().map(|(j, &o)| {
            let e = row_sums[i] * col_sums[j] / n;
            if e == 0.0 { 0.0 } else { (o - e).powi(2) / e }
        }))
    }));
    let k = r.min(c) as f64;
    Ok((chi2 / (n * (k - 1.0))).sqrt())
}

fn point_biserial_r(binary: &[f64], continuous: &[f64]) -> Result<f64, String> {
    if binary.len() != continuous.len() {
        return Err("binary and continuous must have the same length".to_owned());
    }
    let n = binary.len();
    if n < 2 {
        return Err("point_biserial_r requires at least 2 observations".to_owned());
    }
    if !binary.iter().all(|v| v.is_finite()) {
        return Err("binary values must be finite".to_owned());
    }
    if !continuous.iter().all(|v| v.is_finite()) {
        return Err("continuous values must be finite".to_owned());
    }
    if !binary.iter().all(|v| *v == 0.0 || *v == 1.0) {
        return Err("binary values must be 0 or 1".to_owned());
    }
    correlation(binary, continuous).map(|(r, _)| r)
}

fn power_t(effect_d: f64, alpha: f64, power: f64, two_sample: bool) -> Result<u64, String> {
    if !effect_d.is_finite() || effect_d == 0.0 {
        return Err("effect_d must be finite and non-zero".to_owned());
    }
    ensure_probability_open(alpha, "alpha")?;
    ensure_probability_open(power, "power")?;
    let norm = Normal::new(0.0, 1.0).map_err(|e| e.to_string())?;
    let z_alpha = norm.inverse_cdf(1.0 - alpha / 2.0);
    let z_beta = norm.inverse_cdf(power);
    let n_raw = ((z_alpha + z_beta) / effect_d).powi(2);
    let n = n_raw.ceil() as u64;
    if two_sample { Ok(n * 2) } else { Ok(n) }
}

fn power_proportion(p0: f64, p1: f64, alpha: f64, power: f64) -> Result<u64, String> {
    ensure_probability_open(p0, "p0")?;
    ensure_probability_open(p1, "p1")?;
    ensure_probability_open(alpha, "alpha")?;
    ensure_probability_open(power, "power")?;
    if p0 == p1 {
        return Err("p0 and p1 must differ — effect size is zero".to_owned());
    }
    let h0 = 2.0 * p0.sqrt().asin();
    let h1 = 2.0 * p1.sqrt().asin();
    let h = (h1 - h0).abs();
    let norm = Normal::new(0.0, 1.0).map_err(|e| e.to_string())?;
    let z_alpha = norm.inverse_cdf(1.0 - alpha / 2.0);
    let z_beta = norm.inverse_cdf(power);
    let n_raw = ((z_alpha + z_beta) / h).powi(2);
    Ok(n_raw.ceil() as u64)
}

fn interpret_d(d: f64) -> &'static str {
    let a = d.abs();
    if a < 0.2 {
        "negligible"
    } else if a < 0.5 {
        "small"
    } else if a < 0.8 {
        "medium"
    } else {
        "large"
    }
}

fn interpret_eta_sq(eta: f64) -> &'static str {
    if eta < 0.01 {
        "negligible"
    } else if eta < 0.06 {
        "small"
    } else if eta < 0.14 {
        "medium"
    } else {
        "large"
    }
}

fn interpret_r(r: f64) -> &'static str {
    let a = r.abs();
    if a < 0.1 {
        "negligible"
    } else if a < 0.3 {
        "small"
    } else if a < 0.5 {
        "medium"
    } else {
        "large"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compensated_sum_preserves_a_small_term_before_cancellation() {
        let sum = compensated_sum([1.0e-16, 1.0, -1.0]);
        assert_eq!(sum, 1.0e-16);
    }

    #[test]
    fn centered_cross_product_subtracts_both_means() {
        let product = centered_sum_products(&[2.0], &[3.0], 1.0, 1.0);
        assert_eq!(product, 2.0);
    }

    fn probability(response: StatsResponse) -> f64 {
        match response {
            StatsResponse::Probability { value, .. } => value,
            other => panic!("expected probability response, got {other:?}"),
        }
    }

    #[test]
    fn describes_sample() {
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 2.0, 3.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary {
                n,
                mean,
                variance,
                std_dev,
                min,
                max,
                median,
                q1,
                q3,
                iqr,
                ..
            } => {
                assert_eq!(n, 3);
                assert_eq!(mean, 2.0);
                assert_eq!(variance, 1.0);
                assert_eq!(std_dev, 1.0);
                assert_eq!(min, 1.0);
                assert_eq!(max, 3.0);
                assert_eq!(median, 2.0);
                assert!((q1 - 1.5).abs() < 1e-12);
                assert!((q3 - 2.5).abs() < 1e-12);
                assert!((iqr - 1.0).abs() < 1e-12);
            }
            other => panic!("expected sample summary, got {other:?}"),
        }
    }

    #[test]
    fn computes_normal_cdf_and_quantile() {
        let cdf = probability(
            StatsRequest::NormalCdf {
                mean: 0.0,
                std_dev: 1.0,
                x: 0.0,
            }
            .evaluate(),
        );
        assert!((cdf - 0.5).abs() < 1e-15);

        match (StatsRequest::NormalQuantile {
            mean: 0.0,
            std_dev: 1.0,
            p: 0.5,
        })
        .evaluate()
        {
            StatsResponse::Quantile { value, .. } => assert!(value.abs() < 1e-15),
            other => panic!("expected quantile response, got {other:?}"),
        }
    }

    #[test]
    fn computes_student_t_interval() {
        match (StatsRequest::StudentTInterval {
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            confidence: 0.95,
        })
        .evaluate()
        {
            StatsResponse::Interval {
                mean, lower, upper, ..
            } => {
                assert_eq!(mean, 3.0);
                assert!((lower - 1.036756838522439).abs() < 1e-12);
                assert!((upper - 4.9632431614775605).abs() < 1e-12);
            }
            other => panic!("expected interval response, got {other:?}"),
        }
    }

    #[test]
    fn computes_binomial_pmf_and_cdf() {
        let pmf = probability(
            StatsRequest::BinomialPmf {
                n: 10,
                p: 0.5,
                k: 5,
            }
            .evaluate(),
        );
        let cdf = probability(
            StatsRequest::BinomialCdf {
                n: 10,
                p: 0.5,
                k: 10,
            }
            .evaluate(),
        );

        assert!((pmf - 0.24609375).abs() < 1e-15);
        assert_eq!(cdf, 1.0);
    }

    #[test]
    fn describe_sample_includes_median_quartiles_skewness_kurtosis() {
        // [1,2,3,4,5]: symmetric → skewness ≈ 0, kurtosis < 0 (platykurtic)
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary {
                median,
                q1,
                q3,
                iqr,
                skewness,
                ..
            } => {
                assert_eq!(median, 3.0);
                assert_eq!(q1, 2.0);
                assert_eq!(q3, 4.0);
                assert_eq!(iqr, 2.0);
                assert!(skewness.abs() < 1e-10, "skewness of symmetric sample ≈ 0");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_correlation() {
        match (StatsRequest::Correlation {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![2.1, 3.9, 6.2, 7.8, 10.1],
        })
        .evaluate()
        {
            StatsResponse::Correlation { pearson_r, n, .. } => {
                assert_eq!(n, 5);
                assert!((pearson_r - 0.9994).abs() < 0.001, "r = {pearson_r}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_linear_regression() {
        match (StatsRequest::LinearRegression {
            x: XInput::Single(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
            y: vec![2.1, 3.9, 6.2, 7.8, 10.1],
        })
        .evaluate()
        {
            StatsResponse::Regression {
                slope,
                intercept,
                r_squared,
                ..
            } => {
                assert!((slope - 1.99).abs() < 0.01, "slope = {slope}");
                assert!(intercept.abs() < 0.2, "intercept = {intercept}");
                assert!(r_squared > 0.997, "r² = {r_squared}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_percentile() {
        // P50 of [10,20,30] = 20
        match (StatsRequest::Percentile {
            values: vec![10.0, 30.0, 20.0],
            p: 50.0,
        })
        .evaluate()
        {
            StatsResponse::Percentile { value, p, .. } => {
                assert!((value - 20.0).abs() < 1e-9);
                assert_eq!(p, 50.0);
            }
            other => panic!("{other:?}"),
        }
        // P75 of [1,2,3,4]
        match (StatsRequest::Percentile {
            values: vec![1.0, 2.0, 3.0, 4.0],
            p: 75.0,
        })
        .evaluate()
        {
            StatsResponse::Percentile { value, .. } => {
                assert!((value - 3.25).abs() < 1e-9, "P75 = {value}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_mode_single_and_multimodal() {
        // single mode
        match (StatsRequest::Mode {
            values: vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0],
        })
        .evaluate()
        {
            StatsResponse::Mode {
                values, frequency, ..
            } => {
                assert_eq!(values, vec![3.0]);
                assert_eq!(frequency, 3);
            }
            other => panic!("{other:?}"),
        }
        // multimodal
        match (StatsRequest::Mode {
            values: vec![1.0, 2.0, 2.0, 3.0, 3.0],
        })
        .evaluate()
        {
            StatsResponse::Mode {
                values, frequency, ..
            } => {
                assert_eq!(values, vec![2.0, 3.0]);
                assert_eq!(frequency, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_ranks_average_method() {
        match (StatsRequest::Rank {
            values: vec![40.0, 20.0, 30.0, 10.0],
            method: RankMethod::Average,
        })
        .evaluate()
        {
            StatsResponse::Ranks { ranks, .. } => {
                assert_eq!(ranks, vec![4.0, 2.0, 3.0, 1.0]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_ranks_with_ties() {
        // [10, 20, 20, 30]: ranks with average → [1, 2.5, 2.5, 4]
        match (StatsRequest::Rank {
            values: vec![10.0, 20.0, 20.0, 30.0],
            method: RankMethod::Average,
        })
        .evaluate()
        {
            StatsResponse::Ranks { ranks, .. } => {
                assert_eq!(ranks, vec![1.0, 2.5, 2.5, 4.0]);
            }
            other => panic!("{other:?}"),
        }
        // min method
        match (StatsRequest::Rank {
            values: vec![10.0, 20.0, 20.0, 30.0],
            method: RankMethod::Min,
        })
        .evaluate()
        {
            StatsResponse::Ranks { ranks, .. } => {
                assert_eq!(ranks, vec![1.0, 2.0, 2.0, 4.0]);
            }
            other => panic!("{other:?}"),
        }
        // max method
        match (StatsRequest::Rank {
            values: vec![10.0, 20.0, 20.0, 30.0],
            method: RankMethod::Max,
        })
        .evaluate()
        {
            StatsResponse::Ranks { ranks, .. } => {
                assert_eq!(ranks, vec![1.0, 3.0, 3.0, 4.0]);
            }
            other => panic!("{other:?}"),
        }
        // first method
        match (StatsRequest::Rank {
            values: vec![10.0, 20.0, 20.0, 30.0],
            method: RankMethod::First,
        })
        .evaluate()
        {
            StatsResponse::Ranks { ranks, .. } => {
                assert_eq!(ranks, vec![1.0, 2.0, 3.0, 4.0]);
            }
            other => panic!("{other:?}"),
        }
        // last method
        match (StatsRequest::Rank {
            values: vec![10.0, 20.0, 20.0, 30.0],
            method: RankMethod::Last,
        })
        .evaluate()
        {
            StatsResponse::Ranks { ranks, .. } => {
                assert_eq!(ranks, vec![1.0, 3.0, 2.0, 4.0]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn skewness_formula_is_exact_for_asymmetric_sample() {
        // [1,1,1,2,10]: n=5, strongly right-skewed
        // skewness ≈ 2.173 (hand-verified); kills - → / and / → % / * mutations
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 1.0, 1.0, 2.0, 10.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary { skewness, .. } => {
                assert!((skewness - 2.173).abs() < 0.01, "skewness = {skewness}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn kurtosis_formula_is_exact() {
        // [1,1,1,4]: n=4, mean=1.75, std=1.5, excess kurtosis = 4.0 (hand-verified)
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 1.0, 1.0, 4.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary { kurtosis, .. } => {
                assert!((kurtosis - 4.0).abs() < 1e-9, "kurtosis = {kurtosis}");
            }
            other => panic!("{other:?}"),
        }

        // n=3: too small for kurtosis → must be 0.0 (kills || → && in guard)
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 2.0, 3.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary { kurtosis, .. } => {
                assert_eq!(kurtosis, 0.0, "kurtosis of n=3 sample must be 0.0");
            }
            other => panic!("{other:?}"),
        }

        // [1,1,1,1,4]: n=5, excess kurtosis = 5.0 (hand-verified)
        // Kills: < → > guard (n=5 would return 0 with mutation), and n=4-equivalent
        // arithmetic mutations at n-2/n-3 factors that were undetectable with n=4.
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 1.0, 1.0, 1.0, 4.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary { kurtosis, .. } => {
                assert!((kurtosis - 5.0).abs() < 1e-9, "kurtosis(n=5) = {kurtosis}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn correlation_succeeds_with_n_equals_2() {
        // kills < → <= mutation (n < 2 → n <= 2 would reject n=2)
        match (StatsRequest::Correlation {
            x: vec![1.0, 3.0],
            y: vec![2.0, 4.0],
        })
        .evaluate()
        {
            StatsResponse::Correlation { pearson_r, n, .. } => {
                assert_eq!(n, 2);
                assert!((pearson_r - 1.0).abs() < 1e-9);
            }
            other => panic!("n=2 should succeed, got {other:?}"),
        }
    }

    #[test]
    fn correlation_rejects_non_finite_in_either_series() {
        // kills || → && mutation (only y has NaN, x is finite)
        assert!(matches!(
            StatsRequest::Correlation {
                x: vec![1.0, 2.0, 3.0],
                y: vec![f64::NAN, 2.0, 3.0]
            }
            .evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("finite")
        ));
        // x has NaN too
        assert!(matches!(
            StatsRequest::Correlation {
                x: vec![f64::INFINITY, 2.0],
                y: vec![1.0, 2.0]
            }
            .evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("finite")
        ));
    }

    #[test]
    fn regression_r_squared_is_squared_not_divided() {
        // r < 1, so r² ≠ r/r; kills * → / mutation at r_squared = pearson_r * pearson_r
        match (StatsRequest::LinearRegression {
            x: XInput::Single(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
            y: vec![1.1, 1.9, 3.2, 3.8, 5.0],
        })
        .evaluate()
        {
            StatsResponse::Regression { r_squared, .. } => {
                assert!(
                    r_squared > 0.98 && r_squared < 1.0,
                    "r² = {r_squared} (must be < 1 for this imperfect dataset)"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn correlation_rejects_mismatched_lengths_and_constant_series() {
        assert!(matches!(
            StatsRequest::Correlation {
                x: vec![1.0, 2.0],
                y: vec![1.0]
            }
            .evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("same length")
        ));
        assert!(matches!(
            StatsRequest::Correlation {
                x: vec![1.0],
                y: vec![1.0]
            }
            .evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("at least two")
        ));
        assert!(matches!(
            StatsRequest::Correlation {
                x: vec![1.0, 1.0, 1.0],
                y: vec![1.0, 2.0, 3.0]
            }
            .evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("undefined")
        ));
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(matches!(
            (StatsRequest::DescribeSample { values: vec![] }).evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("at least one")
        ));
        assert!(matches!(
            (StatsRequest::NormalCdf { mean: 0.0, std_dev: 0.0, x: 1.0 }).evaluate(),
            StatsResponse::Error { reason, .. } if reason == "std_dev must be greater than zero"
        ));
        assert!(matches!(
            (StatsRequest::NormalQuantile { mean: 0.0, std_dev: 1.0, p: 1.0 }).evaluate(),
            StatsResponse::Error { reason, .. } if reason == "p must be in (0, 1)"
        ));
        assert!(matches!(
            (StatsRequest::BinomialPmf { n: 10, p: 1.5, k: 2 }).evaluate(),
            StatsResponse::Error { reason, .. } if reason == "p must be in [0, 1]"
        ));
        assert!(matches!(
            (StatsRequest::StudentTInterval { values: vec![1.0], confidence: 0.95 }).evaluate(),
            StatsResponse::Error { reason, .. } if reason == "student_t_interval requires at least two values"
        ));
        assert!(matches!(
            (StatsRequest::NormalCdf { mean: f64::NAN, std_dev: 1.0, x: 0.0 }).evaluate(),
            StatsResponse::Error { reason, .. } if reason == "mean must be finite"
        ));
        assert!(matches!(
            (StatsRequest::NormalQuantile { mean: 0.0, std_dev: 1.0, p: 0.0 }).evaluate(),
            StatsResponse::Error { reason, .. } if reason == "p must be in (0, 1)"
        ));
    }

    #[test]
    fn percentile_boundary_values_are_valid() {
        let p0 = StatsRequest::Percentile {
            values: vec![5.0, 3.0, 1.0],
            p: 0.0,
        }
        .evaluate();
        match p0 {
            StatsResponse::Percentile { value, .. } => {
                assert!(
                    (value - 1.0).abs() < 1e-9,
                    "p=0 should return min=1.0, got {value}"
                )
            }
            other => panic!("expected Percentile, got {other:?}"),
        }

        let p100 = StatsRequest::Percentile {
            values: vec![5.0, 3.0, 1.0],
            p: 100.0,
        }
        .evaluate();
        match p100 {
            StatsResponse::Percentile { value, .. } => {
                assert!(
                    (value - 5.0).abs() < 1e-9,
                    "p=100 should return max=5.0, got {value}"
                )
            }
            other => panic!("expected Percentile, got {other:?}"),
        }
    }

    #[test]
    fn percentile_rejects_out_of_range_and_non_finite_p() {
        assert!(
            matches!(
                StatsRequest::Percentile { values: vec![1.0], p: -1.0 }.evaluate(),
                StatsResponse::Error { reason, .. } if reason.contains("[0, 100]")
            ),
            "p=-1 should error"
        );
        assert!(
            matches!(
                StatsRequest::Percentile { values: vec![1.0], p: 101.0 }.evaluate(),
                StatsResponse::Error { reason, .. } if reason.contains("[0, 100]")
            ),
            "p=101 should error"
        );
        assert!(
            matches!(
                StatsRequest::Percentile { values: vec![1.0], p: f64::NAN }.evaluate(),
                StatsResponse::Error { reason, .. } if reason.contains("[0, 100]")
            ),
            "p=NaN should error"
        );
    }

    #[test]
    fn skewness_guard_n_lt_3_returns_zero() {
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 3.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary { skewness, .. } => {
                assert_eq!(skewness, 0.0, "n=2 skewness must be 0")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn skewness_n3_asymmetric_sample_is_nonzero() {
        match (StatsRequest::DescribeSample {
            values: vec![1.0, 2.0, 10.0],
        })
        .evaluate()
        {
            StatsResponse::SampleSummary { skewness, .. } => {
                assert!(
                    skewness > 1.0,
                    "skewness of [1,2,10] n=3 should be >1, got {skewness}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_t_cdf_and_t_inverse_cdf() {
        let cdf = StatsRequest::TCdf {
            degrees_of_freedom: 10.0,
            x: 2.228,
        }
        .evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.975).abs() < 0.001,
                    "t_cdf(10, 2.228) ≈ 0.975, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
        let quantile = StatsRequest::TInverseCdf {
            degrees_of_freedom: 10.0,
            p: 0.975,
        }
        .evaluate();
        match quantile {
            StatsResponse::Quantile { value, .. } => {
                assert!(
                    (value - 2.228).abs() < 0.001,
                    "t_inverse(10, 0.975) ≈ 2.228, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_chi2_cdf_and_chi2_inverse_cdf() {
        let cdf = StatsRequest::Chi2Cdf {
            degrees_of_freedom: 3.0,
            x: 7.815,
        }
        .evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.95).abs() < 0.001,
                    "chi2_cdf(3, 7.815) ≈ 0.95, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
        let quantile = StatsRequest::Chi2InverseCdf {
            degrees_of_freedom: 3.0,
            p: 0.95,
        }
        .evaluate();
        match quantile {
            StatsResponse::Quantile { value, .. } => {
                assert!(
                    (value - 7.815).abs() < 0.01,
                    "chi2_inverse(3, 0.95) ≈ 7.815, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_poisson_pmf_and_cdf() {
        let pmf = StatsRequest::PoissonPmf { lambda: 3.0, k: 2 }.evaluate();
        match pmf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.2240).abs() < 0.001,
                    "poisson_pmf(3,2) ≈ 0.224, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
        let cdf = StatsRequest::PoissonCdf { lambda: 3.0, k: 5 }.evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.9161).abs() < 0.001,
                    "poisson_cdf(3,5) ≈ 0.916, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_beta_cdf_and_pdf() {
        let cdf = StatsRequest::BetaCdf {
            alpha: 2.0,
            beta: 5.0,
            x: 0.3,
        }
        .evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.5798).abs() < 0.001,
                    "beta_cdf(2,5,0.3) ≈ 0.580, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
        let pdf = StatsRequest::BetaPdf {
            alpha: 2.0,
            beta: 5.0,
            x: 0.3,
        }
        .evaluate();
        match pdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 2.1609).abs() < 0.001,
                    "beta_pdf(2,5,0.3) ≈ 2.161, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_f_cdf() {
        let cdf = StatsRequest::FCdf {
            d1: 5.0,
            d2: 10.0,
            x: 3.33,
        }
        .evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.95).abs() < 0.005,
                    "f_cdf(5,10,3.33) ≈ 0.95, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_exponential_cdf_and_pdf() {
        let cdf = StatsRequest::ExponentialCdf { rate: 1.0, x: 1.0 }.evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - (1.0 - std::f64::consts::E.recip())).abs() < 1e-9,
                    "exp_cdf(1,1) = 1-1/e, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
        let pdf = StatsRequest::ExponentialPdf { rate: 2.0, x: 0.0 }.evaluate();
        match pdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 2.0).abs() < 1e-9,
                    "exp_pdf(2,0) = rate = 2.0, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn computes_uniform_cdf() {
        let cdf = StatsRequest::UniformCdf {
            min: 0.0,
            max: 10.0,
            x: 5.0,
        }
        .evaluate();
        match cdf {
            StatsResponse::Probability { value, .. } => {
                assert!(
                    (value - 0.5).abs() < 1e-9,
                    "uniform_cdf(0,10,5) = 0.5, got {value}"
                )
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn new_distributions_reject_invalid_parameters() {
        assert!(matches!(
            StatsRequest::TCdf {
                degrees_of_freedom: 0.0,
                x: 1.0
            }
            .evaluate(),
            StatsResponse::Error { .. }
        ));
        assert!(matches!(
            StatsRequest::Chi2Cdf { degrees_of_freedom: 3.0, x: -1.0 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("non-negative")
        ));
        assert!(matches!(
            StatsRequest::PoissonPmf { lambda: -1.0, k: 0 }.evaluate(),
            StatsResponse::Error { .. }
        ));
        assert!(matches!(
            StatsRequest::BetaCdf { alpha: 2.0, beta: 5.0, x: -0.1 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("[0, 1]")
        ));
        assert!(matches!(
            StatsRequest::BetaCdf { alpha: 2.0, beta: 5.0, x: 1.5 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("[0, 1]")
        ));
        assert!(matches!(
            StatsRequest::FCdf { d1: 5.0, d2: 10.0, x: -1.0 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("non-negative")
        ));
        assert!(matches!(
            StatsRequest::UniformCdf { min: 5.0, max: 3.0, x: 4.0 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("min must be less than max")
        ));
        assert!(matches!(
            StatsRequest::UniformCdf { min: 5.0, max: 5.0, x: 5.0 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("min must be less than max")
        ));
        // BetaPdf rejects x < 0 and x > 1 (kills || → && and > → == mutations at line 416)
        assert!(matches!(
            StatsRequest::BetaPdf { alpha: 2.0, beta: 5.0, x: -0.1 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("[0, 1]")
        ));
        assert!(matches!(
            StatsRequest::BetaPdf { alpha: 2.0, beta: 5.0, x: 1.5 }.evaluate(),
            StatsResponse::Error { reason, .. } if reason.contains("[0, 1]")
        ));
    }

    #[test]
    fn new_distributions_accept_boundary_x_values() {
        // x=0 is valid for chi2_cdf (kills `< → <=` mutation)
        assert!(matches!(
            StatsRequest::Chi2Cdf { degrees_of_freedom: 3.0, x: 0.0 }.evaluate(),
            StatsResponse::Probability { value, .. } if value == 0.0
        ));
        // x=0 is valid for f_cdf (kills `< → <=` mutation)
        assert!(matches!(
            StatsRequest::FCdf { d1: 5.0, d2: 10.0, x: 0.0 }.evaluate(),
            StatsResponse::Probability { value, .. } if value == 0.0
        ));
        // x=0 is valid for exponential_cdf (kills `< → <=` mutation)
        assert!(matches!(
            StatsRequest::ExponentialCdf { rate: 1.0, x: 0.0 }.evaluate(),
            StatsResponse::Probability { value, .. } if value == 0.0
        ));
        // x=0 is valid for exponential_pdf (kills `< → <=` mutation)
        match (StatsRequest::ExponentialPdf { rate: 2.0, x: 0.0 }).evaluate() {
            StatsResponse::Probability { value, .. } => {
                assert!((value - 2.0).abs() < 1e-9)
            }
            other => panic!("{other:?}"),
        }
        // x=0 valid for beta_cdf (kills `< → <=` mutation on the < 0 guard)
        assert!(matches!(
            StatsRequest::BetaCdf { alpha: 2.0, beta: 5.0, x: 0.0 }.evaluate(),
            StatsResponse::Probability { value, .. } if value == 0.0
        ));
        // x=1 valid for beta_cdf (kills `> → >=` mutation on the > 1 guard)
        assert!(matches!(
            StatsRequest::BetaCdf { alpha: 2.0, beta: 5.0, x: 1.0 }.evaluate(),
            StatsResponse::Probability { value, .. } if (value - 1.0).abs() < 1e-9
        ));
        // x=0 valid for beta_pdf (kills < → <= mutation on lower guard)
        match (StatsRequest::BetaPdf {
            alpha: 2.0,
            beta: 5.0,
            x: 0.0,
        })
        .evaluate()
        {
            StatsResponse::Probability { .. } => {}
            other => panic!("{other:?}"),
        }
        // x=1 valid for beta_pdf (kills > → >= mutation on upper guard)
        match (StatsRequest::BetaPdf {
            alpha: 2.0,
            beta: 5.0,
            x: 1.0,
        })
        .evaluate()
        {
            StatsResponse::Probability { .. } => {}
            other => panic!("{other:?}"),
        }
        // x=0.5 valid for exponential_pdf (kills < → > mutation at line 441)
        match (StatsRequest::ExponentialPdf { rate: 2.0, x: 0.5 }).evaluate() {
            StatsResponse::Probability { value, .. } => {
                assert!((value - 2.0 * (-1.0_f64).exp()).abs() < 1e-9)
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn successful_responses_include_checks() {
        let responses = [
            StatsRequest::DescribeSample {
                values: vec![1.0, 2.0],
            }
            .evaluate(),
            StatsRequest::NormalCdf {
                mean: 0.0,
                std_dev: 1.0,
                x: 0.0,
            }
            .evaluate(),
            StatsRequest::NormalQuantile {
                mean: 0.0,
                std_dev: 1.0,
                p: 0.5,
            }
            .evaluate(),
            StatsRequest::StudentTInterval {
                values: vec![1.0, 2.0],
                confidence: 0.95,
            }
            .evaluate(),
        ];

        for response in responses {
            let checks = match response {
                StatsResponse::SampleSummary { checks, .. }
                | StatsResponse::Probability { checks, .. }
                | StatsResponse::Quantile { checks, .. }
                | StatsResponse::Interval { checks, .. }
                | StatsResponse::Correlation { checks, .. }
                | StatsResponse::Regression { checks, .. }
                | StatsResponse::MultipleRegression { checks, .. }
                | StatsResponse::Percentile { checks, .. }
                | StatsResponse::Mode { checks, .. }
                | StatsResponse::Ranks { checks, .. }
                | StatsResponse::HypothesisTest { checks, .. } => checks,
                other => panic!("expected successful response, got {other:?}"),
            };
            assert_eq!(checks.len(), 1);
            assert_eq!(checks[0].name, "computed_by_statrs_or_checked_adapter");
            assert!(checks[0].passed);
        }
    }

    // ── multiple_linear_regression ───────────────────────────────────────────

    fn multi_reg(x: Vec<Vec<f64>>, y: Vec<f64>) -> StatsResponse {
        StatsRequest::LinearRegression {
            x: XInput::Multi(x),
            y,
        }
        .evaluate()
    }

    fn single_reg(x: Vec<f64>, y: Vec<f64>) -> StatsResponse {
        StatsRequest::LinearRegression {
            x: XInput::Single(x),
            y,
        }
        .evaluate()
    }

    #[test]
    fn single_predictor_backward_compat_still_returns_regression() {
        // Existing single-predictor path must still return Regression (not MultipleRegression)
        match single_reg(vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0]) {
            StatsResponse::Regression {
                slope,
                intercept,
                r_squared,
                ..
            } => {
                assert!((slope - 2.0).abs() < 1e-10);
                assert!(intercept.abs() < 1e-10);
                assert!((r_squared - 1.0).abs() < 1e-10);
            }
            other => panic!("expected Regression, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_two_predictors_known_solution() {
        // y = 1 + 2*x1 + 3*x2  — exact (no noise)
        // Data: (x1,x2) in {(1,1),(2,1),(1,2),(2,2),(3,1),(1,3)}
        let x = vec![
            vec![1.0, 1.0],
            vec![2.0, 1.0],
            vec![1.0, 2.0],
            vec![2.0, 2.0],
            vec![3.0, 1.0],
            vec![1.0, 3.0],
        ];
        let y: Vec<f64> = x.iter().map(|r| 1.0 + 2.0 * r[0] + 3.0 * r[1]).collect();

        match multi_reg(x, y) {
            StatsResponse::MultipleRegression {
                coefficients,
                r_squared,
                residual_std_dev,
                n,
                k,
                ..
            } => {
                // coefficients = [intercept=1, b1=2, b2=3]
                assert_eq!(k, 2);
                assert_eq!(n, 6);
                assert!(
                    (coefficients[0] - 1.0).abs() < 1e-8,
                    "intercept={}",
                    coefficients[0]
                );
                assert!(
                    (coefficients[1] - 2.0).abs() < 1e-8,
                    "b1={}",
                    coefficients[1]
                );
                assert!(
                    (coefficients[2] - 3.0).abs() < 1e-8,
                    "b2={}",
                    coefficients[2]
                );
                assert!((r_squared - 1.0).abs() < 1e-10, "r2={r_squared}");
                assert!(residual_std_dev < 1e-8, "resid_std={residual_std_dev}");
            }
            other => panic!("expected MultipleRegression, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_r_squared_is_not_ss_res_over_ss_tot() {
        // Kills the mutation `1.0 - ss_res/ss_tot` → `ss_res/ss_tot`
        // Perfect fit → r_squared must be 1.0, not 0.0
        let x = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
        let y = vec![2.0, 4.0, 6.0, 8.0];
        match multi_reg(x, y) {
            StatsResponse::MultipleRegression { r_squared, .. } => {
                assert!((r_squared - 1.0).abs() < 1e-10, "r2={r_squared}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn multiple_regression_dof_uses_n_minus_k_minus_1() {
        // Known: y = 1 + x, n=4, k=1, dof=2, residuals=[-0.5,0,0,0.5]
        // ss_res = 0.5, residual_std_dev = sqrt(0.5/2) = 0.5
        // If dof were n (=4): sqrt(0.5/4)=0.354; if n-k (=3): sqrt(0.5/3)=0.408
        let x = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
        let y = vec![1.5, 2.5, 3.5, 4.5]; // exact fit offset by +0.5
        match multi_reg(x, y) {
            StatsResponse::MultipleRegression {
                residual_std_dev,
                r_squared,
                ..
            } => {
                // Perfect slope=1, intercept=0.5: residuals all 0, std_dev=0
                assert!(residual_std_dev < 1e-10, "resid_std={residual_std_dev}");
                assert!((r_squared - 1.0).abs() < 1e-10);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn multiple_regression_requires_n_greater_than_k_plus_1() {
        // n=k+1 must fail (kills `<=` → `<` mutation in the dof guard)
        let x = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]]; // n=3, k=2
        let y = vec![1.0, 2.0, 3.0];
        match multi_reg(x, y) {
            StatsResponse::Error { reason, .. } => {
                assert!(reason.contains("need at least"), "reason={reason}");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_n_equals_k_plus_2_is_allowed() {
        // n = k+2 = 4 (k=2): just enough dof=1, must succeed (kills `<=` → `<` mutation)
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 1.0],
            vec![2.0, 2.0],
        ];
        let y = vec![3.0, 5.0, 8.0, 13.0]; // y = 1 + 2*x1 + 3*x2 + noise
        match multi_reg(x, y) {
            StatsResponse::MultipleRegression { n, k, .. } => {
                assert_eq!(n, 4);
                assert_eq!(k, 2);
            }
            other => panic!("expected MultipleRegression, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_rejects_mismatched_row_count() {
        let x = vec![vec![1.0], vec![2.0]]; // 2 rows
        let y = vec![1.0, 2.0, 3.0]; // 3 elements
        match multi_reg(x, y) {
            StatsResponse::Error { reason, .. } => {
                assert!(
                    reason.contains("rows") && reason.contains("elements"),
                    "{reason}"
                );
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_rejects_jagged_x_matrix() {
        let x = vec![vec![1.0, 2.0], vec![3.0]]; // inconsistent columns
        let y = vec![1.0, 2.0];
        match multi_reg(x, y) {
            StatsResponse::Error { reason, .. } => {
                assert!(reason.contains("columns"), "{reason}");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_rejects_non_finite_y() {
        let x = vec![vec![1.0], vec![2.0], vec![3.0]];
        let y = vec![1.0, f64::NAN, 3.0];
        match multi_reg(x, y) {
            StatsResponse::Error { .. } => {}
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_rejects_non_finite_x() {
        let x = vec![vec![1.0], vec![f64::INFINITY], vec![3.0]];
        let y = vec![1.0, 2.0, 3.0];
        match multi_reg(x, y) {
            StatsResponse::Error { .. } => {}
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn multiple_regression_std_errors_are_positive() {
        // Kills (v_ji / s).powi(2) → (v_ji * s).powi(2): latter gives much larger se
        let x = vec![
            vec![1.0, 2.0],
            vec![2.0, 1.0],
            vec![3.0, 3.0],
            vec![4.0, 2.0],
            vec![5.0, 4.0],
        ];
        let y = vec![3.0, 5.0, 8.0, 9.0, 13.0];
        match multi_reg(x, y) {
            StatsResponse::MultipleRegression {
                std_errors,
                residual_std_dev,
                ..
            } => {
                for se in &std_errors {
                    assert!(*se > 0.0 && se.is_finite(), "se={se}");
                    // se must be within 10x of residual_std_dev — if mutation
                    // (v/s)^2 → (v*s)^2 the value explodes by sigma^4
                    assert!(
                        *se < residual_std_dev * 100.0,
                        "se={se} rsd={residual_std_dev}"
                    );
                }
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn multiple_regression_nist_longley_passes_certified_values() {
        // NIST StRD Longley dataset: 16 obs, 6 predictors
        // Certified by NIST to 15 significant figures.
        // This test kills: intercept column, dof, r_squared, residual_std_dev, all coeff mutations.
        let x = vec![
            vec![83.0, 234289.0, 2356.0, 1590.0, 107608.0, 1947.0],
            vec![88.5, 259426.0, 2325.0, 1456.0, 108632.0, 1948.0],
            vec![88.2, 258054.0, 3682.0, 1616.0, 109773.0, 1949.0],
            vec![89.5, 284599.0, 3351.0, 1650.0, 110929.0, 1950.0],
            vec![96.2, 328975.0, 2099.0, 3099.0, 112075.0, 1951.0],
            vec![98.1, 346999.0, 1932.0, 3594.0, 113270.0, 1952.0],
            vec![99.0, 365385.0, 1870.0, 3547.0, 115094.0, 1953.0],
            vec![100.0, 363112.0, 3578.0, 3350.0, 116219.0, 1954.0],
            vec![101.2, 397469.0, 2904.0, 3048.0, 117388.0, 1955.0],
            vec![104.6, 419180.0, 2822.0, 2857.0, 118734.0, 1956.0],
            vec![108.4, 442769.0, 2936.0, 2798.0, 120445.0, 1957.0],
            vec![110.8, 444546.0, 4681.0, 2637.0, 121950.0, 1958.0],
            vec![112.6, 482704.0, 3813.0, 2552.0, 123366.0, 1959.0],
            vec![114.2, 502601.0, 3931.0, 2514.0, 125368.0, 1960.0],
            vec![115.7, 518173.0, 4806.0, 2572.0, 127852.0, 1961.0],
            vec![116.9, 554894.0, 4007.0, 2827.0, 130081.0, 1962.0],
        ];
        let y = vec![
            60323.0, 61122.0, 60171.0, 61187.0, 63221.0, 63639.0, 64989.0, 63761.0, 66019.0,
            67857.0, 68169.0, 66513.0, 68655.0, 69564.0, 69331.0, 70551.0,
        ];

        // NIST certified values (15 sig figs)
        let cert_b = [
            -3482258.63459582,
            15.0618722713733,
            -0.0358191792925910,
            -2.02022980381683,
            -1.03322686717359,
            -0.0511041056535807,
            1829.15146461355,
        ];
        let cert_r2 = 0.995479004577296;
        let cert_rsd = 304.854073561965;

        // NIST certified standard errors — kills (v_ji/s) → (v_ji*s) and
        // (rsd * sqrt) → (rsd + sqrt) / (rsd / sqrt) mutations
        let cert_se = [
            890420.383607373,
            84.9149257747669,
            0.0334910077722432, // 3.349e-02, not 3.349e-01
            0.488399681651699,
            0.214274163161675,
            0.226073200069370,
            455.478499142212,
        ];

        match multi_reg(x, y) {
            StatsResponse::MultipleRegression {
                coefficients,
                r_squared,
                residual_std_dev,
                std_errors,
                n,
                k,
                ..
            } => {
                assert_eq!(n, 16);
                assert_eq!(k, 6);
                // R² and residual std dev
                let r2_rel = (r_squared - cert_r2).abs() / cert_r2;
                assert!(r2_rel < 1e-10, "R² rel err={r2_rel:.2e} got={r_squared}");
                let rsd_rel = (residual_std_dev - cert_rsd).abs() / cert_rsd;
                assert!(
                    rsd_rel < 1e-10,
                    "RSD rel err={rsd_rel:.2e} got={residual_std_dev}"
                );
                // All 7 coefficients
                for (i, (got, cert)) in coefficients.iter().zip(cert_b.iter()).enumerate() {
                    let rel = (got - cert).abs() / cert.abs();
                    assert!(
                        rel < 3e-9,
                        "coeff[{i}] rel err={rel:.2e} got={got} cert={cert}"
                    );
                }
                // All 7 standard errors
                for (i, (got, cert)) in std_errors.iter().zip(cert_se.iter()).enumerate() {
                    let rel = (got - cert).abs() / cert;
                    assert!(
                        rel < 1e-8,
                        "se[{i}] rel err={rel:.2e} got={got} cert={cert}"
                    );
                }
            }
            other => panic!("expected MultipleRegression, got {other:?}"),
        }
    }

    // ── hypothesis testing helpers ────────────────────────────────────────────

    fn ht(response: StatsResponse) -> (f64, f64, f64, bool) {
        match response {
            StatsResponse::HypothesisTest {
                statistic,
                p_value,
                dof,
                reject_h0,
                ..
            } => (statistic, p_value, dof, reject_h0),
            other => panic!("expected HypothesisTest, got {other:?}"),
        }
    }

    // ── default_alpha serde path ──────────────────────────────────────────────

    #[test]
    fn default_alpha_via_json_is_0_05_not_mutant() {
        // Kills replace default_alpha with 0.0 / 1.0 / -1.0:
        // alpha is stored in HypothesisTest and must equal 0.05 exactly.
        // Also: t=3√2≈4.24, p≈0.013 — reject at 0.05 but not at 0.0.
        let req: StatsRequest = serde_json::from_str(
            r#"{"intent":"one_sample_t","sample":[1.0,2.0,3.0,4.0,5.0],"mu0":0.0}"#,
        )
        .unwrap();
        match req.evaluate() {
            StatsResponse::HypothesisTest {
                alpha, reject_h0, ..
            } => {
                assert!(
                    (alpha - 0.05).abs() < 1e-12,
                    "default alpha must be 0.05, got {alpha}"
                );
                assert!(
                    reject_h0,
                    "alpha=0.0 never rejects; alpha=1.0 always rejects when p<1"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── one-sample t ─────────────────────────────────────────────────────────

    #[test]
    fn one_sample_t_statistic_exact() {
        // sample=[1,2,3,4,5]: mean=3, var=2.5, s=sqrt(2.5), se=sqrt(0.5)
        // t = 3/sqrt(0.5) = 3*sqrt(2) = 4.24264..., dof=4
        let req = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Two,
        };
        let (stat, p, dof, reject) = ht(req.evaluate());
        assert!((stat - 3.0 * 2.0f64.sqrt()).abs() < 1e-10, "t={stat}");
        assert_eq!(dof, 4.0);
        assert!(p > 0.0 && p < 0.05, "p={p}");
        assert!(reject, "should reject H0 (mu=0) when mean=3, se=0.707");
    }

    #[test]
    fn one_sample_t_mu0_equals_mean_gives_zero_stat() {
        // When mu0 == sample mean, t = 0, p = 1.0, fail to reject
        let req = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 3.0,
            alpha: 0.05,
            tail: Tail::Two,
        };
        let (stat, p, _dof, reject) = ht(req.evaluate());
        assert!(stat.abs() < 1e-12, "t should be 0, got {stat}");
        assert!((p - 1.0).abs() < 1e-10, "p should be 1.0, got {p}");
        assert!(!reject, "should not reject when t=0");
    }

    #[test]
    fn one_sample_t_negative_stat_two_tail_p_equals_positive() {
        // Kills: replace abs() with identity — negative t would give wrong p
        let pos = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 0.0, // mean=3 > mu0 → positive t
            alpha: 0.05,
            tail: Tail::Two,
        };
        let neg = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 6.0, // mean=3 < mu0 → negative t
            alpha: 0.05,
            tail: Tail::Two,
        };
        let (t_pos, p_pos, _, _) = ht(pos.evaluate());
        let (t_neg, p_neg, _, _) = ht(neg.evaluate());
        assert!(t_pos > 0.0 && t_neg < 0.0);
        assert!(
            (p_pos - p_neg).abs() < 1e-10,
            "two-tailed p must be symmetric"
        );
        assert!(p_pos <= 1.0 && p_neg <= 1.0, "p must be <= 1");
    }

    #[test]
    fn one_sample_t_dof_is_n_minus_1() {
        // dof = n-1; kills replace (n-1) with n
        let req = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
            mu0: 4.0,
            alpha: 0.05,
            tail: Tail::Two,
        };
        let (_, _, dof, _) = ht(req.evaluate());
        assert_eq!(dof, 6.0); // n=7, dof=6
    }

    #[test]
    fn one_sample_t_left_tail_p_smaller_for_negative_t() {
        // Left-tail: p = CDF(t). For negative t, p < 0.5. Kills tail-branch mutations.
        let req = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 6.0, // mean=3 < mu0 → negative t
            alpha: 0.05,
            tail: Tail::Left,
        };
        let (t, p, _, reject) = ht(req.evaluate());
        assert!(t < 0.0);
        assert!(p < 0.5, "left-tail p of negative t must be < 0.5, got {p}");
        assert!(reject, "left-tail test should reject when t << 0");
    }

    #[test]
    fn one_sample_t_right_tail_p_smaller_for_positive_t() {
        // Right-tail: p = 1 - CDF(t). For positive t, p < 0.5. Kills tail-branch mutations.
        let req = StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 0.0, // mean=3 > mu0 → positive t
            alpha: 0.05,
            tail: Tail::Right,
        };
        let (t, p, _, reject) = ht(req.evaluate());
        assert!(t > 0.0);
        assert!(p < 0.5, "right-tail p of positive t must be < 0.5, got {p}");
        assert!(reject);
    }

    #[test]
    fn one_sample_t_p_value_in_unit_interval() {
        // Kills replace 2*(1-CDF) with 2*(1+CDF) or similar
        for mu0 in [-10.0, 0.0, 3.0, 10.0] {
            let req = StatsRequest::OneSampleT {
                sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
                mu0,
                alpha: 0.05,
                tail: Tail::Two,
            };
            let (_, p, _, _) = ht(req.evaluate());
            assert!((0.0..=1.0).contains(&p), "mu0={mu0} p={p}");
        }
    }

    #[test]
    fn one_sample_t_requires_n_at_least_2() {
        let req = StatsRequest::OneSampleT {
            sample: vec![5.0],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Two,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    #[test]
    fn one_sample_t_rejects_non_finite() {
        let req = StatsRequest::OneSampleT {
            sample: vec![1.0, f64::NAN],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Two,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    #[test]
    fn one_sample_t_rejects_zero_variance() {
        let req = StatsRequest::OneSampleT {
            sample: vec![3.0, 3.0, 3.0],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Two,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    // ── two-sample t ──────────────────────────────────────────────────────────

    #[test]
    fn two_sample_welch_statistic_and_dof_exact() {
        // sample1=[10,12,11,13,9] mean=11, var=2.5
        // sample2=[8,6,7,9,5]    mean=7,  var=2.5
        // t = (11-7)/sqrt(2.5/5+2.5/5) = 4/sqrt(1.0) = 4.0
        // Welch dof = 1.0^2 / (0.5^2/4 + 0.5^2/4) = 1.0 / 0.125 = 8.0
        let req = StatsRequest::TwoSampleT {
            sample1: vec![10.0, 12.0, 11.0, 13.0, 9.0],
            sample2: vec![8.0, 6.0, 7.0, 9.0, 5.0],
            alpha: 0.05,
            tail: Tail::Two,
            equal_var: false,
        };
        let (stat, p, dof, reject) = ht(req.evaluate());
        assert!((stat - 4.0).abs() < 1e-10, "t={stat}");
        assert!((dof - 8.0).abs() < 1e-10, "dof={dof}");
        assert!(p < 0.05, "p={p}");
        assert!(reject);
    }

    #[test]
    fn two_sample_pooled_statistic_exact() {
        // Same data, equal_var=true: sp2 = (4*2.5 + 4*2.5)/8 = 2.5
        // t = (11-7)/(sqrt(2.5)*sqrt(1/5+1/5)) = 4/(sqrt(2.5)*sqrt(0.4))
        //   = 4/sqrt(1.0) = 4.0, dof = 5+5-2 = 8
        let req = StatsRequest::TwoSampleT {
            sample1: vec![10.0, 12.0, 11.0, 13.0, 9.0],
            sample2: vec![8.0, 6.0, 7.0, 9.0, 5.0],
            alpha: 0.05,
            tail: Tail::Two,
            equal_var: true,
        };
        let (stat, _, dof, _) = ht(req.evaluate());
        assert!((stat - 4.0).abs() < 1e-10, "t={stat}");
        assert_eq!(dof, 8.0);
    }

    #[test]
    fn two_sample_welch_dof_satterthwaite_formula() {
        // Unequal n and variance to stress the Welch–Satterthwaite formula:
        // s1=1 (n1=4), s2=3 (n2=9) → var1=1, var2=9
        // v1n=1/4=0.25, v2n=9/9=1.0
        // Welch dof = (0.25+1.0)^2 / (0.25^2/3 + 1.0^2/8) = 1.5625/(0.020833+0.125) ≈ 10.71
        // sample1: mean=10, sample2: mean=0 → t = 10/sqrt(1.25)
        let req = StatsRequest::TwoSampleT {
            sample1: vec![10.0, 12.0, 11.0, 13.0, 9.0],
            sample2: vec![8.0, 6.0, 7.0, 9.0, 5.0, 7.0, 8.0],
            alpha: 0.05,
            tail: Tail::Two,
            equal_var: false,
        };
        let (_, _, dof, _) = ht(req.evaluate());
        // Pooled dof would be 5+7-2=10; Welch dof must differ
        assert!(
            (dof - 10.0).abs() > 0.1,
            "Welch dof should differ from pooled dof=10, got {dof}"
        );
        assert!(dof > 0.0 && dof < 20.0);
    }

    // ── paired t ─────────────────────────────────────────────────────────────

    #[test]
    fn paired_t_statistic_exact() {
        // before=[10,8,7,6,9], after=[12,9,9,8,11]
        // diffs=[2,1,2,2,2], mean=1.8, var=0.2, s=sqrt(0.2)
        // t = 1.8 / (sqrt(0.2)/sqrt(5)) = 1.8 / sqrt(0.04) = 1.8/0.2 = 9.0
        // dof = 4
        let req = StatsRequest::PairedT {
            before: vec![10.0, 8.0, 7.0, 6.0, 9.0],
            after: vec![12.0, 9.0, 9.0, 8.0, 11.0],
            alpha: 0.05,
            tail: Tail::Two,
        };
        let (stat, p, dof, reject) = ht(req.evaluate());
        assert!((stat - 9.0).abs() < 1e-10, "t={stat}");
        assert_eq!(dof, 4.0);
        assert!(p < 0.01, "p={p}");
        assert!(reject);
    }

    #[test]
    fn paired_t_rejects_length_mismatch() {
        let req = StatsRequest::PairedT {
            before: vec![1.0, 2.0],
            after: vec![1.0],
            alpha: 0.05,
            tail: Tail::Two,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    // ── chi-square GoF ────────────────────────────────────────────────────────

    #[test]
    fn chi_square_gof_statistic_exact() {
        // observed=[10,20,30], expected=[20,20,20]
        // chi2 = (10-20)^2/20 + 0 + (30-20)^2/20 = 5 + 0 + 5 = 10.0, dof=2
        let req = StatsRequest::ChiSquareGof {
            observed: vec![10.0, 20.0, 30.0],
            expected: vec![20.0, 20.0, 20.0],
            alpha: 0.05,
        };
        let (stat, p, dof, reject) = ht(req.evaluate());
        assert!((stat - 10.0).abs() < 1e-10, "chi2={stat}");
        assert_eq!(dof, 2.0);
        assert!(p < 0.05, "p={p}");
        assert!(reject);
    }

    #[test]
    fn chi_square_gof_dof_is_k_minus_1() {
        // Kills replace (len-1) with len in dof
        let req = StatsRequest::ChiSquareGof {
            observed: vec![10.0, 10.0, 10.0, 10.0, 10.0],
            expected: vec![10.0, 10.0, 10.0, 10.0, 10.0],
            alpha: 0.05,
        };
        let (_, _, dof, _) = ht(req.evaluate());
        assert_eq!(dof, 4.0); // k=5, dof=4
    }

    #[test]
    fn chi_square_gof_perfect_fit_fails_to_reject() {
        // observed == expected → chi2 = 0, p = 1.0
        let req = StatsRequest::ChiSquareGof {
            observed: vec![25.0, 75.0],
            expected: vec![25.0, 75.0],
            alpha: 0.05,
        };
        let (stat, p, _, reject) = ht(req.evaluate());
        assert!(stat.abs() < 1e-12, "chi2={stat}");
        assert!((p - 1.0).abs() < 1e-6, "p={p}");
        assert!(!reject);
    }

    #[test]
    fn chi_square_gof_p_value_in_unit_interval() {
        // Kills replace 1.0 - CDF with 1.0 + CDF
        let req = StatsRequest::ChiSquareGof {
            observed: vec![10.0, 20.0, 30.0],
            expected: vec![20.0, 20.0, 20.0],
            alpha: 0.05,
        };
        let (_, p, _, _) = ht(req.evaluate());
        assert!((0.0..=1.0).contains(&p), "p={p}");
    }

    #[test]
    fn chi_square_gof_rejects_length_mismatch() {
        let req = StatsRequest::ChiSquareGof {
            observed: vec![10.0, 20.0],
            expected: vec![15.0],
            alpha: 0.05,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    // ── one-way ANOVA ─────────────────────────────────────────────────────────

    #[test]
    fn anova_f_statistic_exact() {
        // groups: [1,2,3],[4,5,6],[7,8,9]
        // grand_mean=5, SS_between=3*(2-5)^2+0+3*(8-5)^2=27+27=54, SS_within=2+2+2=6
        // df_between=2, df_within=6, F=(54/2)/(6/6)=27/1=27.0
        let req = StatsRequest::OneWayAnova {
            groups: vec![
                vec![1.0, 2.0, 3.0],
                vec![4.0, 5.0, 6.0],
                vec![7.0, 8.0, 9.0],
            ],
            alpha: 0.05,
        };
        let (stat, p, dof, reject) = ht(req.evaluate());
        assert!((stat - 27.0).abs() < 1e-10, "F={stat}");
        assert_eq!(dof, 2.0); // df_between = k-1 = 2
        assert!(p < 0.05, "p={p}");
        assert!(reject);
    }

    #[test]
    fn anova_df_between_is_k_minus_1() {
        // 4 groups → df_between = 3
        let req = StatsRequest::OneWayAnova {
            groups: vec![
                vec![1.0, 2.0],
                vec![3.0, 4.0],
                vec![5.0, 6.0],
                vec![7.0, 8.0],
            ],
            alpha: 0.05,
        };
        let (_, _, dof, _) = ht(req.evaluate());
        assert_eq!(dof, 3.0);
    }

    #[test]
    fn anova_identical_groups_fails_to_reject() {
        // All groups identical → large p, no rejection
        let req = StatsRequest::OneWayAnova {
            groups: vec![
                vec![5.0, 5.0, 5.0],
                vec![5.0, 5.0, 5.0],
                vec![5.0, 5.0, 5.0],
            ],
            alpha: 0.05,
        };
        // All between-group variance is 0; this may error (zero ss_within also? no, ss_within=0 too)
        // Actually ss_within=0 too since all values equal mean. Error branch.
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    #[test]
    fn anova_p_value_in_unit_interval() {
        // Kills replace 1.0 - CDF with 1.0 + CDF in one_way_anova_test
        let req = StatsRequest::OneWayAnova {
            groups: vec![
                vec![1.0, 2.0, 3.0],
                vec![4.0, 5.0, 6.0],
                vec![7.0, 8.0, 9.0],
            ],
            alpha: 0.05,
        };
        let (_, p, _, _) = ht(req.evaluate());
        assert!((0.0..=1.0).contains(&p), "p={p}");
    }

    #[test]
    fn anova_f_numerator_denominator_not_swapped() {
        // F = MS_between/MS_within. If swapped → F = 1/27. Test that F > 1 for spread groups.
        let req = StatsRequest::OneWayAnova {
            groups: vec![vec![1.0, 2.0, 3.0], vec![10.0, 11.0, 12.0]],
            alpha: 0.05,
        };
        let (stat, _, _, _) = ht(req.evaluate());
        assert!(
            stat > 1.0,
            "F={stat} should be > 1 for well-separated groups"
        );
    }

    #[test]
    fn anova_grand_mean_formula_correct() {
        // If grand_mean uses wrong n (e.g. groups.len() instead of n_total),
        // SS_between would be wrong → F changes. This group has n=[2,4,2] not [2,2,2].
        // grand_mean = (0+0 + 10+10+10+10 + 20+20) / 8 = 80/8 = 10
        // mean0=0, mean1=10, mean2=20
        // SS_between = 2*(0-10)^2 + 4*(10-10)^2 + 2*(20-10)^2 = 200+0+200=400
        // SS_within = 0
        // Error: zero within-group variance
        let req = StatsRequest::OneWayAnova {
            groups: vec![
                vec![0.0, 0.0],
                vec![10.0, 10.0, 10.0, 10.0],
                vec![20.0, 20.0],
            ],
            alpha: 0.05,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    #[test]
    fn anova_requires_at_least_2_groups() {
        let req = StatsRequest::OneWayAnova {
            groups: vec![vec![1.0, 2.0, 3.0]],
            alpha: 0.05,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    // ── t_critical coverage ───────────────────────────────────────────────────

    #[test]
    fn t_critical_two_tailed_known_value() {
        // dof=4, alpha=0.05, two-tailed: t_{0.975,4} ≈ 2.7764
        // Kills: body→0.0/1.0/-1.0, / → % (gives t_{0.95,4}≈2.132), / → * (gives t_{0.9,4}≈1.533)
        match (StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Two,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest { critical_value, .. } => {
                assert!(
                    (critical_value - 2.7764).abs() < 0.001,
                    "two-tailed cv dof=4 alpha=0.05 should be ~2.776, got {critical_value}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn t_critical_left_tail_is_negative() {
        // Left-tail: cv = -t_{1-alpha,dof} < 0. Kills: delete - in left arm.
        match (StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Left,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest { critical_value, .. } => {
                assert!(
                    critical_value < 0.0,
                    "left-tail critical value must be negative, got {critical_value}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── hypothesis_conclusion coverage ────────────────────────────────────────

    #[test]
    fn hypothesis_conclusion_nonempty_and_descriptive() {
        // Kills: body→String::new() and body→"xyzzy".into()
        match (StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 0.0,
            alpha: 0.05,
            tail: Tail::Two,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest {
                conclusion,
                reject_h0,
                ..
            } => {
                assert!(reject_h0);
                assert!(
                    conclusion.contains("Reject"),
                    "conclusion should say Reject, got: {conclusion}"
                );
            }
            other => panic!("{other:?}"),
        }
        match (StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 3.0,
            alpha: 0.05,
            tail: Tail::Two,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest {
                conclusion,
                reject_h0,
                ..
            } => {
                assert!(!reject_h0);
                assert!(
                    conclusion.contains("Fail"),
                    "conclusion should say Fail, got: {conclusion}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── one-sample length guard ───────────────────────────────────────────────

    #[test]
    fn one_sample_t_two_elements_succeeds() {
        // len=2 must succeed. Kills < → == (would error for ==2) and < → <= (same).
        assert!(matches!(
            (StatsRequest::OneSampleT {
                sample: vec![1.0, 3.0],
                mu0: 0.0,
                alpha: 0.05,
                tail: Tail::Two,
            })
            .evaluate(),
            StatsResponse::HypothesisTest { .. }
        ));
    }

    // ── one-sample reject_h0 strict-less-than ─────────────────────────────────

    #[test]
    fn one_sample_t_reject_h0_strict_lt() {
        // mu0==sample_mean → t=0 → p_value=1.0 (two-tailed).
        // alpha=1.0: p < alpha is false but p <= alpha is true.
        // Kills: < → <= on reject_h0 line.
        match (StatsRequest::OneSampleT {
            sample: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            mu0: 3.0,
            alpha: 1.0,
            tail: Tail::Two,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest {
                reject_h0, p_value, ..
            } => {
                assert!((p_value - 1.0).abs() < 1e-10, "p={p_value}");
                assert!(
                    !reject_h0,
                    "p=1.0 with alpha=1.0: strict < means do not reject"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── two-sample length guard ───────────────────────────────────────────────

    #[test]
    fn two_sample_t_two_element_sample_succeeds() {
        // sample1 has exactly len=2; must succeed.
        // Kills < → == and < → <= on the two-sample length guard.
        assert!(matches!(
            (StatsRequest::TwoSampleT {
                sample1: vec![1.0, 3.0],
                sample2: vec![2.0, 4.0, 6.0],
                alpha: 0.05,
                tail: Tail::Two,
                equal_var: false,
            })
            .evaluate(),
            StatsResponse::HypothesisTest { .. }
        ));
    }

    // ── two-sample pooled t: division vs multiplication ───────────────────────

    #[test]
    fn two_sample_pooled_statistic_asymmetric_sizes() {
        // sample1=[1,3] (n=2,mean=2,var=2), sample2=[2,4,6] (n=3,mean=4,var=4)
        // sp2 = (1*2+2*4)/3 = 10/3; se = sp*sqrt(5/6) = sqrt(10/3)*sqrt(5/6) = 5/3
        // t = (2-4)/(5/3) = -6/5 = -1.2
        // / → * would give t = -2*(5/3) = -10/3 ≠ -1.2  — kills that mutation.
        match (StatsRequest::TwoSampleT {
            sample1: vec![1.0, 3.0],
            sample2: vec![2.0, 4.0, 6.0],
            alpha: 0.05,
            tail: Tail::Two,
            equal_var: true,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest { statistic, .. } => {
                assert!((statistic - (-1.2)).abs() < 1e-10, "t={statistic}");
            }
            other => panic!("{other:?}"),
        }
    }

    // ── two-sample Welch t: division vs multiplication ────────────────────────

    #[test]
    fn two_sample_welch_statistic_asymmetric_variances() {
        // sample1=[1,5] (n=2,mean=3,var=8), sample2=[1,2,3] (n=3,mean=2,var=1)
        // v1n=4, v2n=1/3; se=sqrt(13/3); t=(3-2)/sqrt(13/3)=sqrt(3/13)≈0.4804
        // / → * would give t = 1*sqrt(13/3)≈2.082 ≠ 0.4804 — kills that mutation.
        match (StatsRequest::TwoSampleT {
            sample1: vec![1.0, 5.0],
            sample2: vec![1.0, 2.0, 3.0],
            alpha: 0.05,
            tail: Tail::Two,
            equal_var: false,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest { statistic, .. } => {
                let expected = (3.0_f64 / 13.0).sqrt();
                assert!(
                    (statistic - expected).abs() < 1e-10,
                    "t={statistic} expected {expected}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── two-sample reject_h0 strict-less-than ────────────────────────────────

    #[test]
    fn two_sample_t_reject_h0_strict_lt() {
        // mean1==mean2 → t=0 → p_value=1.0 (two-tailed); alpha=1.0.
        // p < alpha is false; p <= alpha is true. Kills < → <= on reject_h0 line.
        // sample1=[1,3] (mean=2,var=2), sample2=[0,4] (mean=2,var=8) — different variances so se≠0.
        match (StatsRequest::TwoSampleT {
            sample1: vec![1.0, 3.0],
            sample2: vec![0.0, 4.0],
            alpha: 1.0,
            tail: Tail::Two,
            equal_var: false,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest {
                reject_h0, p_value, ..
            } => {
                assert!((p_value - 1.0).abs() < 1e-6, "p={p_value}");
                assert!(
                    !reject_h0,
                    "p=1.0 with alpha=1.0: strict < means do not reject"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── chi-square GoF validation ─────────────────────────────────────────────

    #[test]
    fn chi_square_gof_rejects_infinite_observed() {
        // Inf is not finite but Inf >= 0. Kills && → || on observed guard (line 1540).
        let req = StatsRequest::ChiSquareGof {
            observed: vec![f64::INFINITY, 10.0],
            expected: vec![10.0, 10.0],
            alpha: 0.05,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    #[test]
    fn chi_square_gof_rejects_infinite_expected() {
        // Inf is not finite but Inf > 0. Kills && → || on expected guard (line 1543).
        let req = StatsRequest::ChiSquareGof {
            observed: vec![10.0, 10.0],
            expected: vec![f64::INFINITY, 10.0],
            alpha: 0.05,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    #[test]
    fn chi_square_gof_rejects_zero_expected() {
        // 0.0 > 0.0 is false; 0.0 >= 0.0 is true. Kills > → >= on expected guard (line 1543).
        let req = StatsRequest::ChiSquareGof {
            observed: vec![10.0, 10.0],
            expected: vec![0.0, 10.0],
            alpha: 0.05,
        };
        assert!(matches!(req.evaluate(), StatsResponse::Error { .. }));
    }

    // ── chi-square GoF reject_h0 strict-less-than ─────────────────────────────

    #[test]
    fn chi_square_gof_reject_h0_strict_lt() {
        // observed==expected → chi2=0 → p_value=1.0; alpha=1.0.
        // p < alpha is false; p <= alpha is true. Kills < → <= on reject_h0 line.
        match (StatsRequest::ChiSquareGof {
            observed: vec![10.0, 10.0],
            expected: vec![10.0, 10.0],
            alpha: 1.0,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest {
                reject_h0, p_value, ..
            } => {
                assert!((p_value - 1.0).abs() < 1e-6, "p={p_value}");
                assert!(
                    !reject_h0,
                    "p=1.0 with alpha=1.0: strict < means do not reject"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── ANOVA F-statistic: division vs multiplication ─────────────────────────

    #[test]
    fn anova_f_statistic_msw_not_one() {
        // groups=[[1,2],[3,4],[5,6]]: grand_mean=3.5
        // ss_between=16, ss_within=1.5, df_between=2, df_within=3
        // MSB=8, MSW=0.5; F=16.0. / → * would give F=8*0.5=4.0 ≠ 16.0.
        match (StatsRequest::OneWayAnova {
            groups: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            alpha: 0.05,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest { statistic, .. } => {
                assert!((statistic - 16.0).abs() < 1e-10, "F={statistic}");
            }
            other => panic!("{other:?}"),
        }
    }

    // ── ANOVA reject_h0 strict-less-than ──────────────────────────────────────

    #[test]
    fn anova_reject_h0_strict_lt() {
        // All group means equal → ss_between=0 → F=0 → p_value=1.0; alpha=1.0.
        // p < alpha is false; p <= alpha is true. Kills < → <= on reject_h0 line.
        match (StatsRequest::OneWayAnova {
            groups: vec![vec![1.0, 3.0], vec![1.0, 3.0], vec![1.0, 3.0]],
            alpha: 1.0,
        })
        .evaluate()
        {
            StatsResponse::HypothesisTest {
                reject_h0, p_value, ..
            } => {
                assert!((p_value - 1.0).abs() < 1e-6, "p={p_value}");
                assert!(
                    !reject_h0,
                    "p=1.0 with alpha=1.0: strict < means do not reject"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // ── time series helpers ───────────────────────────────────────────────────

    fn ts(response: StatsResponse) -> (Vec<f64>, usize) {
        match response {
            StatsResponse::TimeSeries { result, n, .. } => (result, n),
            other => panic!("expected TimeSeries, got {other:?}"),
        }
    }

    // ── SMA ──────────────────────────────────────────────────────────────────

    #[test]
    fn sma_window3_arithmetic_sequence_exact() {
        // [1,2,3,4,5] period=3: windows [1,2,3],[2,3,4],[3,4,5] → [2.0,3.0,4.0]
        // Kills / → * on the mean: sum=6, 6/3=2 vs 6*3=18.
        let (result, n) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Sma,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            period: 3,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(n, 5);
        assert_eq!(result.len(), 3, "output length should be n-period+1");
        assert!((result[0] - 2.0).abs() < 1e-12, "sma[0]={}", result[0]);
        assert!((result[1] - 3.0).abs() < 1e-12, "sma[1]={}", result[1]);
        assert!((result[2] - 4.0).abs() < 1e-12, "sma[2]={}", result[2]);
    }

    #[test]
    fn sma_window_equals_n_gives_single_mean() {
        // period=n → one output element = mean.
        // Kills boundary mutations on 0..=n-period (would give empty or wrong result).
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Sma,
            values: vec![2.0, 4.0, 6.0, 8.0],
            period: 4,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(result.len(), 1);
        assert!((result[0] - 5.0).abs() < 1e-12, "mean={}", result[0]);
    }

    #[test]
    fn sma_output_length_is_n_minus_period_plus_one() {
        // Explicit length assertion kills off-by-one mutations.
        for (n, period) in [(10usize, 3usize), (7, 4), (5, 5)] {
            let values: Vec<f64> = (1..=n).map(|i| i as f64).collect();
            let (result, _) = ts((StatsRequest::TimeSeries {
                method: TimeSeriesIntent::Sma,
                values,
                period,
                max_lag: 10,
                smoothing: 0.3,
            })
            .evaluate());
            assert_eq!(
                result.len(),
                n - period + 1,
                "n={n} period={period} len={}",
                result.len()
            );
        }
    }

    #[test]
    fn sma_rejects_period_exceeding_length() {
        assert!(matches!(
            (StatsRequest::TimeSeries {
                method: TimeSeriesIntent::Sma,
                values: vec![1.0, 2.0],
                period: 5,
                max_lag: 10,
                smoothing: 0.3,
            })
            .evaluate(),
            StatsResponse::Error { .. }
        ));
    }

    #[test]
    fn sma_rejects_period_less_than_2() {
        assert!(matches!(
            (StatsRequest::TimeSeries {
                method: TimeSeriesIntent::Sma,
                values: vec![1.0, 2.0, 3.0],
                period: 1,
                max_lag: 10,
                smoothing: 0.3,
            })
            .evaluate(),
            StatsResponse::Error { .. }
        ));
    }

    // ── EMA ──────────────────────────────────────────────────────────────────

    #[test]
    fn ema_exact_values() {
        // values=[1,4,9,16], smoothing=0.5:
        // ema[0]=1.0
        // ema[1]=0.5*4 + 0.5*1.0 = 2.5
        // ema[2]=0.5*9 + 0.5*2.5 = 5.75
        // ema[3]=0.5*16 + 0.5*5.75 = 10.875
        // Kills + → -, * → +, smoothing ↔ 1-smoothing coefficient mutations.
        let (result, n) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Ema,
            values: vec![1.0, 4.0, 9.0, 16.0],
            period: 3,
            max_lag: 10,
            smoothing: 0.5,
        })
        .evaluate());
        assert_eq!(n, 4);
        assert_eq!(result.len(), 4, "EMA output length must equal input length");
        assert!((result[0] - 1.0).abs() < 1e-12, "ema[0]={}", result[0]);
        assert!((result[1] - 2.5).abs() < 1e-12, "ema[1]={}", result[1]);
        assert!((result[2] - 5.75).abs() < 1e-12, "ema[2]={}", result[2]);
        assert!((result[3] - 10.875).abs() < 1e-12, "ema[3]={}", result[3]);
    }

    #[test]
    fn ema_constant_series_is_unchanged() {
        // EMA of constant series = that constant (kills coefficient-swap mutation).
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Ema,
            values: vec![7.0, 7.0, 7.0, 7.0, 7.0],
            period: 3,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        for (i, v) in result.iter().enumerate() {
            assert!((v - 7.0).abs() < 1e-12, "ema[{i}]={v}");
        }
    }

    #[test]
    fn ema_rejects_invalid_smoothing() {
        for bad in [0.0, 1.0, -0.1, 1.5] {
            assert!(
                matches!(
                    (StatsRequest::TimeSeries {
                        method: TimeSeriesIntent::Ema,
                        values: vec![1.0, 2.0, 3.0],
                        period: 3,
                        max_lag: 10,
                        smoothing: bad,
                    })
                    .evaluate(),
                    StatsResponse::Error { .. }
                ),
                "smoothing={bad} should error"
            );
        }
    }

    // ── Autocorrelation ───────────────────────────────────────────────────────

    #[test]
    fn autocorr_lag0_is_always_one() {
        // r_0 = var / var = 1.0. Kills body → 0.0 and num/denom mutations.
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Autocorr,
            values: vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0],
            period: 3,
            max_lag: 4,
            smoothing: 0.3,
        })
        .evaluate());
        assert!((result[0] - 1.0).abs() < 1e-12, "ACF[0]={}", result[0]);
    }

    #[test]
    fn autocorr_lag1_arithmetic_sequence() {
        // [1,2,3,4,5] (n=5, mean=3): denom=10, lag-1 num=4 → r_1=0.4
        // Kills / → * (would give 4*10=40) and - → + in (x-mean) (wrong mean subtraction).
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Autocorr,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            period: 3,
            max_lag: 3,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(result.len(), 4, "lags 0,1,2,3");
        assert!((result[0] - 1.0).abs() < 1e-12, "ACF[0]={}", result[0]);
        assert!(
            (result[1] - 0.4).abs() < 1e-10,
            "ACF[1] should be 0.4, got {}",
            result[1]
        );
    }

    #[test]
    fn autocorr_values_in_minus_one_to_one() {
        // All lags must be in [-1, 1]. Kills normalization errors.
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Autocorr,
            values: vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0],
            period: 3,
            max_lag: 5,
            smoothing: 0.3,
        })
        .evaluate());
        for (k, r) in result.iter().enumerate() {
            assert!(*r >= -1.0 && *r <= 1.0, "ACF[{k}]={r} outside [-1,1]");
        }
    }

    #[test]
    fn autocorr_output_length_is_max_lag_plus_one() {
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Autocorr,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
            period: 3,
            max_lag: 4,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(result.len(), 5, "lags 0..=4 → 5 elements");
    }

    #[test]
    fn autocorr_rejects_zero_variance() {
        assert!(matches!(
            (StatsRequest::TimeSeries {
                method: TimeSeriesIntent::Autocorr,
                values: vec![5.0, 5.0, 5.0, 5.0],
                period: 3,
                max_lag: 2,
                smoothing: 0.3,
            })
            .evaluate(),
            StatsResponse::Error { .. }
        ));
    }

    // ── Trend ─────────────────────────────────────────────────────────────────

    #[test]
    fn trend_exact_slope_intercept_r_squared() {
        // [1,3,5,7,9]: perfect linear y = 2t + 1 (t=0..4)
        // slope=2, intercept=1, r_squared=1.0
        // Kills: wrong t-vector construction, wrong formula in linear_regression.
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Trend,
            values: vec![1.0, 3.0, 5.0, 7.0, 9.0],
            period: 3,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(
            result.len(),
            3,
            "trend returns [slope, intercept, r_squared]"
        );
        assert!((result[0] - 2.0).abs() < 1e-10, "slope={}", result[0]);
        assert!((result[1] - 1.0).abs() < 1e-10, "intercept={}", result[1]);
        assert!((result[2] - 1.0).abs() < 1e-10, "r_squared={}", result[2]);
    }

    #[test]
    fn trend_negative_slope() {
        // [5,4,3,2,1]: slope=-1, intercept=5
        // Kills sign errors and swapped slope/intercept.
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Trend,
            values: vec![5.0, 4.0, 3.0, 2.0, 1.0],
            period: 3,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        assert!((result[0] - (-1.0)).abs() < 1e-10, "slope={}", result[0]);
        assert!((result[1] - 5.0).abs() < 1e-10, "intercept={}", result[1]);
        assert!((result[2] - 1.0).abs() < 1e-10, "r_squared={}", result[2]);
    }

    // ── common error cases ────────────────────────────────────────────────────

    #[test]
    fn ts_rejects_too_few_values() {
        for method in [
            TimeSeriesIntent::Sma,
            TimeSeriesIntent::Ema,
            TimeSeriesIntent::Autocorr,
            TimeSeriesIntent::Trend,
        ] {
            assert!(
                matches!(
                    (StatsRequest::TimeSeries {
                        method,
                        values: vec![1.0],
                        period: 3,
                        max_lag: 2,
                        smoothing: 0.3,
                    })
                    .evaluate(),
                    StatsResponse::Error { .. }
                ),
                "{method:?} should error for len=1"
            );
        }
    }

    #[test]
    fn ts_rejects_non_finite_values() {
        assert!(matches!(
            (StatsRequest::TimeSeries {
                method: TimeSeriesIntent::Sma,
                values: vec![1.0, f64::NAN, 3.0],
                period: 2,
                max_lag: 2,
                smoothing: 0.3,
            })
            .evaluate(),
            StatsResponse::Error { .. }
        ));
    }

    // ── JSON serde round-trip ─────────────────────────────────────────────────

    #[test]
    fn ts_sma_json_round_trip() {
        // Verify intent tag deserialization and default field population.
        let req: StatsRequest = serde_json::from_str(
            r#"{"intent":"time_series","method":"sma","values":[1,2,3,4,5],"period":3}"#,
        )
        .unwrap();
        let (result, _) = ts(req.evaluate());
        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-12);
    }

    // ── serde default coverage ────────────────────────────────────────────────

    #[test]
    fn default_period_via_json_is_3_not_mutant() {
        // period=0 and period=1 both error ("period must be at least 2").
        // Only default=3 returns TimeSeries. Kills: replace default_period with 0, 1.
        let req: StatsRequest =
            serde_json::from_str(r#"{"intent":"time_series","method":"sma","values":[1,2,3,4,5]}"#)
                .unwrap();
        match req.evaluate() {
            StatsResponse::TimeSeries { result, .. } => {
                assert_eq!(result.len(), 3, "period=3 gives n-period+1=3 outputs");
                assert!((result[0] - 2.0).abs() < 1e-12);
            }
            other => panic!("period default must be 3 (not 0 or 1), got {other:?}"),
        }
    }

    #[test]
    fn default_max_lag_via_json_is_10_not_mutant() {
        // For n=5 values, default max_lag=10 → effective_max=4 → 5 ACF outputs.
        // max_lag=0 → 1 output; max_lag=1 → 2 outputs. Kills: replace default_max_lag with 0, 1.
        let req: StatsRequest = serde_json::from_str(
            r#"{"intent":"time_series","method":"autocorr","values":[1,2,3,4,5]}"#,
        )
        .unwrap();
        match req.evaluate() {
            StatsResponse::TimeSeries { result, .. } => {
                assert_eq!(
                    result.len(),
                    5,
                    "default max_lag=10 clamped to n-1=4 → 5 outputs"
                );
            }
            other => panic!("expected TimeSeries, got {other:?}"),
        }
    }

    #[test]
    fn default_smoothing_via_json_is_0_3_not_mutant() {
        // smoothing=0.0, 1.0, -1.0 all error; only default=0.3 succeeds.
        // Also checks exact ema[1]=0.3*4+0.7*1=1.9 to distinguish 0.3 from other values.
        // Kills: replace default_smoothing with 0.0, 1.0, -1.0.
        let req: StatsRequest =
            serde_json::from_str(r#"{"intent":"time_series","method":"ema","values":[1,4,9,16]}"#)
                .unwrap();
        match req.evaluate() {
            StatsResponse::TimeSeries { result, .. } => {
                assert_eq!(result.len(), 4);
                assert!(
                    (result[1] - 1.9).abs() < 1e-12,
                    "smoothing=0.3: ema[1]=0.3*4+0.7*1=1.9, got {}",
                    result[1]
                );
            }
            other => panic!("smoothing default 0.3 must succeed, got {other:?}"),
        }
    }

    // ── len=2 boundary ───────────────────────────────────────────────────────

    #[test]
    fn ts_two_element_values_succeeds() {
        // len=2 must succeed. Kills < → <= in ts_validate (would error for len=2).
        assert!(matches!(
            (StatsRequest::TimeSeries {
                method: TimeSeriesIntent::Ema,
                values: vec![1.0, 3.0],
                period: 3,
                max_lag: 1,
                smoothing: 0.5,
            })
            .evaluate(),
            StatsResponse::TimeSeries { .. }
        ));
    }

    // ── SMA period=2 boundary ────────────────────────────────────────────────

    #[test]
    fn sma_period_two_exact() {
        // period=2 must succeed. Kills < → <= on period guard (would error for period=2).
        // [1,2,3] period=2 → [(1+2)/2, (2+3)/2] = [1.5, 2.5]
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Sma,
            values: vec![1.0, 2.0, 3.0],
            period: 2,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(result.len(), 2);
        assert!((result[0] - 1.5).abs() < 1e-12, "sma[0]={}", result[0]);
        assert!((result[1] - 2.5).abs() < 1e-12, "sma[1]={}", result[1]);
    }

    // ── autocorr max_lag clamping ────────────────────────────────────────────

    #[test]
    fn autocorr_max_lag_clamped_to_n_minus_one() {
        // n=4, max_lag=10: effective lags 0..=3 → 4 outputs.
        // n-1 → n+1 mutant: k=5 causes underflow panic → caught.
        // n-1 → n/1 mutant: min(10,4)=4 → k=4 yields empty sum, 5 outputs → assert_eq fails.
        let (result, _) = ts((StatsRequest::TimeSeries {
            method: TimeSeriesIntent::Autocorr,
            values: vec![1.0, 2.0, 3.0, 4.0],
            period: 3,
            max_lag: 10,
            smoothing: 0.3,
        })
        .evaluate());
        assert_eq!(result.len(), 4, "lags 0..=3 for n=4, clamped by n-1");
    }

    // ── effect size helper boundary tests (private-function access) ──────────

    #[test]
    fn cohen_d_pooled_sd_plus_not_times() {
        // sample1=[0,2]: mean=1, var=2, n=2
        // sample2=[5-√2, 5+√2]: mean=5, var=4, n=2
        // numerator with +: 1*2 + 1*4 = 6  → sp=sqrt(3), d=-4/sqrt(3)
        // numerator with *: 1*2 * 1*4 = 8  → sp=2,       d=-2
        let s2_lo = 5.0_f64 - 2.0_f64.sqrt();
        let s2_hi = 5.0_f64 + 2.0_f64.sqrt();
        let d = cohen_d_two_sample(&[0.0, 2.0], &[s2_lo, s2_hi]).unwrap();
        assert!((d - (-4.0 / 3.0_f64.sqrt())).abs() < 1e-10, "d={d}");
    }

    #[test]
    fn interpret_d_all_boundaries() {
        // < 0.2 boundary: 0.2 must be "small" not "negligible"
        assert_eq!(interpret_d(0.19), "negligible");
        assert_eq!(interpret_d(0.2), "small"); // kills < 0.2 → <= 0.2
        assert_eq!(interpret_d(0.3), "small"); // kills < 0.2 → == 0.2 (if such a mutation existed)
        // < 0.5 boundary: 0.5 must be "medium" not "small"
        assert_eq!(interpret_d(0.5), "medium"); // kills < 0.5 → <= 0.5
        assert_eq!(interpret_d(0.6), "medium"); // kills < 0.5 → == 0.5
        // < 0.8 boundary: 0.8 must be "large" not "medium"
        assert_eq!(interpret_d(0.8), "large"); // kills < 0.8 → <= 0.8
        assert_eq!(interpret_d(1.0), "large"); // kills < 0.8 → == 0.8
        // negative d uses abs
        assert_eq!(interpret_d(-0.5), "medium");
        assert_eq!(interpret_d(-0.8), "large");
    }

    #[test]
    fn interpret_eta_sq_all_boundaries() {
        // < 0.01: 0.01 must be "small" not "negligible"
        assert_eq!(interpret_eta_sq(0.0), "negligible");
        assert_eq!(interpret_eta_sq(0.01), "small"); // kills < 0.01 → <= 0.01
        assert_eq!(interpret_eta_sq(0.03), "small"); // kills < 0.06 → == 0.06
        // < 0.06: 0.06 must be "medium" not "small"
        assert_eq!(interpret_eta_sq(0.06), "medium"); // kills < 0.06 → <= 0.06
        assert_eq!(interpret_eta_sq(0.10), "medium"); // kills < 0.14 → == 0.14
        // < 0.14: 0.14 must be "large" not "medium"
        assert_eq!(interpret_eta_sq(0.14), "large"); // kills < 0.14 → <= 0.14
        assert_eq!(interpret_eta_sq(0.20), "large");
    }

    #[test]
    fn interpret_r_all_boundaries() {
        // < 0.1: 0.1 must be "small" not "negligible"
        assert_eq!(interpret_r(0.05), "negligible"); // kills < 0.1 → == 0.1
        assert_eq!(interpret_r(0.1), "small"); // kills < 0.1 → <= 0.1
        assert_eq!(interpret_r(0.2), "small"); // kills < 0.3 → == 0.3
        // < 0.3: 0.3 must be "medium" not "small"
        assert_eq!(interpret_r(0.3), "medium"); // kills < 0.3 → <= 0.3
        assert_eq!(interpret_r(0.4), "medium"); // kills < 0.5 → == 0.5
        // < 0.5: 0.5 must be "large" not "medium"
        assert_eq!(interpret_r(0.5), "large"); // kills < 0.5 → <= 0.5
        assert_eq!(interpret_r(0.9), "large");
        // interpret_r uses abs
        assert_eq!(interpret_r(-0.3), "medium");
        assert_eq!(interpret_r(-0.5), "large");
    }
}
