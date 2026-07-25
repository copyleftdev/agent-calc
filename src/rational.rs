use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rational {
    inner: BigRational,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RationalError {
    ZeroDenominator,
    InvalidInteger(String),
    InvalidDecimal(String),
}

impl fmt::Display for RationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RationalError::ZeroDenominator => write!(f, "denominator must not be zero"),
            RationalError::InvalidInteger(value) => write!(f, "invalid integer `{value}`"),
            RationalError::InvalidDecimal(value) => write!(f, "invalid decimal `{value}`"),
        }
    }
}

impl std::error::Error for RationalError {}

impl Rational {
    pub fn zero() -> Self {
        Self {
            inner: BigRational::zero(),
        }
    }

    pub fn one() -> Self {
        Self {
            inner: BigRational::one(),
        }
    }

    pub fn new<N, D>(numerator: N, denominator: D) -> Result<Self, RationalError>
    where
        N: Into<BigInt>,
        D: Into<BigInt>,
    {
        let numerator = numerator.into();
        let denominator = denominator.into();
        if denominator.is_zero() {
            return Err(RationalError::ZeroDenominator);
        }
        Ok(Self {
            inner: BigRational::new(numerator, denominator),
        })
    }

    pub fn integer<N>(value: N) -> Self
    where
        N: Into<BigInt>,
    {
        Self {
            inner: BigRational::from_integer(value.into()),
        }
    }

    pub fn parse_integer(value: &str) -> Result<Self, RationalError> {
        BigInt::from_str(value)
            .map(Self::integer)
            .map_err(|_| RationalError::InvalidInteger(value.to_owned()))
    }

    pub fn parse(numerator: &str, denominator: &str) -> Result<Self, RationalError> {
        let numerator = parse_bigint(numerator)?;
        let denominator = parse_bigint(denominator)?;
        Self::new(numerator, denominator)
    }

    /// Parse a fixed-point decimal string exactly, without an `f64` conversion.
    pub fn parse_decimal(value: &str) -> Result<Self, RationalError> {
        let (negative, unsigned) = match value.strip_prefix('-') {
            Some(unsigned) => (true, unsigned),
            None => (false, value),
        };
        let mut parts = unsigned.split('.');
        let whole = parts.next().unwrap_or_default();
        let fractional = parts.next();
        if parts.next().is_some()
            || whole.is_empty()
            || !whole.bytes().all(|byte| byte.is_ascii_digit())
            || fractional.is_some_and(|digits| {
                digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit())
            })
        {
            return Err(RationalError::InvalidDecimal(value.to_owned()));
        }

        let fractional = fractional.unwrap_or_default();
        let digits = format!("{whole}{fractional}");
        let mut numerator = BigInt::from_str(&digits)
            .map_err(|_| RationalError::InvalidDecimal(value.to_owned()))?;
        if negative {
            numerator = -numerator;
        }
        let denominator = BigInt::from(10u8).pow(fractional.len() as u32);
        Self::new(numerator, denominator)
    }

    pub fn numerator(&self) -> &BigInt {
        self.inner.numer()
    }

    pub fn denominator(&self) -> &BigInt {
        self.inner.denom()
    }

    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    pub fn checked_add(&self, rhs: &Self) -> Result<Self, RationalError> {
        Ok(Self {
            inner: &self.inner + &rhs.inner,
        })
    }

    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, RationalError> {
        Ok(Self {
            inner: &self.inner - &rhs.inner,
        })
    }

    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, RationalError> {
        Ok(Self {
            inner: &self.inner * &rhs.inner,
        })
    }

    pub fn checked_div(&self, rhs: &Self) -> Result<Self, RationalError> {
        if rhs.is_zero() {
            return Err(RationalError::ZeroDenominator);
        }
        Ok(Self {
            inner: &self.inner / &rhs.inner,
        })
    }

    pub fn checked_neg(&self) -> Result<Self, RationalError> {
        Ok(Self {
            inner: -&self.inner,
        })
    }

    pub fn checked_pow_i32(&self, exponent: i32) -> Result<Self, RationalError> {
        if exponent < 0 && self.is_zero() {
            return Err(RationalError::ZeroDenominator);
        }
        Ok(Self {
            inner: self.inner.pow(exponent),
        })
    }

    pub fn is_negative(&self) -> bool {
        self.inner.numer().is_negative()
    }

    pub fn abs(&self) -> Self {
        if self.is_negative() {
            Self {
                inner: -&self.inner,
            }
        } else {
            self.clone()
        }
    }

    pub fn floor(&self) -> Self {
        let n = self.inner.numer();
        let d = self.inner.denom();
        Self::integer(floor_bigint(n, d))
    }

    pub fn ceil(&self) -> Self {
        let n = self.inner.numer();
        let d = self.inner.denom();
        Self::integer(ceil_bigint(n, d))
    }

    pub fn round(&self) -> Self {
        let n = self.inner.numer();
        let d = self.inner.denom();
        let two_n_plus_d = BigInt::from(2i32) * n + d;
        let two_d = BigInt::from(2i32) * d;
        Self::integer(floor_bigint(&two_n_plus_d, &two_d))
    }

    pub fn to_f64(&self) -> f64 {
        self.inner.to_f64().unwrap_or(f64::NAN)
    }

    pub fn exact_sqrt(&self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }
        let n = self.inner.numer();
        let d = self.inner.denom();
        let sqrt_n = exact_integer_sqrt(n)?;
        let sqrt_d = exact_integer_sqrt(d)?;
        Self::new(sqrt_n, sqrt_d).ok()
    }

    pub fn decimal_string(&self, places: usize) -> String {
        let sign = if self.numerator().is_negative() {
            "-"
        } else {
            ""
        };
        let n = self.numerator().abs();
        let d = self.denominator();
        let whole = &n / d;
        let mut rem = n % d;
        if places == 0 {
            return format!("{sign}{whole}");
        }
        let mut frac = String::with_capacity(places);
        for _ in 0..places {
            rem *= 10;
            let digit = (&rem / d)
                .to_u8()
                .expect("decimal digit after remainder scaling must fit in u8");
            frac.push(char::from(b'0' + digit));
            rem %= d;
        }
        format!("{sign}{whole}.{frac}")
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator().is_one() {
            write!(f, "{}", self.numerator())
        } else {
            write!(f, "{}/{}", self.numerator(), self.denominator())
        }
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(&rhs)
            .expect("rational addition should be infallible")
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(&rhs)
            .expect("rational subtraction should be infallible")
    }
}

