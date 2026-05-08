use crate::{
    CONTRACT_VERSION,
    protocol::{ErrorCode, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use statrs::distribution::{Binomial, ContinuousCDF, Discrete, DiscreteCDF, Normal, StudentsT};

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
    Correlation {
        x: Vec<f64>,
        y: Vec<f64>,
    },
    LinearRegression {
        x: Vec<f64>,
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
            StatsRequest::Correlation { x, y } => {
                let (pearson_r, n) = correlation(x, y)?;
                Ok(StatsOutput::CorrelationResult { pearson_r, n })
            }
            StatsRequest::LinearRegression { x, y } => {
                let (slope, intercept, r_squared) = linear_regression(x, y)?;
                Ok(StatsOutput::RegressionResult {
                    slope,
                    intercept,
                    r_squared,
                })
            }
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
            {"$ref": "#/$defs/Correlation"},
            {"$ref": "#/$defs/LinearRegression"},
            {"$ref": "#/$defs/Percentile"},
            {"$ref": "#/$defs/Mode"},
            {"$ref": "#/$defs/Rank"}
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
                    "x": {"$ref": "#/$defs/Sample"},
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
    PercentileValue {
        value: f64,
        p: f64,
    },
    ModeResult {
        values: Vec<f64>,
        frequency: usize,
    },
    RanksResult(Vec<f64>),
}

fn describe_sample(values: &[f64]) -> Result<SampleSummary, String> {
    if values.is_empty() {
        return Err("sample must contain at least one value".to_owned());
    }
    if !values.iter().all(|v| v.is_finite()) {
        return Err("sample values must be finite".to_owned());
    }
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let mut sum_sq = 0.0;
    let mut min = values[0];
    let mut max = values[0];
    for value in values {
        let delta = value - mean;
        sum_sq += delta * delta;
        min = min.min(*value);
        max = max.max(*value);
    }
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
    let sum: f64 = values.iter().map(|v| ((v - mean) / std_dev).powi(3)).sum();
    factor * sum
}

fn sample_kurtosis(values: &[f64], mean: f64, std_dev: f64) -> f64 {
    let n = values.len() as f64;
    if n < 4.0 || std_dev == 0.0 {
        return 0.0;
    }
    let factor1 = (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0));
    let sum: f64 = values.iter().map(|v| ((v - mean) / std_dev).powi(4)).sum();
    let correction = 3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0));
    factor1 * sum - correction
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
    let mean_x = x.iter().sum::<f64>() / n as f64;
    let mean_y = y.iter().sum::<f64>() / n as f64;
    let (mut num, mut denom_x, mut denom_y) = (0.0f64, 0.0f64, 0.0f64);
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        num += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }
    let denom = (denom_x * denom_y).sqrt();
    if denom == 0.0 {
        return Err(
            "correlation is undefined when all values in a series are identical".to_owned(),
        );
    }
    Ok((num / denom, n))
}

fn linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64, f64), String> {
    let (pearson_r, n) = correlation(x, y)?;
    let mean_x = x.iter().sum::<f64>() / n as f64;
    let mean_y = y.iter().sum::<f64>() / n as f64;
    let denom_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
    let num: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();
    let slope = num / denom_x;
    let intercept = mean_y - slope * mean_x;
    let r_squared = pearson_r * pearson_r;
    Ok((slope, intercept, r_squared))
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

#[cfg(test)]
mod tests {
    use super::*;

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
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
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
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
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
                | StatsResponse::Percentile { checks, .. }
                | StatsResponse::Mode { checks, .. }
                | StatsResponse::Ranks { checks, .. } => checks,
                other => panic!("expected successful response, got {other:?}"),
            };
            assert_eq!(checks.len(), 1);
            assert_eq!(checks[0].name, "computed_by_statrs_or_checked_adapter");
            assert!(checks[0].passed);
        }
    }
}
