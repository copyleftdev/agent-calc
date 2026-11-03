use agent_calc::Rational;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use proptest::prelude::*;

fn rational_strategy() -> impl Strategy<Value = Rational> {
    (-1_000_000i128..=1_000_000, 1i128..=1_000_000).prop_map(|(n, d)| Rational::new(n, d).unwrap())
}

proptest! {
    #[test]
    fn addition_is_commutative(a in rational_strategy(), b in rational_strategy()) {
        prop_assert_eq!(a.clone() + b.clone(), b + a);
    }

    #[test]
    fn multiplication_is_commutative(a in rational_strategy(), b in rational_strategy()) {
        prop_assert_eq!(a.clone() * b.clone(), b * a);
    }

    #[test]
    fn addition_has_identity(a in rational_strategy()) {
        prop_assert_eq!(a.clone() + Rational::zero(), a.clone());
        prop_assert_eq!(Rational::zero() + a.clone(), a);
    }

    #[test]
    fn multiplication_has_identity(a in rational_strategy()) {
        prop_assert_eq!(a.clone() * Rational::one(), a.clone());
        prop_assert_eq!(Rational::one() * a.clone(), a);
    }

    #[test]
    fn subtraction_is_addition_of_negation(a in rational_strategy(), b in rational_strategy()) {
        prop_assert_eq!(a.clone() - b.clone(), a + (-b));
    }

    #[test]
    fn division_is_multiplication_by_inverse(
        a in rational_strategy(),
        b in rational_strategy().prop_filter("non-zero divisor", |b| !b.is_zero())
    ) {
        let inverse = Rational::new(b.denominator().clone(), b.numerator().clone()).unwrap();
        prop_assert_eq!(a.clone() / b, a * inverse);
    }

    #[test]
    fn rational_is_always_canonical(a in rational_strategy()) {
        prop_assert!(a.denominator() > &BigInt::zero());
        if a.numerator().is_zero() {
            prop_assert_eq!(a.denominator(), &BigInt::one());
        }
    }
}

// --- round: explicit tie rules ---

use agent_calc::RoundingMode;

const ALL_MODES: [RoundingMode; 4] = [
    RoundingMode::HalfCeiling,
    RoundingMode::HalfAwayFromZero,
    RoundingMode::HalfEven,
    RoundingMode::TowardZero,
];

const NEAREST_MODES: [RoundingMode; 3] = [
    RoundingMode::HalfCeiling,
    RoundingMode::HalfAwayFromZero,
    RoundingMode::HalfEven,
];

fn mode_strategy() -> impl Strategy<Value = RoundingMode> {
    proptest::sample::select(ALL_MODES.to_vec())
}

fn nearest_mode_strategy() -> impl Strategy<Value = RoundingMode> {
    proptest::sample::select(NEAREST_MODES.to_vec())
}

/// Exactly halfway between two integers: (2k + 1) / 2.
fn tie_strategy() -> impl Strategy<Value = Rational> {
    (-1_000_000i128..=1_000_000).prop_map(|k| Rational::new(2 * k + 1, 2).unwrap())
}

fn half() -> Rational {
    Rational::new(1, 2).unwrap()
}

fn is_integer(r: &Rational) -> bool {
    r.denominator() == &BigInt::one()
}

proptest! {
    #[test]
    fn round_always_returns_an_integer(a in rational_strategy(), mode in mode_strategy()) {
        prop_assert!(is_integer(&a.round_with(mode)));
    }

    #[test]
    fn round_of_an_integer_is_identity(n in -1_000_000i128..=1_000_000, mode in mode_strategy()) {
        let a = Rational::integer(n);
        prop_assert_eq!(a.round_with(mode), a);
    }

    #[test]
    fn nearest_modes_stay_within_one_half(a in rational_strategy(), mode in nearest_mode_strategy()) {
        let distance = (a.round_with(mode) - a).abs();
        prop_assert!(distance <= half());
    }

    #[test]
    fn nearest_modes_agree_off_ties(a in rational_strategy()) {
        let fraction = a.clone() - a.floor();
        prop_assume!(fraction != half());
        let reference = a.round_with(RoundingMode::HalfCeiling);
        for mode in NEAREST_MODES {
            prop_assert_eq!(a.round_with(mode), reference.clone());
        }
    }

    #[test]
    fn default_round_is_half_ceiling_and_matches_floor_of_x_plus_half(a in rational_strategy()) {
        prop_assert_eq!(RoundingMode::default(), RoundingMode::HalfCeiling);
        prop_assert_eq!(a.round(), (a.clone() + half()).floor());
        prop_assert_eq!(a.round(), a.round_with(RoundingMode::HalfCeiling));
    }

    #[test]
    fn half_ceiling_sends_ties_up(t in tie_strategy()) {
        prop_assert_eq!(t.round_with(RoundingMode::HalfCeiling), t.clone() + half());
    }

    #[test]
    fn half_away_from_zero_is_odd_symmetric(a in rational_strategy()) {
        let m = RoundingMode::HalfAwayFromZero;
        prop_assert_eq!((-a.clone()).round_with(m), -a.round_with(m));
    }

    #[test]
    fn half_away_from_zero_sends_ties_outward(t in tie_strategy()) {
        let r = t.round_with(RoundingMode::HalfAwayFromZero);
        prop_assert!(r.abs() > t.abs());
        prop_assert_eq!((r - t).abs(), half());
    }

    #[test]
    fn half_even_sends_ties_to_an_even_integer(t in tie_strategy()) {
        let r = t.round_with(RoundingMode::HalfEven);
        prop_assert!((r.numerator() % BigInt::from(2)).is_zero());
        prop_assert_eq!((r - t).abs(), half());
    }

    #[test]
    fn half_even_is_odd_symmetric(a in rational_strategy()) {
        let m = RoundingMode::HalfEven;
        prop_assert_eq!((-a.clone()).round_with(m), -a.round_with(m));
    }

    #[test]
    fn toward_zero_truncates(a in rational_strategy()) {
        let r = a.round_with(RoundingMode::TowardZero);
        prop_assert!(r.abs() <= a.abs());
        prop_assert!((a.clone() - r.clone()).abs() < Rational::one());
        prop_assert_eq!((-a.clone()).round_with(RoundingMode::TowardZero), -r);
    }

    /// Dyadic rationals n / 2^k are exact in binary64, so the f64 rule must
    /// give the same integer as the exact rule, ties included.
    #[test]
    fn f64_rule_matches_exact_rule_on_dyadic_values(
        n in -(1i64 << 40)..=(1i64 << 40),
        k in 0u32..=12,
        mode in mode_strategy(),
    ) {
        let d = 1i64 << k;
        let exact = Rational::new(n, d).unwrap().round_with(mode);
        let approx = mode.round_f64(n as f64 / d as f64);
        prop_assert_eq!(Rational::integer(approx as i64), exact);
    }

    #[test]
    fn f64_rule_matches_exact_rule_on_ties(t in tie_strategy(), mode in mode_strategy()) {
        let exact = t.round_with(mode);
        let approx = mode.round_f64(t.to_f64());
        prop_assert_eq!(Rational::integer(approx as i64), exact);
    }
}