impl Mul for Rational {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(&rhs)
            .expect("rational multiplication should be infallible")
    }
}

impl Div for Rational {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(&rhs).expect("rational division failed")
    }
}

impl Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.checked_neg()
            .expect("rational negation should be infallible")
    }
}

fn parse_bigint(value: &str) -> Result<BigInt, RationalError> {
    BigInt::from_str(value).map_err(|_| RationalError::InvalidInteger(value.to_owned()))
}

fn floor_bigint(n: &BigInt, d: &BigInt) -> BigInt {
    let q = n / d;
    let r = n % d;
    if r.is_negative() {
        q - BigInt::one()
    } else {
        q
    }
}

fn ceil_bigint(n: &BigInt, d: &BigInt) -> BigInt {
    let q = n / d;
    let r = n % d;
    if r.is_positive() {
        q + BigInt::one()
    } else {
        q
    }
}

fn exact_integer_sqrt(n: &BigInt) -> Option<BigInt> {
    if n.is_zero() {
        return Some(BigInt::zero());
    }
    if n.is_negative() {
        return None;
    }
    let two = BigInt::from(2i32);
    let mut x = n / &two + BigInt::one();
    loop {
        let next = (&x + n / &x) / &two;
        if next >= x {
            return if &x * &x == *n { Some(x) } else { None };
        }
        x = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_sign_and_gcd() {
        assert_eq!(Rational::new(6, -8).unwrap().to_string(), "-3/4");
        assert_eq!(Rational::new(-6, -8).unwrap().to_string(), "3/4");
        assert_eq!(Rational::new(0, -8).unwrap(), Rational::zero());
    }

    #[test]
    fn parses_fixed_point_decimals_exactly() {
        assert_eq!(
            Rational::parse_decimal("123.45").unwrap().to_string(),
            "2469/20"
        );
        assert_eq!(
            Rational::parse_decimal("-0.0825").unwrap().to_string(),
            "-33/400"
        );
        assert_eq!(Rational::parse_decimal("100").unwrap().to_string(), "100");
        assert!(Rational::parse_decimal(".5").is_err());
        assert!(Rational::parse_decimal("1.").is_err());
        assert!(Rational::parse_decimal("1e2").is_err());
    }

    #[test]
    fn renders_decimal_with_requested_places() {
        assert_eq!(Rational::zero().decimal_string(3), "0.000");
        assert_eq!(Rational::new(1, 2).unwrap().decimal_string(0), "0");
        assert_eq!(Rational::new(1, 2).unwrap().decimal_string(3), "0.500");
        assert_eq!(Rational::new(-22, 7).unwrap().decimal_string(4), "-3.1428");
    }

    #[test]
    fn reports_zero_and_ordering() {
        assert!(Rational::zero().is_zero());
        assert!(!Rational::one().is_zero());
        assert!(Rational::new(1, 3).unwrap() < Rational::new(1, 2).unwrap());
        assert!(Rational::new(-1, 3).unwrap() > Rational::new(-1, 2).unwrap());
        assert!(Rational::new(3, 2).unwrap() > Rational::new(4, 3).unwrap());
        assert_eq!(
            Rational::new(1, 3)
                .unwrap()
                .partial_cmp(&Rational::new(1, 2).unwrap()),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn error_messages_are_specific() {
        assert_eq!(
            Rational::new(1, 0).unwrap_err().to_string(),
            "denominator must not be zero"
        );
        assert_eq!(
            Rational::parse_integer("nope").unwrap_err().to_string(),
            "invalid integer `nope`"
        );
    }

    #[test]
    fn supports_integers_larger_than_i128() {
        let huge = "170141183460469231731687303715884105728";
        let value = Rational::parse(huge, "2").unwrap();
        assert_eq!(value.to_string(), "85070591730234615865843651857942052864");
    }

    #[test]
    fn round_uses_correct_half_up_formula() {
        // 7/4 = 1.75 → 2; kills all 6 mutations on the two_n_plus_d / two_d lines
        assert_eq!(Rational::new(7, 4).unwrap().round().to_string(), "2");
        // 5/4 = 1.25 → 1 (additional value to disambiguate * vs + at col 47)
        assert_eq!(Rational::new(5, 4).unwrap().round().to_string(), "1");
    }

    #[test]
    fn raises_to_integer_powers_exactly() {
        assert_eq!(
            Rational::new(2, 3)
                .unwrap()
                .checked_pow_i32(3)
                .unwrap()
                .to_string(),
            "8/27"
        );
        assert_eq!(
            Rational::new(2, 3)
                .unwrap()
                .checked_pow_i32(-2)
                .unwrap()
                .to_string(),
            "9/4"
        );
        assert_eq!(
            Rational::new(0, 3)
                .unwrap()
                .checked_pow_i32(0)
                .unwrap()
                .to_string(),
            "1"
        );
        assert_eq!(
            Rational::zero()
                .checked_pow_i32(-1)
                .unwrap_err()
                .to_string(),
            "denominator must not be zero"
        );
    }
}
