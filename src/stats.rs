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
            {"$ref": "#/$defs/BinomialCdf"}
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
    Ok(SampleSummary {
        n,
        mean,
        variance,
        std_dev: variance.sqrt(),
        min,
        max,
    })
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
                ..
            } => {
                assert_eq!(n, 3);
                assert_eq!(mean, 2.0);
                assert_eq!(variance, 1.0);
                assert_eq!(std_dev, 1.0);
                assert_eq!(min, 1.0);
                assert_eq!(max, 3.0);
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
                | StatsResponse::Interval { checks, .. } => checks,
                other => panic!("expected successful response, got {other:?}"),
            };
            assert_eq!(checks.len(), 1);
            assert_eq!(checks[0].name, "computed_by_statrs_or_checked_adapter");
            assert!(checks[0].passed);
        }
    }
}
