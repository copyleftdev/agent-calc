use crate::{
    CONTRACT_VERSION, Expr, Rational,
    protocol::{ErrorCode, ExactRational, classify_error},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum AssumptionsRequest {
    Validate {
        assumptions: Vec<Assumption>,
    },
    Bounds {
        assumptions: Vec<Assumption>,
        symbol: String,
    },
    Entails {
        assumptions: Vec<Assumption>,
        query: Assumption,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Assumption {
    Domain {
        symbol: String,
        domain: AssumptionDomain,
    },
    Compare {
        symbol: String,
        op: ComparisonOp,
        value: Expr,
    },
    Bounded {
        symbol: String,
        lower: Expr,
        upper: Expr,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssumptionDomain {
    Rational,
    Integer,
    Natural,
    Nonzero,
    Positive,
    Negative,
    Nonnegative,
    Nonpositive,
    UnitInterval,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOp {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum AssumptionsResponse {
    Context {
        contract_version: String,
        facts: Vec<DerivedFact>,
        checks: Vec<AssumptionCheck>,
    },
    Bounds {
        contract_version: String,
        symbol: String,
        lower: Option<Bound>,
        upper: Option<Bound>,
        excluded: Vec<ExactRational>,
        domains: Vec<AssumptionDomain>,
        checks: Vec<AssumptionCheck>,
    },
    Entailment {
        contract_version: String,
        entailed: bool,
        evidence: Vec<String>,
        checks: Vec<AssumptionCheck>,
    },
    Error {
        contract_version: String,
        code: ErrorCode,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bound {
    pub value: ExactRational,
    pub inclusive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DerivedFact {
    pub symbol: String,
    pub lower: Option<Bound>,
    pub upper: Option<Bound>,
    pub excluded: Vec<ExactRational>,
    pub domains: Vec<AssumptionDomain>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssumptionCheck {
    pub name: String,
    pub passed: bool,
}

impl AssumptionsRequest {
    pub fn evaluate(&self) -> AssumptionsResponse {
        match self.evaluate_inner() {
            Ok(output) => output,
            Err(reason) => AssumptionsResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                code: classify_error(&reason),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<AssumptionsResponse, String> {
        match self {
            AssumptionsRequest::Validate { assumptions } => {
                let context = Context::from_assumptions(assumptions)?;
                Ok(AssumptionsResponse::Context {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    facts: context.derived_facts(),
                    checks: default_checks(),
                })
            }
            AssumptionsRequest::Bounds {
                assumptions,
                symbol,
            } => {
                validate_symbol(symbol)?;
                let context = Context::from_assumptions(assumptions)?;
                let fact = context.fact_for(symbol);
                Ok(AssumptionsResponse::Bounds {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    symbol: symbol.clone(),
                    lower: fact.lower.map(bound),
                    upper: fact.upper.map(bound),
                    excluded: fact.excluded.iter().map(exact_rational).collect(),
                    domains: fact.domains.into_iter().collect(),
                    checks: default_checks(),
                })
            }
            AssumptionsRequest::Entails { assumptions, query } => {
                let context = Context::from_assumptions(assumptions)?;
                let entailment = context.entails(query)?;
                Ok(AssumptionsResponse::Entailment {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    entailed: entailment.entailed,
                    evidence: entailment.evidence,
                    checks: default_checks(),
                })
            }
        }
    }
}

pub fn assumptions_schema_json() -> Value {
    let expr_defs = crate::schema_json()["$defs"].clone();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/assumptions.json"),
        "title": "agent-calc calc1 assumptions request",
        "description": "Typed exact symbolic assumption context request for validation, bound derivation, and entailment.",
        "type": "object",
        "required": ["intent", "assumptions"],
        "oneOf": [
            {"$ref": "#/$defs/Validate"},
            {"$ref": "#/$defs/Bounds"},
            {"$ref": "#/$defs/Entails"}
        ],
        "$defs": {
            "Expr": expr_defs["Expr"].clone(),
            "Integer": expr_defs["Integer"].clone(),
            "Rational": expr_defs["Rational"].clone(),
            "Symbol": expr_defs["Symbol"].clone(),
            "Add": expr_defs["Add"].clone(),
            "Sub": expr_defs["Sub"].clone(),
            "Mul": expr_defs["Mul"].clone(),
            "Div": expr_defs["Div"].clone(),
            "Pow": expr_defs["Pow"].clone(),
            "Neg": expr_defs["Neg"].clone(),
            "Domain": {
                "enum": ["rational", "integer", "natural", "nonzero", "positive", "negative", "nonnegative", "nonpositive", "unit_interval"]
            },
            "ComparisonOp": {
                "enum": ["eq", "neq", "gt", "gte", "lt", "lte"]
            },
            "Assumption": {
                "oneOf": [
                    {"$ref": "#/$defs/DomainAssumption"},
                    {"$ref": "#/$defs/CompareAssumption"},
                    {"$ref": "#/$defs/BoundedAssumption"}
                ]
            },
            "DomainAssumption": {
                "type": "object",
                "required": ["kind", "symbol", "domain"],
                "additionalProperties": false,
                "properties": {
                    "kind": {"const": "domain"},
                    "symbol": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"},
                    "domain": {"$ref": "#/$defs/Domain"}
                }
            },
            "CompareAssumption": {
                "type": "object",
                "required": ["kind", "symbol", "op", "value"],
                "additionalProperties": false,
                "properties": {
                    "kind": {"const": "compare"},
                    "symbol": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"},
                    "op": {"$ref": "#/$defs/ComparisonOp"},
                    "value": {"$ref": "#/$defs/Expr"}
                }
            },
            "Validate": {
                "type": "object",
                "required": ["intent", "assumptions"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "validate"},
                    "assumptions": {"type": "array", "items": {"$ref": "#/$defs/Assumption"}}
                }
            },
            "Bounds": {
                "type": "object",
                "required": ["intent", "assumptions", "symbol"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "bounds"},
                    "assumptions": {"type": "array", "items": {"$ref": "#/$defs/Assumption"}},
                    "symbol": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"}
                }
            },
            "Entails": {
                "type": "object",
                "required": ["intent", "assumptions", "query"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "entails"},
                    "assumptions": {"type": "array", "items": {"$ref": "#/$defs/Assumption"}},
                    "query": {"$ref": "#/$defs/Assumption"}
                }
            },
            "BoundedAssumption": {
                "type": "object",
                "required": ["kind", "symbol", "lower", "upper"],
                "additionalProperties": false,
                "properties": {
                    "kind": {"const": "bounded"},
                    "symbol": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"},
                    "lower": {"$ref": "#/$defs/Expr"},
                    "upper": {"$ref": "#/$defs/Expr"}
                }
            }
        }
    })
}

#[derive(Clone, Debug, Default)]
struct Context {
    facts: BTreeMap<String, SymbolFacts>,
}

#[derive(Clone, Debug, Default)]
struct SymbolFacts {
    lower: Option<InternalBound>,
    upper: Option<InternalBound>,
    excluded: BTreeSet<Rational>,
    domains: BTreeSet<AssumptionDomain>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InternalBound {
    value: Rational,
    inclusive: bool,
}

struct Entailment {
    entailed: bool,
    evidence: Vec<String>,
}

impl Context {
    fn from_assumptions(assumptions: &[Assumption]) -> Result<Self, String> {
        let mut context = Context::default();
        for assumption in assumptions {
            context.apply_assumption(assumption)?;
        }
        context.validate_consistency()?;
        Ok(context)
    }

    fn apply_assumption(&mut self, assumption: &Assumption) -> Result<(), String> {
        match assumption {
            Assumption::Domain { symbol, domain } => {
                validate_symbol(symbol)?;
                let facts = self.facts.entry(symbol.clone()).or_default();
                facts.domains.insert(*domain);
                apply_domain_bounds(facts, *domain)?;
                Ok(())
            }
            Assumption::Compare { symbol, op, value } => {
                validate_symbol(symbol)?;
                let value = value.evaluate()?;
                let facts = self.facts.entry(symbol.clone()).or_default();
                apply_comparison(facts, *op, value)
            }
            Assumption::Bounded {
                symbol,
                lower,
                upper,
            } => {
                validate_symbol(symbol)?;
                let lower_val = lower.evaluate()?;
                let upper_val = upper.evaluate()?;
                let facts = self.facts.entry(symbol.clone()).or_default();
                apply_comparison(facts, ComparisonOp::Gte, lower_val)?;
                apply_comparison(facts, ComparisonOp::Lte, upper_val)?;
                Ok(())
            }
        }
    }

    fn validate_consistency(&self) -> Result<(), String> {
        for (symbol, facts) in &self.facts {
            if let (Some(lower), Some(upper)) = (&facts.lower, &facts.upper) {
                if lower.value > upper.value {
                    return Err(format!("assumptions for `{symbol}` have empty bounds"));
                }
                if lower.value == upper.value && (!lower.inclusive || !upper.inclusive) {
                    return Err(format!("assumptions for `{symbol}` have empty bounds"));
                }
                if lower.value == upper.value && facts.excluded.contains(&lower.value) {
                    return Err(format!(
                        "assumptions for `{symbol}` exclude their only value"
                    ));
                }
            }
            if facts.domains.contains(&AssumptionDomain::Positive)
                && facts.domains.contains(&AssumptionDomain::Nonpositive)
            {
                return Err(format!(
                    "assumptions for `{symbol}` require positive and nonpositive"
                ));
            }
            if facts.domains.contains(&AssumptionDomain::Negative)
                && facts.domains.contains(&AssumptionDomain::Nonnegative)
            {
                return Err(format!(
                    "assumptions for `{symbol}` require negative and nonnegative"
                ));
            }
        }
        Ok(())
    }

    fn fact_for(&self, symbol: &str) -> SymbolFacts {
        self.facts.get(symbol).cloned().unwrap_or_default()
    }

    fn derived_facts(&self) -> Vec<DerivedFact> {
        self.facts
            .iter()
            .map(|(symbol, facts)| DerivedFact {
                symbol: symbol.clone(),
                lower: facts.lower.clone().map(bound),
                upper: facts.upper.clone().map(bound),
                excluded: facts.excluded.iter().map(exact_rational).collect(),
                domains: facts.domains.iter().copied().collect(),
            })
            .collect()
    }

    fn entails(&self, query: &Assumption) -> Result<Entailment, String> {
        match query {
            Assumption::Domain { symbol, domain } => {
                validate_symbol(symbol)?;
                let facts = self.fact_for(symbol);
                if facts.domains.contains(domain) {
                    return Ok(Entailment {
                        entailed: true,
                        evidence: vec![format!("domain `{domain:?}` was asserted for `{symbol}`")],
                    });
                }
                domain_entailment(symbol, *domain, &facts)
            }
            Assumption::Compare { symbol, op, value } => {
                validate_symbol(symbol)?;
                let value = value.evaluate()?;
                let facts = self.fact_for(symbol);
                comparison_entailment(symbol, *op, &value, &facts)
            }
            Assumption::Bounded {
                symbol,
                lower,
                upper,
            } => {
                validate_symbol(symbol)?;
                let lower_val = lower.evaluate()?;
                let upper_val = upper.evaluate()?;
                let facts = self.fact_for(symbol);
                let lower_ent =
                    comparison_entailment(symbol, ComparisonOp::Gte, &lower_val, &facts)?;
                let upper_ent =
                    comparison_entailment(symbol, ComparisonOp::Lte, &upper_val, &facts)?;
                let entailed = lower_ent.entailed && upper_ent.entailed;
                let mut evidence = lower_ent.evidence;
                evidence.extend(upper_ent.evidence);
                Ok(Entailment { entailed, evidence })
            }
        }
    }
}

fn apply_domain_bounds(facts: &mut SymbolFacts, domain: AssumptionDomain) -> Result<(), String> {
    match domain {
        AssumptionDomain::Rational | AssumptionDomain::Integer => Ok(()),
        AssumptionDomain::Natural => {
            facts.domains.insert(AssumptionDomain::Integer);
            apply_comparison(facts, ComparisonOp::Gte, Rational::one())
        }
        AssumptionDomain::Nonzero => {
            facts.excluded.insert(Rational::zero());
            Ok(())
        }
        AssumptionDomain::Positive => apply_comparison(facts, ComparisonOp::Gt, Rational::zero()),
        AssumptionDomain::Negative => apply_comparison(facts, ComparisonOp::Lt, Rational::zero()),
        AssumptionDomain::Nonnegative => {
            apply_comparison(facts, ComparisonOp::Gte, Rational::zero())
        }
        AssumptionDomain::Nonpositive => {
            apply_comparison(facts, ComparisonOp::Lte, Rational::zero())
        }
        AssumptionDomain::UnitInterval => {
            apply_comparison(facts, ComparisonOp::Gte, Rational::zero())?;
            apply_comparison(facts, ComparisonOp::Lte, Rational::one())
        }
    }
}

fn apply_comparison(
    facts: &mut SymbolFacts,
    op: ComparisonOp,
    value: Rational,
) -> Result<(), String> {
    match op {
        ComparisonOp::Eq => {
            merge_lower(
                facts,
                InternalBound {
                    value: value.clone(),
                    inclusive: true,
                },
            );
            merge_upper(
                facts,
                InternalBound {
                    value,
                    inclusive: true,
                },
            );
            Ok(())
        }
        ComparisonOp::Neq => {
            facts.excluded.insert(value);
            Ok(())
        }
        ComparisonOp::Gt => {
            merge_lower(
                facts,
                InternalBound {
                    value,
                    inclusive: false,
                },
            );
            Ok(())
        }
        ComparisonOp::Gte => {
            merge_lower(
                facts,
                InternalBound {
                    value,
                    inclusive: true,
                },
            );
            Ok(())
        }
        ComparisonOp::Lt => {
            merge_upper(
                facts,
                InternalBound {
                    value,
                    inclusive: false,
                },
            );
            Ok(())
        }
        ComparisonOp::Lte => {
            merge_upper(
                facts,
                InternalBound {
                    value,
                    inclusive: true,
                },
            );
            Ok(())
        }
    }
}

fn merge_lower(facts: &mut SymbolFacts, candidate: InternalBound) {
    if facts
        .lower
        .as_ref()
        .is_none_or(|current| is_stronger_lower(&candidate, current))
    {
        facts.lower = Some(candidate);
    }
}

fn merge_upper(facts: &mut SymbolFacts, candidate: InternalBound) {
    if facts
        .upper
        .as_ref()
        .is_none_or(|current| is_stronger_upper(&candidate, current))
    {
        facts.upper = Some(candidate);
    }
}

fn is_stronger_lower(candidate: &InternalBound, current: &InternalBound) -> bool {
    candidate.value > current.value
        || (candidate.value == current.value && !candidate.inclusive && current.inclusive)
}

fn is_stronger_upper(candidate: &InternalBound, current: &InternalBound) -> bool {
    candidate.value < current.value
        || (candidate.value == current.value && !candidate.inclusive && current.inclusive)
}

fn domain_entailment(
    symbol: &str,
    domain: AssumptionDomain,
    facts: &SymbolFacts,
) -> Result<Entailment, String> {
    let zero = Rational::zero();
    let one = Rational::one();
    let entailed = match domain {
        AssumptionDomain::Rational => {
            !facts.domains.is_empty() || facts.lower.is_some() || facts.upper.is_some()
        }
        AssumptionDomain::Integer => false,
        AssumptionDomain::Natural => {
            facts.domains.contains(&AssumptionDomain::Integer)
                && comparison_entailment(symbol, ComparisonOp::Gte, &one, facts)?.entailed
        }
        AssumptionDomain::Nonzero => {
            facts.excluded.contains(&zero)
                || comparison_entailment(symbol, ComparisonOp::Gt, &zero, facts)?.entailed
                || comparison_entailment(symbol, ComparisonOp::Lt, &zero, facts)?.entailed
        }
        AssumptionDomain::Positive => {
            comparison_entailment(symbol, ComparisonOp::Gt, &zero, facts)?.entailed
        }
        AssumptionDomain::Negative => {
            comparison_entailment(symbol, ComparisonOp::Lt, &zero, facts)?.entailed
        }
        AssumptionDomain::Nonnegative => {
            comparison_entailment(symbol, ComparisonOp::Gte, &zero, facts)?.entailed
        }
        AssumptionDomain::Nonpositive => {
            comparison_entailment(symbol, ComparisonOp::Lte, &zero, facts)?.entailed
        }
        AssumptionDomain::UnitInterval => {
            comparison_entailment(symbol, ComparisonOp::Gte, &zero, facts)?.entailed
                && comparison_entailment(symbol, ComparisonOp::Lte, &one, facts)?.entailed
        }
    };
    Ok(Entailment {
        entailed,
        evidence: if entailed {
            vec![format!("bounds imply `{domain:?}` for `{symbol}`")]
        } else {
            Vec::new()
        },
    })
}

fn comparison_entailment(
    symbol: &str,
    op: ComparisonOp,
    value: &Rational,
    facts: &SymbolFacts,
) -> Result<Entailment, String> {
    let entailed = match op {
        ComparisonOp::Eq => {
            facts
                .lower
                .as_ref()
                .zip(facts.upper.as_ref())
                .is_some_and(|(lower, upper)| {
                    lower.value == *value
                        && upper.value == *value
                        && lower.inclusive
                        && upper.inclusive
                })
        }
        ComparisonOp::Neq => {
            facts.excluded.contains(value)
                || comparison_entailment(symbol, ComparisonOp::Gt, value, facts)?.entailed
                || comparison_entailment(symbol, ComparisonOp::Lt, value, facts)?.entailed
        }
        ComparisonOp::Gt => facts.lower.as_ref().is_some_and(|lower| {
            lower.value > *value || (lower.value == *value && !lower.inclusive)
        }),
        ComparisonOp::Gte => facts
            .lower
            .as_ref()
            .is_some_and(|lower| lower.value >= *value),
        ComparisonOp::Lt => facts.upper.as_ref().is_some_and(|upper| {
            upper.value < *value || (upper.value == *value && !upper.inclusive)
        }),
        ComparisonOp::Lte => facts
            .upper
            .as_ref()
            .is_some_and(|upper| upper.value <= *value),
    };
    Ok(Entailment {
        entailed,
        evidence: if entailed {
            vec![format!("bounds imply `{symbol}` {op:?} {value}")]
        } else {
            Vec::new()
        },
    })
}

fn bound(bound: InternalBound) -> Bound {
    Bound {
        value: exact_rational(&bound.value),
        inclusive: bound.inclusive,
    }
}

fn exact_rational(value: &Rational) -> ExactRational {
    ExactRational {
        numerator: value.numerator().to_string(),
        denominator: value.denominator().to_string(),
        display: value.to_string(),
    }
}

fn validate_symbol(symbol: &str) -> Result<(), String> {
    if is_valid_symbol_name(symbol) {
        Ok(())
    } else {
        Err(format!("invalid symbol name `{symbol}`"))
    }
}

fn is_valid_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn default_checks() -> Vec<AssumptionCheck> {
    vec![
        AssumptionCheck {
            name: "assumptions_consistent".to_owned(),
            passed: true,
        },
        AssumptionCheck {
            name: "derived_bounds_exact".to_owned(),
            passed: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i32) -> Expr {
        Expr::Integer {
            value: value.to_string(),
        }
    }

    fn cmp(symbol: &str, op: ComparisonOp, value: i32) -> Assumption {
        Assumption::Compare {
            symbol: symbol.to_owned(),
            op,
            value: int(value),
        }
    }

    fn domain(symbol: &str, domain: AssumptionDomain) -> Assumption {
        Assumption::Domain {
            symbol: symbol.to_owned(),
            domain,
        }
    }

    fn expected_checks() -> Vec<AssumptionCheck> {
        vec![
            AssumptionCheck {
                name: "assumptions_consistent".to_owned(),
                passed: true,
            },
            AssumptionCheck {
                name: "derived_bounds_exact".to_owned(),
                passed: true,
            },
        ]
    }

    fn assert_entails(assumptions: Vec<Assumption>, query: Assumption, expected: bool) {
        match (AssumptionsRequest::Entails { assumptions, query }).evaluate() {
            AssumptionsResponse::Entailment {
                entailed, checks, ..
            } => {
                assert_eq!(entailed, expected);
                assert_eq!(checks, expected_checks());
            }
            other => panic!("expected entailment, got {other:?}"),
        }
    }

    fn assert_bounds(
        assumptions: Vec<Assumption>,
        lower: Option<(&str, bool)>,
        upper: Option<(&str, bool)>,
    ) {
        match (AssumptionsRequest::Bounds {
            assumptions,
            symbol: "x".to_owned(),
        })
        .evaluate()
        {
            AssumptionsResponse::Bounds {
                lower: actual_lower,
                upper: actual_upper,
                ..
            } => {
                assert_eq!(
                    actual_lower.map(|bound| (bound.value.display, bound.inclusive)),
                    lower.map(|(value, inclusive)| (value.to_owned(), inclusive))
                );
                assert_eq!(
                    actual_upper.map(|bound| (bound.value.display, bound.inclusive)),
                    upper.map(|(value, inclusive)| (value.to_owned(), inclusive))
                );
            }
            other => panic!("expected bounds, got {other:?}"),
        }
    }

    #[test]
    fn validates_and_derives_symbol_facts() {
        let response = AssumptionsRequest::Validate {
            assumptions: vec![
                domain("x", AssumptionDomain::Integer),
                domain("x", AssumptionDomain::Nonnegative),
                cmp("x", ComparisonOp::Lte, 10),
                cmp("x", ComparisonOp::Neq, 3),
            ],
        }
        .evaluate();

        match response {
            AssumptionsResponse::Context { facts, checks, .. } => {
                assert_eq!(checks, expected_checks());
                assert_eq!(facts.len(), 1);
                assert_eq!(facts[0].symbol, "x");
                assert_eq!(facts[0].lower.as_ref().unwrap().value.display, "0");
                assert!(facts[0].lower.as_ref().unwrap().inclusive);
                assert_eq!(facts[0].upper.as_ref().unwrap().value.display, "10");
                assert!(facts[0].upper.as_ref().unwrap().inclusive);
                assert_eq!(facts[0].excluded[0].display, "3");
                assert_eq!(
                    facts[0].domains,
                    vec![AssumptionDomain::Integer, AssumptionDomain::Nonnegative]
                );
            }
            other => panic!("expected context, got {other:?}"),
        }
    }

    #[test]
    fn bounds_reports_requested_symbol_only() {
        let response = AssumptionsRequest::Bounds {
            assumptions: vec![cmp("x", ComparisonOp::Gt, 2), cmp("y", ComparisonOp::Lt, 5)],
            symbol: "x".to_owned(),
        }
        .evaluate();

        match response {
            AssumptionsResponse::Bounds {
                symbol,
                lower,
                upper,
                excluded,
                domains,
                checks,
                ..
            } => {
                assert_eq!(symbol, "x");
                assert_eq!(lower.unwrap().value.display, "2");
                assert!(upper.is_none());
                assert!(excluded.is_empty());
                assert!(domains.is_empty());
                assert_eq!(checks, expected_checks());
            }
            other => panic!("expected bounds, got {other:?}"),
        }
    }

    #[test]
    fn answers_entailment_queries_from_bounds_and_exclusions() {
        let response = AssumptionsRequest::Entails {
            assumptions: vec![
                cmp("x", ComparisonOp::Gte, 2),
                cmp("x", ComparisonOp::Neq, 4),
            ],
            query: cmp("x", ComparisonOp::Gt, 1),
        }
        .evaluate();
        assert!(matches!(
            response,
            AssumptionsResponse::Entailment { entailed: true, .. }
        ));

        let response = AssumptionsRequest::Entails {
            assumptions: vec![
                cmp("x", ComparisonOp::Gte, 2),
                cmp("x", ComparisonOp::Neq, 4),
            ],
            query: cmp("x", ComparisonOp::Neq, 4),
        }
        .evaluate();
        assert!(matches!(
            response,
            AssumptionsResponse::Entailment { entailed: true, .. }
        ));

        let response = AssumptionsRequest::Entails {
            assumptions: vec![cmp("x", ComparisonOp::Gte, 2)],
            query: cmp("x", ComparisonOp::Gt, 2),
        }
        .evaluate();
        assert!(matches!(
            response,
            AssumptionsResponse::Entailment {
                entailed: false,
                ..
            }
        ));
    }

    #[test]
    fn covers_bound_tie_breaking_and_weaker_bounds() {
        assert_bounds(
            vec![
                cmp("x", ComparisonOp::Gte, 2),
                cmp("x", ComparisonOp::Gt, 2),
            ],
            Some(("2", false)),
            None,
        );
        assert_bounds(
            vec![
                cmp("x", ComparisonOp::Gt, 2),
                cmp("x", ComparisonOp::Gte, 2),
            ],
            Some(("2", false)),
            None,
        );
        assert_bounds(
            vec![
                cmp("x", ComparisonOp::Gte, 5),
                cmp("x", ComparisonOp::Gt, 2),
            ],
            Some(("5", true)),
            None,
        );
        assert_bounds(
            vec![
                cmp("x", ComparisonOp::Lte, 2),
                cmp("x", ComparisonOp::Lt, 2),
            ],
            None,
            Some(("2", false)),
        );
        assert_bounds(
            vec![
                cmp("x", ComparisonOp::Lt, 2),
                cmp("x", ComparisonOp::Lte, 2),
            ],
            None,
            Some(("2", false)),
        );
        assert_bounds(
            vec![
                cmp("x", ComparisonOp::Lte, -5),
                cmp("x", ComparisonOp::Lt, 2),
            ],
            None,
            Some(("-5", true)),
        );
    }

    #[test]
    fn covers_comparison_entailment_truth_table() {
        assert_entails(
            vec![cmp("x", ComparisonOp::Eq, 2)],
            cmp("x", ComparisonOp::Eq, 2),
            true,
        );
        assert_entails(
            vec![
                cmp("x", ComparisonOp::Gte, 2),
                cmp("x", ComparisonOp::Lte, 3),
            ],
            cmp("x", ComparisonOp::Eq, 2),
            false,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Gt, 2)],
            cmp("x", ComparisonOp::Gte, 2),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Gte, 2)],
            cmp("x", ComparisonOp::Gte, 3),
            false,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Gte, 3)],
            cmp("x", ComparisonOp::Gte, 2),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lt, 0)],
            cmp("x", ComparisonOp::Lt, 1),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lt, 2)],
            cmp("x", ComparisonOp::Lt, 1),
            false,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lt, 2)],
            cmp("x", ComparisonOp::Lt, 2),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lte, 2)],
            cmp("x", ComparisonOp::Lt, 2),
            false,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lt, 2)],
            cmp("x", ComparisonOp::Lte, 2),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lte, 1)],
            cmp("x", ComparisonOp::Lte, 2),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lte, 2)],
            cmp("x", ComparisonOp::Lte, 1),
            false,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Gt, 0)],
            cmp("x", ComparisonOp::Neq, 0),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Lt, 0)],
            cmp("x", ComparisonOp::Neq, 0),
            true,
        );
        assert_entails(
            vec![cmp("x", ComparisonOp::Gte, 0)],
            cmp("x", ComparisonOp::Neq, 0),
            false,
        );
    }

    #[test]
    fn covers_domain_entailment_edges() {
        assert_entails(Vec::new(), domain("x", AssumptionDomain::Rational), false);
        assert_entails(
            vec![cmp("x", ComparisonOp::Gte, 0)],
            domain("x", AssumptionDomain::Rational),
            true,
        );
        assert_entails(
            vec![domain("x", AssumptionDomain::Integer)],
            domain("x", AssumptionDomain::Rational),
            true,
        );
        assert_entails(
            vec![domain("x", AssumptionDomain::Integer)],
            domain("x", AssumptionDomain::Integer),
            true,
        );
        assert_entails(
            vec![domain("x", AssumptionDomain::Nonpositive)],
            domain("x", AssumptionDomain::Positive),
            false,
        );
    }

    fn bounded(symbol: &str, lo: i32, hi: i32) -> Assumption {
        Assumption::Bounded {
            symbol: symbol.to_owned(),
            lower: int(lo),
            upper: int(hi),
        }
    }

    #[test]
    fn natural_domain_applies_integer_and_lower_one_bound() {
        // natural → lower bound ≥ 1 (inclusive)
        assert_bounds(
            vec![domain("x", AssumptionDomain::Natural)],
            Some(("1", true)),
            None,
        );
        // natural also inserts Integer into domains
        match (AssumptionsRequest::Validate {
            assumptions: vec![domain("x", AssumptionDomain::Natural)],
        })
        .evaluate()
        {
            AssumptionsResponse::Context { facts, .. } => {
                assert!(facts[0].domains.contains(&AssumptionDomain::Integer));
                assert!(facts[0].domains.contains(&AssumptionDomain::Natural));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unit_interval_domain_applies_zero_one_bounds() {
        assert_bounds(
            vec![domain("x", AssumptionDomain::UnitInterval)],
            Some(("0", true)),
            Some(("1", true)),
        );
    }

    #[test]
    fn bounded_assumption_applies_inclusive_lower_and_upper() {
        assert_bounds(
            vec![bounded("x", 5, 10)],
            Some(("5", true)),
            Some(("10", true)),
        );
    }

    #[test]
    fn natural_entailment_examples() {
        // integer ∧ >=1 → natural
        assert_entails(
            vec![
                domain("x", AssumptionDomain::Integer),
                cmp("x", ComparisonOp::Gte, 1),
            ],
            domain("x", AssumptionDomain::Natural),
            true,
        );
        // integer alone is not enough (kills &&→|| mutation)
        assert_entails(
            vec![domain("x", AssumptionDomain::Integer)],
            domain("x", AssumptionDomain::Natural),
            false,
        );
        // >=1 alone is not enough (kills &&→|| mutation)
        assert_entails(
            vec![cmp("x", ComparisonOp::Gte, 1)],
            domain("x", AssumptionDomain::Natural),
            false,
        );
        // integer ∧ >=0 is not natural (0 ∉ ℕ; kills >=→== in bound check)
        assert_entails(
            vec![
                domain("x", AssumptionDomain::Integer),
                cmp("x", ComparisonOp::Gte, 0),
            ],
            domain("x", AssumptionDomain::Natural),
            false,
        );
        // non_negative ∧ lte(10) does NOT entail positive (x could be 0)
        assert_entails(
            vec![
                domain("x", AssumptionDomain::Nonnegative),
                cmp("x", ComparisonOp::Lte, 10),
            ],
            domain("x", AssumptionDomain::Positive),
            false,
        );
    }

    #[test]
    fn unit_interval_entailment_examples() {
        // unit_interval ⊢ lte(1) — true
        assert_entails(
            vec![domain("x", AssumptionDomain::UnitInterval)],
            cmp("x", ComparisonOp::Lte, 1),
            true,
        );
        // unit_interval ⊢ gte(0) — true
        assert_entails(
            vec![domain("x", AssumptionDomain::UnitInterval)],
            cmp("x", ComparisonOp::Gte, 0),
            true,
        );
        // gte(0) ∧ lte(1) → unit_interval — true
        assert_entails(
            vec![
                cmp("x", ComparisonOp::Gte, 0),
                cmp("x", ComparisonOp::Lte, 1),
            ],
            domain("x", AssumptionDomain::UnitInterval),
            true,
        );
        // gte(0) alone → NOT unit_interval (kills &&→|| mutation on upper check)
        assert_entails(
            vec![cmp("x", ComparisonOp::Gte, 0)],
            domain("x", AssumptionDomain::UnitInterval),
            false,
        );
        // lte(1) alone → NOT unit_interval (kills &&→|| mutation on lower check)
        assert_entails(
            vec![cmp("x", ComparisonOp::Lte, 1)],
            domain("x", AssumptionDomain::UnitInterval),
            false,
        );
        // nonneg ∧ lte(10) → NOT unit_interval (upper bound > 1)
        assert_entails(
            vec![
                domain("x", AssumptionDomain::Nonnegative),
                cmp("x", ComparisonOp::Lte, 10),
            ],
            domain("x", AssumptionDomain::UnitInterval),
            false,
        );
    }

    #[test]
    fn bounded_assumption_entailment() {
        // bounded(5,10) ⊢ gte(5) → true
        assert_entails(
            vec![bounded("x", 5, 10)],
            cmp("x", ComparisonOp::Gte, 5),
            true,
        );
        // bounded(5,10) ⊢ lte(10) → true
        assert_entails(
            vec![bounded("x", 5, 10)],
            cmp("x", ComparisonOp::Lte, 10),
            true,
        );
        // bounded(5,10) ⊢ bounded(5,10) → true (both bounds entailed)
        assert_entails(vec![bounded("x", 5, 10)], bounded("x", 5, 10), true);
        // bounded(5,10) ⊢ bounded(6,10) → false (gte(6) not entailed from lower=5)
        assert_entails(vec![bounded("x", 5, 10)], bounded("x", 6, 10), false);
        // bounded(5,10) ⊢ bounded(5,9) → false (lte(9) not entailed from upper=10)
        assert_entails(vec![bounded("x", 5, 10)], bounded("x", 5, 9), false);
    }

    #[test]
    fn rejects_invalid_or_inconsistent_assumptions() {
        assert!(matches!(
            (AssumptionsRequest::Validate {
                assumptions: vec![cmp("1x", ComparisonOp::Gt, 0)]
            })
            .evaluate(),
            AssumptionsResponse::Error { reason, .. } if reason == "invalid symbol name `1x`"
        ));
        assert!(matches!(
            (AssumptionsRequest::Validate {
                assumptions: vec![cmp("x", ComparisonOp::Gt, 2), cmp("x", ComparisonOp::Lte, 2)]
            })
            .evaluate(),
            AssumptionsResponse::Error { reason, .. } if reason == "assumptions for `x` have empty bounds"
        ));
        assert!(matches!(
            (AssumptionsRequest::Validate {
                assumptions: vec![cmp("x", ComparisonOp::Eq, 2), cmp("x", ComparisonOp::Neq, 2)]
            })
            .evaluate(),
            AssumptionsResponse::Error { reason, .. } if reason == "assumptions for `x` exclude their only value"
        ));
        assert!(matches!(
            (AssumptionsRequest::Validate {
                assumptions: vec![
                    cmp("x", ComparisonOp::Eq, 2),
                    cmp("x", ComparisonOp::Neq, 3)
                ]
            })
            .evaluate(),
            AssumptionsResponse::Context { .. }
        ));
        assert!(matches!(
            (AssumptionsRequest::Validate {
                assumptions: vec![domain("x_1", AssumptionDomain::Rational)]
            })
            .evaluate(),
            AssumptionsResponse::Context { .. }
        ));
    }
}
