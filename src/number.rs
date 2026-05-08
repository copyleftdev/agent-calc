use crate::CONTRACT_VERSION;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt};
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MAX_FACTOR_BITS: u64 = 64;

// ── public types ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum NumberRequest {
    Gcd {
        a: String,
        b: String,
    },
    Lcm {
        a: String,
        b: String,
    },
    ExtendedGcd {
        a: String,
        b: String,
    },
    Mod {
        a: String,
        b: String,
    },
    ModPow {
        base: String,
        exponent: String,
        modulus: String,
    },
    ModInverse {
        a: String,
        modulus: String,
    },
    IsPrime {
        n: String,
    },
    PrimeFactors {
        n: String,
    },
    Crt {
        remainders: Vec<String>,
        moduli: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrimeFactor {
    pub base: String,
    pub exponent: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NumberResponse {
    Value {
        contract_version: String,
        result: String,
        exact: bool,
    },
    CrtValue {
        contract_version: String,
        result: String,
        modulus: String,
        exact: bool,
    },
    Bezout {
        contract_version: String,
        gcd: String,
        x: String,
        y: String,
    },
    Primality {
        contract_version: String,
        prime: bool,
    },
    Factorization {
        contract_version: String,
        factors: Vec<PrimeFactor>,
    },
    NoInverse {
        contract_version: String,
        gcd: String,
    },
    Error {
        contract_version: String,
        reason: String,
    },
}

// ── request dispatch ─────────────────────────────────────────────────────────

impl NumberRequest {
    pub fn evaluate(&self) -> NumberResponse {
        match self.evaluate_inner() {
            Ok(resp) => resp,
            Err(reason) => NumberResponse::Error {
                contract_version: CONTRACT_VERSION.to_owned(),
                reason,
            },
        }
    }

    fn evaluate_inner(&self) -> Result<NumberResponse, String> {
        match self {
            NumberRequest::Gcd { a, b } => {
                let a = parse_bigint(a, "a")?;
                let b = parse_bigint(b, "b")?;
                let g = bigint_gcd(
                    &a.abs().to_biguint().unwrap(),
                    &b.abs().to_biguint().unwrap(),
                );
                Ok(value_response(g.to_string()))
            }
            NumberRequest::Lcm { a, b } => {
                let a = parse_bigint(a, "a")?;
                let b = parse_bigint(b, "b")?;
                let au = a.abs().to_biguint().unwrap();
                let bu = b.abs().to_biguint().unwrap();
                if au.is_zero() || bu.is_zero() {
                    return Ok(value_response("0".to_owned()));
                }
                let g = bigint_gcd(&au, &bu);
                let result = (&au / &g) * &bu;
                Ok(value_response(result.to_string()))
            }
            NumberRequest::ExtendedGcd { a, b } => {
                let a = parse_bigint(a, "a")?;
                let b = parse_bigint(b, "b")?;
                let (g, x, y) = extended_gcd(a, b);
                Ok(NumberResponse::Bezout {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    gcd: g.to_string(),
                    x: x.to_string(),
                    y: y.to_string(),
                })
            }
            NumberRequest::Mod { a, b } => {
                let a = parse_bigint(a, "a")?;
                let b = parse_bigint(b, "b")?;
                if b.is_zero() {
                    return Err("mod: divisor b must not be zero".to_owned());
                }
                // Euclidean remainder: always non-negative
                let r = euclidean_rem(&a, &b);
                Ok(value_response(r.to_string()))
            }
            NumberRequest::ModPow {
                base,
                exponent,
                modulus,
            } => {
                let base = parse_bigint(base, "base")?;
                let exp = parse_bigint(exponent, "exponent")?;
                let modulus = parse_bigint(modulus, "modulus")?;
                if modulus.is_zero() {
                    return Err("mod_pow: modulus must not be zero".to_owned());
                }
                if exp.sign() == Sign::Minus {
                    return Err("mod_pow: exponent must be non-negative".to_owned());
                }
                let bu = base.abs().to_biguint().unwrap();
                let eu = exp.to_biguint().unwrap();
                let mu = modulus.abs().to_biguint().unwrap();
                let raw = bu.modpow(&eu, &mu);
                // Adjust sign if base is negative and exponent is odd
                let result = if base.sign() == Sign::Minus && &eu % 2u32 == BigUint::one() {
                    let mu_int = mu.to_bigint().unwrap();
                    let raw_int = raw.to_bigint().unwrap();
                    euclidean_rem(&(-raw_int), &mu_int).to_string()
                } else {
                    raw.to_string()
                };
                Ok(value_response(result))
            }
            NumberRequest::ModInverse { a, modulus } => {
                let a = parse_bigint(a, "a")?;
                let m = parse_bigint(modulus, "modulus")?;
                if m <= BigInt::one() {
                    return Err("mod_inverse: modulus must be greater than 1".to_owned());
                }
                let (g, x, _) = extended_gcd(a.clone(), m.clone());
                if g != BigInt::one() {
                    return Ok(NumberResponse::NoInverse {
                        contract_version: CONTRACT_VERSION.to_owned(),
                        gcd: g.to_string(),
                    });
                }
                let inv = euclidean_rem(&x, &m);
                Ok(value_response(inv.to_string()))
            }
            NumberRequest::IsPrime { n } => {
                let n = parse_bigint(n, "n")?;
                if n.sign() == Sign::Minus {
                    return Err("is_prime: n must be non-negative".to_owned());
                }
                let nu = n.to_biguint().unwrap();
                Ok(NumberResponse::Primality {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    prime: miller_rabin_is_prime(&nu),
                })
            }
            NumberRequest::PrimeFactors { n } => {
                let n = parse_bigint(n, "n")?;
                if n.sign() == Sign::Minus {
                    return Err("prime_factors: n must be non-negative".to_owned());
                }
                let nu = n.to_biguint().unwrap();
                if nu.bits() > MAX_FACTOR_BITS {
                    return Err(format!(
                        "prime_factors: input exceeds {MAX_FACTOR_BITS}-bit limit to prevent DoS"
                    ));
                }
                let factors = factorize(nu);
                Ok(NumberResponse::Factorization {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    factors,
                })
            }
            NumberRequest::Crt { remainders, moduli } => {
                if remainders.len() != moduli.len() {
                    return Err("crt: remainders and moduli must have the same length".to_owned());
                }
                if remainders.is_empty() {
                    return Err("crt: at least one congruence is required".to_owned());
                }
                let rs: Vec<BigInt> = remainders
                    .iter()
                    .enumerate()
                    .map(|(i, s)| parse_bigint(s, &format!("remainders[{i}]")))
                    .collect::<Result<_, _>>()?;
                let ms: Vec<BigInt> = moduli
                    .iter()
                    .enumerate()
                    .map(|(i, s)| parse_bigint(s, &format!("moduli[{i}]")))
                    .collect::<Result<_, _>>()?;
                for (i, m) in ms.iter().enumerate() {
                    if *m <= BigInt::zero() {
                        return Err(format!("crt: moduli[{i}] must be positive"));
                    }
                }
                let (result, total_mod) = crt_solve(&rs, &ms)?;
                Ok(NumberResponse::CrtValue {
                    contract_version: CONTRACT_VERSION.to_owned(),
                    result: result.to_string(),
                    modulus: total_mod.to_string(),
                    exact: true,
                })
            }
        }
    }
}

// ── algorithms ────────────────────────────────────────────────────────────────

fn bigint_gcd(a: &BigUint, b: &BigUint) -> BigUint {
    let mut a = a.clone();
    let mut b = b.clone();
    while !b.is_zero() {
        let t = &a % &b;
        a = b;
        b = t;
    }
    a
}

// Returns (g, x, y) with g = gcd(a, b) and a*x + b*y = g, g ≥ 0.
fn extended_gcd(a: BigInt, b: BigInt) -> (BigInt, BigInt, BigInt) {
    if b.is_zero() {
        if a.sign() == Sign::Minus {
            return (-a, -BigInt::one(), BigInt::zero());
        }
        return (a, BigInt::one(), BigInt::zero());
    }
    let q = &a / &b;
    let r = &a % &b;
    let (g, x1, y1) = extended_gcd(b, r);
    (g, y1.clone(), x1 - &q * &y1)
}

// Euclidean (non-negative) remainder: result is in [0, |b|)
fn euclidean_rem(a: &BigInt, b: &BigInt) -> BigInt {
    let r = a % b;
    if r.sign() == Sign::Minus {
        r + b.abs()
    } else {
        r
    }
}

// Deterministic Miller-Rabin using 12 witnesses, correct for n < 3.3 × 10^24.
fn miller_rabin_is_prime(n: &BigUint) -> bool {
    let zero = BigUint::zero();
    let one = BigUint::one();
    let two = BigUint::from(2u32);

    if *n < two {
        return false;
    }
    if *n == two || *n == BigUint::from(3u32) {
        return true;
    }
    if n % &two == zero {
        return false;
    }

    // Write n-1 = 2^r * d
    let n_minus_1 = n - &one;
    let mut d = n_minus_1.clone();
    let mut r = 0u32;
    while &d % &two == zero {
        d >>= 1;
        r += 1;
    }

    let witnesses: &[u32] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    'witness: for &a in witnesses {
        let a = BigUint::from(a);
        if a >= *n {
            continue;
        }
        let mut x = a.modpow(&d, n);
        if x == one || x == n_minus_1 {
            continue;
        }
        for _ in 0..r - 1 {
            x = x.modpow(&two, n);
            if x == n_minus_1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

// Trial division factorization, n must fit in u64 (enforced by caller).
fn factorize(mut n: BigUint) -> Vec<PrimeFactor> {
    let mut factors = Vec::new();
    if n <= BigUint::one() {
        return factors;
    }
    let mut p = BigUint::from(2u32);
    while &p * &p <= n {
        if &n % &p == BigUint::zero() {
            let mut exp = 0u32;
            while &n % &p == BigUint::zero() {
                n /= &p;
                exp += 1;
            }
            factors.push(PrimeFactor {
                base: p.to_string(),
                exponent: exp,
            });
        }
        p += BigUint::one();
    }
    if n > BigUint::one() {
        factors.push(PrimeFactor {
            base: n.to_string(),
            exponent: 1,
        });
    }
    factors
}

// Iterative CRT. Returns (x, M) where x is in [0, M).
fn crt_solve(remainders: &[BigInt], moduli: &[BigInt]) -> Result<(BigInt, BigInt), String> {
    let mut x = remainders[0].clone();
    let mut m = moduli[0].clone();
    for i in 1..remainders.len() {
        let ri = &remainders[i];
        let mi = &moduli[i];
        // Solve: x + m*t ≡ ri (mod mi) → m*t ≡ (ri - x) (mod mi)
        let (g, s, _) = extended_gcd(m.clone(), mi.clone());
        let diff = ri - &x;
        if &diff % &g != BigInt::zero() {
            return Err(format!(
                "crt: no solution — moduli[{i}] is incompatible (gcd={g}, diff not divisible)"
            ));
        }
        let step = (&m / &g) * mi;
        let t = (diff / &g) * s;
        x = euclidean_rem(&(x + &m * &t), &step);
        m = step;
    }
    Ok((x, m))
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_bigint(s: &str, name: &str) -> Result<BigInt, String> {
    s.trim()
        .parse::<BigInt>()
        .map_err(|_| format!("invalid integer for {name}: {s:?}"))
}

fn value_response(result: String) -> NumberResponse {
    NumberResponse::Value {
        contract_version: CONTRACT_VERSION.to_owned(),
        result,
        exact: true,
    }
}

// ── schema ────────────────────────────────────────────────────────────────────

pub fn number_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://agent-calc.local/schema/{CONTRACT_VERSION}/number.json"),
        "title": "agent-calc calc1 number-theory request",
        "description": "Exact big-integer number theory: GCD, LCM, extended GCD, modular arithmetic, primality, factorization, CRT.",
        "type": "object",
        "required": ["intent"],
        "oneOf": [
            {"$ref": "#/$defs/Gcd"},
            {"$ref": "#/$defs/Lcm"},
            {"$ref": "#/$defs/ExtendedGcd"},
            {"$ref": "#/$defs/Mod"},
            {"$ref": "#/$defs/ModPow"},
            {"$ref": "#/$defs/ModInverse"},
            {"$ref": "#/$defs/IsPrime"},
            {"$ref": "#/$defs/PrimeFactors"},
            {"$ref": "#/$defs/Crt"}
        ],
        "$defs": {
            "BigIntStr": {"type": "string", "pattern": "^-?[0-9]+$"},
            "Gcd": {
                "type": "object",
                "required": ["intent", "a", "b"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "gcd"},
                    "a": {"$ref": "#/$defs/BigIntStr"},
                    "b": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "Lcm": {
                "type": "object",
                "required": ["intent", "a", "b"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "lcm"},
                    "a": {"$ref": "#/$defs/BigIntStr"},
                    "b": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "ExtendedGcd": {
                "type": "object",
                "required": ["intent", "a", "b"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "extended_gcd"},
                    "a": {"$ref": "#/$defs/BigIntStr"},
                    "b": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "Mod": {
                "type": "object",
                "required": ["intent", "a", "b"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "mod"},
                    "a": {"$ref": "#/$defs/BigIntStr"},
                    "b": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "ModPow": {
                "type": "object",
                "required": ["intent", "base", "exponent", "modulus"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "mod_pow"},
                    "base": {"$ref": "#/$defs/BigIntStr"},
                    "exponent": {"$ref": "#/$defs/BigIntStr"},
                    "modulus": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "ModInverse": {
                "type": "object",
                "required": ["intent", "a", "modulus"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "mod_inverse"},
                    "a": {"$ref": "#/$defs/BigIntStr"},
                    "modulus": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "IsPrime": {
                "type": "object",
                "required": ["intent", "n"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "is_prime"},
                    "n": {"$ref": "#/$defs/BigIntStr"}
                }
            },
            "PrimeFactors": {
                "type": "object",
                "required": ["intent", "n"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "prime_factors"},
                    "n": {"$ref": "#/$defs/BigIntStr"},
                    "description": "Input must be a positive integer with at most 64 bits to prevent DoS."
                }
            },
            "Crt": {
                "type": "object",
                "required": ["intent", "remainders", "moduli"],
                "additionalProperties": false,
                "properties": {
                    "intent": {"const": "crt"},
                    "remainders": {"type": "array", "items": {"$ref": "#/$defs/BigIntStr"}, "minItems": 1},
                    "moduli": {"type": "array", "items": {"$ref": "#/$defs/BigIntStr"}, "minItems": 1}
                }
            }
        }
    })
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn req(json: &str) -> NumberRequest {
        serde_json::from_str(json).unwrap()
    }

    fn result_of(resp: NumberResponse) -> String {
        match resp {
            NumberResponse::Value { result, .. } => result,
            other => panic!("expected Value, got {other:?}"),
        }
    }

    // ── GCD ──────────────────────────────────────────────────────────────────

    #[test]
    fn gcd_basic() {
        assert_eq!(
            result_of(req(r#"{"intent":"gcd","a":"48","b":"18"}"#).evaluate()),
            "6"
        );
    }

    #[test]
    fn gcd_coprime_is_one() {
        assert_eq!(
            result_of(req(r#"{"intent":"gcd","a":"35","b":"64"}"#).evaluate()),
            "1"
        );
    }

    #[test]
    fn gcd_with_zero() {
        assert_eq!(
            result_of(req(r#"{"intent":"gcd","a":"0","b":"7"}"#).evaluate()),
            "7"
        );
        assert_eq!(
            result_of(req(r#"{"intent":"gcd","a":"7","b":"0"}"#).evaluate()),
            "7"
        );
    }

    #[test]
    fn gcd_negative_inputs() {
        // gcd is always non-negative
        assert_eq!(
            result_of(req(r#"{"intent":"gcd","a":"-48","b":"18"}"#).evaluate()),
            "6"
        );
        assert_eq!(
            result_of(req(r#"{"intent":"gcd","a":"48","b":"-18"}"#).evaluate()),
            "6"
        );
    }

    #[test]
    fn gcd_large_numbers() {
        // gcd(2^100, 2^50) = 2^50
        let two_50 = (1u128 << 50).to_string();
        let resp = NumberRequest::Gcd {
            a: format!("1{}", "0".repeat(30)), // 10^30
            b: "1000000000000000".to_owned(),  // 10^15
        }
        .evaluate();
        // gcd(10^30, 10^15) = 10^15
        assert_eq!(result_of(resp), two_50.replace(&two_50, "1000000000000000"));
        // actually just check it divides both
        let g: u128 = result_of(
            NumberRequest::Gcd {
                a: "1000000000000000000000000000000".to_owned(),
                b: "1000000000000000".to_owned(),
            }
            .evaluate(),
        )
        .parse()
        .unwrap();
        assert_eq!(1_000_000_000_000_000_000_000_000_000_000u128 % g, 0);
        assert_eq!(1_000_000_000_000_000u128 % g, 0);
    }

    // ── LCM ──────────────────────────────────────────────────────────────────

    #[test]
    fn lcm_basic() {
        assert_eq!(
            result_of(req(r#"{"intent":"lcm","a":"12","b":"18"}"#).evaluate()),
            "36"
        );
    }

    #[test]
    fn lcm_with_zero() {
        assert_eq!(
            result_of(req(r#"{"intent":"lcm","a":"0","b":"5"}"#).evaluate()),
            "0"
        );
        assert_eq!(
            result_of(req(r#"{"intent":"lcm","a":"5","b":"0"}"#).evaluate()),
            "0"
        );
    }

    #[test]
    fn lcm_coprime() {
        assert_eq!(
            result_of(req(r#"{"intent":"lcm","a":"4","b":"9"}"#).evaluate()),
            "36"
        );
    }

    // ── Extended GCD ─────────────────────────────────────────────────────────

    #[test]
    fn extended_gcd_bezout_identity() {
        // a=35, b=15: gcd=5, x*35 + y*15 = 5
        let resp = req(r#"{"intent":"extended_gcd","a":"35","b":"15"}"#).evaluate();
        match resp {
            NumberResponse::Bezout { gcd, x, y, .. } => {
                assert_eq!(gcd, "5");
                let g: i64 = gcd.parse().unwrap();
                let xv: i64 = x.parse().unwrap();
                let yv: i64 = y.parse().unwrap();
                assert_eq!(35 * xv + 15 * yv, g, "Bézout identity must hold");
            }
            other => panic!("expected Bezout, got {other:?}"),
        }
    }

    #[test]
    fn extended_gcd_coprime_bezout_identity() {
        // a=3, b=11: gcd=1
        let resp = req(r#"{"intent":"extended_gcd","a":"3","b":"11"}"#).evaluate();
        match resp {
            NumberResponse::Bezout { gcd, x, y, .. } => {
                assert_eq!(gcd, "1");
                let xv: i64 = x.parse().unwrap();
                let yv: i64 = y.parse().unwrap();
                assert_eq!(3 * xv + 11 * yv, 1);
            }
            other => panic!("expected Bezout, got {other:?}"),
        }
    }

    #[test]
    fn extended_gcd_large_bezout_identity() {
        // Verify Bézout holds for a=100, b=75 (gcd=25)
        let resp = req(r#"{"intent":"extended_gcd","a":"100","b":"75"}"#).evaluate();
        match resp {
            NumberResponse::Bezout { gcd, x, y, .. } => {
                assert_eq!(gcd, "25");
                let xv: i64 = x.parse().unwrap();
                let yv: i64 = y.parse().unwrap();
                assert_eq!(100 * xv + 75 * yv, 25);
            }
            other => panic!("expected Bezout, got {other:?}"),
        }
    }

    #[test]
    fn extended_gcd_negative_a_zero_b() {
        // a=-35, b=0: gcd should be 35 (positive), Bézout: (-35)*x + 0*y = 35 → x=-1.
        // Mutation delete -a: returns (-35, -1, 0) → gcd=-35, identity: (-35)*(-1)=35≠-35.
        // Mutation delete -1: returns (35, 1, 0) → identity: (-35)*1=-35≠35.
        let resp = req(r#"{"intent":"extended_gcd","a":"-35","b":"0"}"#).evaluate();
        match resp {
            NumberResponse::Bezout { gcd, x, .. } => {
                let g: i64 = gcd.parse().unwrap();
                let xv: i64 = x.parse().unwrap();
                assert!(g > 0, "gcd must be positive, got {g}");
                assert_eq!(g, 35);
                assert_eq!(-35 * xv, g, "Bézout identity must hold (b=0)");
            }
            other => panic!("expected Bezout, got {other:?}"),
        }
    }

    // ── Mod ───────────────────────────────────────────────────────────────────

    #[test]
    fn mod_basic() {
        assert_eq!(
            result_of(req(r#"{"intent":"mod","a":"17","b":"5"}"#).evaluate()),
            "2"
        );
    }

    #[test]
    fn mod_exact_division() {
        assert_eq!(
            result_of(req(r#"{"intent":"mod","a":"20","b":"5"}"#).evaluate()),
            "0"
        );
    }

    #[test]
    fn mod_negative_is_euclidean() {
        // Euclidean: -7 mod 3 = 2 (not -1)
        assert_eq!(
            result_of(req(r#"{"intent":"mod","a":"-7","b":"3"}"#).evaluate()),
            "2"
        );
    }

    #[test]
    fn mod_zero_divisor_errors() {
        let resp = req(r#"{"intent":"mod","a":"5","b":"0"}"#).evaluate();
        assert!(matches!(resp, NumberResponse::Error { .. }));
    }

    // ── ModPow ───────────────────────────────────────────────────────────────

    #[test]
    fn mod_pow_basic() {
        // 2^10 mod 1000 = 1024 mod 1000 = 24
        assert_eq!(
            result_of(
                req(r#"{"intent":"mod_pow","base":"2","exponent":"10","modulus":"1000"}"#)
                    .evaluate()
            ),
            "24"
        );
    }

    #[test]
    fn mod_pow_fermat_little_theorem() {
        // p=7 prime, a=3: 3^6 ≡ 1 (mod 7)
        assert_eq!(
            result_of(
                req(r#"{"intent":"mod_pow","base":"3","exponent":"6","modulus":"7"}"#).evaluate()
            ),
            "1"
        );
    }

    #[test]
    fn mod_pow_zero_exponent() {
        // a^0 mod m = 1 for any a, m > 0
        assert_eq!(
            result_of(
                req(r#"{"intent":"mod_pow","base":"999","exponent":"0","modulus":"7"}"#).evaluate()
            ),
            "1"
        );
    }

    #[test]
    fn mod_pow_large_exponent() {
        // 2^100 mod 1000 = last 3 digits of 2^100
        let resp =
            req(r#"{"intent":"mod_pow","base":"2","exponent":"100","modulus":"1000"}"#).evaluate();
        assert_eq!(result_of(resp), "376");
    }

    #[test]
    fn mod_pow_zero_modulus_errors() {
        let resp =
            req(r#"{"intent":"mod_pow","base":"2","exponent":"10","modulus":"0"}"#).evaluate();
        assert!(matches!(resp, NumberResponse::Error { .. }));
    }

    #[test]
    fn mod_pow_negative_base_odd_exponent() {
        // (-3)^1 mod 7: raw = 3, sign-adjust: euclidean_rem(-3, 7) = 4.
        // Mutation delete - at line 167: euclidean_rem(3, 7) = 3. Different.
        // Mutation && → || at line 164: fires on positive base too (checked below).
        assert_eq!(
            result_of(
                req(r#"{"intent":"mod_pow","base":"-3","exponent":"1","modulus":"7"}"#).evaluate()
            ),
            "4"
        );
    }

    #[test]
    fn mod_pow_negative_base_even_exponent() {
        // (-3)^2 mod 7 = 9 mod 7 = 2. No sign adjust because exp is even.
        // Mutation && → || at line 164: fires because base is negative → sign-adjusts wrongly → 5.
        // Mutation exp-parity == → != at line 164: fires for even → sign-adjusts → 5.
        assert_eq!(
            result_of(
                req(r#"{"intent":"mod_pow","base":"-3","exponent":"2","modulus":"7"}"#).evaluate()
            ),
            "2"
        );
    }

    #[test]
    fn mod_pow_positive_base_odd_exponent_no_sign_adjust() {
        // 3^3 mod 7 = 27 mod 7 = 6. No sign adjust because base is positive.
        // Mutation base.sign() == → != at line 164: fires for positive → sign-adjusts → euclidean_rem(-6,7)=1.
        assert_eq!(
            result_of(
                req(r#"{"intent":"mod_pow","base":"3","exponent":"3","modulus":"7"}"#).evaluate()
            ),
            "6"
        );
    }

    // ── ModInverse ───────────────────────────────────────────────────────────

    #[test]
    fn mod_inverse_basic() {
        // 3 * 4 = 12 ≡ 1 (mod 11)
        let inv = result_of(req(r#"{"intent":"mod_inverse","a":"3","modulus":"11"}"#).evaluate());
        assert_eq!(inv, "4");
    }

    #[test]
    fn mod_inverse_certifies_via_multiply() {
        // Verify returned value satisfies a * inv ≡ 1 (mod m)
        let inv: i64 =
            result_of(req(r#"{"intent":"mod_inverse","a":"7","modulus":"13"}"#).evaluate())
                .parse()
                .unwrap();
        assert_eq!((7 * inv) % 13, 1);
    }

    #[test]
    fn mod_inverse_no_inverse_when_not_coprime() {
        // gcd(2, 4) = 2 ≠ 1
        let resp = req(r#"{"intent":"mod_inverse","a":"2","modulus":"4"}"#).evaluate();
        match resp {
            NumberResponse::NoInverse { gcd, .. } => assert_eq!(gcd, "2"),
            other => panic!("expected NoInverse, got {other:?}"),
        }
    }

    #[test]
    fn mod_inverse_no_inverse_provides_gcd_evidence() {
        // gcd(6, 9) = 3
        let resp = req(r#"{"intent":"mod_inverse","a":"6","modulus":"9"}"#).evaluate();
        match resp {
            NumberResponse::NoInverse { gcd, .. } => assert_eq!(gcd, "3"),
            other => panic!("expected NoInverse, got {other:?}"),
        }
    }

    // ── IsPrime ───────────────────────────────────────────────────────────────

    #[test]
    fn is_prime_small_primes() {
        for n in &[2u32, 3, 5, 7, 11, 13, 97] {
            let resp = NumberRequest::IsPrime { n: n.to_string() }.evaluate();
            match resp {
                NumberResponse::Primality { prime, .. } => {
                    assert!(prime, "{n} should be prime")
                }
                other => panic!("expected Primality, got {other:?}"),
            }
        }
    }

    #[test]
    fn is_prime_composites() {
        for n in &[1u32, 4, 6, 8, 9, 15, 100, 561] {
            let resp = NumberRequest::IsPrime { n: n.to_string() }.evaluate();
            match resp {
                NumberResponse::Primality { prime, .. } => {
                    assert!(!prime, "{n} should not be prime")
                }
                other => panic!("expected Primality, got {other:?}"),
            }
        }
    }

    #[test]
    fn is_prime_large_prime() {
        // 2^31 - 1 = 2147483647 is a Mersenne prime
        let resp = NumberRequest::IsPrime {
            n: "2147483647".to_owned(),
        }
        .evaluate();
        match resp {
            NumberResponse::Primality { prime, .. } => assert!(prime),
            other => panic!("expected Primality, got {other:?}"),
        }
    }

    #[test]
    fn is_prime_carmichael_number_is_composite() {
        // 561 = 3 × 11 × 17 is the smallest Carmichael number (passes Fermat but not Miller-Rabin)
        let resp = NumberRequest::IsPrime {
            n: "561".to_owned(),
        }
        .evaluate();
        match resp {
            NumberResponse::Primality { prime, .. } => assert!(!prime),
            other => panic!("expected Primality, got {other:?}"),
        }
    }

    // ── PrimeFactors ─────────────────────────────────────────────────────────

    #[test]
    fn prime_factors_360() {
        // 360 = 2^3 × 3^2 × 5
        let resp = req(r#"{"intent":"prime_factors","n":"360"}"#).evaluate();
        match resp {
            NumberResponse::Factorization { factors, .. } => {
                assert_eq!(factors.len(), 3);
                assert_eq!(
                    factors[0],
                    PrimeFactor {
                        base: "2".to_owned(),
                        exponent: 3
                    }
                );
                assert_eq!(
                    factors[1],
                    PrimeFactor {
                        base: "3".to_owned(),
                        exponent: 2
                    }
                );
                assert_eq!(
                    factors[2],
                    PrimeFactor {
                        base: "5".to_owned(),
                        exponent: 1
                    }
                );
            }
            other => panic!("expected Factorization, got {other:?}"),
        }
    }

    #[test]
    fn prime_factors_prime_itself() {
        // 97 is prime → single factor with exponent 1
        let resp = req(r#"{"intent":"prime_factors","n":"97"}"#).evaluate();
        match resp {
            NumberResponse::Factorization { factors, .. } => {
                assert_eq!(factors.len(), 1);
                assert_eq!(
                    factors[0],
                    PrimeFactor {
                        base: "97".to_owned(),
                        exponent: 1
                    }
                );
            }
            other => panic!("expected Factorization, got {other:?}"),
        }
    }

    #[test]
    fn prime_factors_one_has_no_factors() {
        let resp = req(r#"{"intent":"prime_factors","n":"1"}"#).evaluate();
        match resp {
            NumberResponse::Factorization { factors, .. } => assert!(factors.is_empty()),
            other => panic!("expected Factorization, got {other:?}"),
        }
    }

    #[test]
    fn prime_factors_power_of_prime() {
        // 64 = 2^6
        let resp = req(r#"{"intent":"prime_factors","n":"64"}"#).evaluate();
        match resp {
            NumberResponse::Factorization { factors, .. } => {
                assert_eq!(factors.len(), 1);
                assert_eq!(
                    factors[0],
                    PrimeFactor {
                        base: "2".to_owned(),
                        exponent: 6
                    }
                );
            }
            other => panic!("expected Factorization, got {other:?}"),
        }
    }

    #[test]
    fn prime_factors_rejects_oversized_input() {
        // 2^65 exceeds 64-bit limit
        let n = "36893488147419103232"; // 2^65
        let resp = NumberRequest::PrimeFactors { n: n.to_owned() }.evaluate();
        assert!(matches!(resp, NumberResponse::Error { .. }));
    }

    #[test]
    fn prime_factors_at_max_bits_succeeds() {
        // 2^63 has bits()=64 exactly = MAX_FACTOR_BITS; > 64 is false → succeeds.
        // Mutation > → >=: 64 >= 64 = true → error.
        let resp = NumberRequest::PrimeFactors {
            n: "9223372036854775808".to_owned(), // 2^63
        }
        .evaluate();
        match resp {
            NumberResponse::Factorization { factors, .. } => {
                assert_eq!(factors.len(), 1);
                assert_eq!(
                    factors[0],
                    PrimeFactor {
                        base: "2".to_owned(),
                        exponent: 63
                    }
                );
            }
            other => panic!("expected Factorization, got {other:?}"),
        }
    }

    // ── CRT ───────────────────────────────────────────────────────────────────

    #[test]
    fn crt_basic() {
        // x ≡ 2 (mod 3), x ≡ 3 (mod 5), x ≡ 2 (mod 7) → x = 23 (mod 105)
        let resp =
            req(r#"{"intent":"crt","remainders":["2","3","2"],"moduli":["3","5","7"]}"#).evaluate();
        match resp {
            NumberResponse::CrtValue {
                result, modulus, ..
            } => {
                assert_eq!(result, "23");
                assert_eq!(modulus, "105");
            }
            other => panic!("expected CrtValue, got {other:?}"),
        }
    }

    #[test]
    fn crt_result_satisfies_all_congruences() {
        // x ≡ 1 (mod 2), x ≡ 2 (mod 3), x ≡ 3 (mod 5) → verify result
        let resp =
            req(r#"{"intent":"crt","remainders":["1","2","3"],"moduli":["2","3","5"]}"#).evaluate();
        match resp {
            NumberResponse::CrtValue {
                result, modulus, ..
            } => {
                let x: i64 = result.parse().unwrap();
                let m: i64 = modulus.parse().unwrap();
                assert_eq!(x % 2, 1);
                assert_eq!(x % 3, 2);
                assert_eq!(x % 5, 3);
                assert_eq!(m, 30);
            }
            other => panic!("expected CrtValue, got {other:?}"),
        }
    }

    #[test]
    fn crt_single_congruence_returns_remainder() {
        let resp = req(r#"{"intent":"crt","remainders":["5"],"moduli":["7"]}"#).evaluate();
        match resp {
            NumberResponse::CrtValue {
                result, modulus, ..
            } => {
                assert_eq!(result, "5");
                assert_eq!(modulus, "7");
            }
            other => panic!("expected CrtValue, got {other:?}"),
        }
    }

    #[test]
    fn crt_mismatched_lengths_errors() {
        let resp = req(r#"{"intent":"crt","remainders":["1","2"],"moduli":["3"]}"#).evaluate();
        assert!(matches!(resp, NumberResponse::Error { .. }));
    }

    #[test]
    fn crt_non_coprime_moduli() {
        // x ≡ 0 (mod 6), x ≡ 2 (mod 4). gcd(6,4)=2, compatible since 2|2.
        // Correct: x=6, modulus=12.
        // Mutation / → * at line 381: step = (6*2)*4 = 48 → modulus=48 (wrong).
        // Mutation / → * at line 382: t becomes wrong → x wrong.
        let resp = req(r#"{"intent":"crt","remainders":["0","2"],"moduli":["6","4"]}"#).evaluate();
        match resp {
            NumberResponse::CrtValue {
                result, modulus, ..
            } => {
                let x: i64 = result.parse().unwrap();
                let m: i64 = modulus.parse().unwrap();
                assert_eq!(m, 12, "modulus should be lcm(6,4)=12, got {m}");
                assert_eq!(x % 6, 0);
                assert_eq!(x % 4, 2);
            }
            other => panic!("expected CrtValue, got {other:?}"),
        }
    }

    #[test]
    fn number_schema_contains_expected_intents() {
        // Mutation: replace fn with Default::default() → returns empty object.
        let schema = number_schema_json();
        let one_of = schema["oneOf"].as_array().expect("oneOf must be present");
        assert!(!one_of.is_empty(), "schema must have oneOf variants");
        assert!(schema["$defs"]["Gcd"].is_object());
        assert!(schema["$defs"]["ModPow"].is_object());
        assert!(schema["$defs"]["Crt"].is_object());
    }
}
