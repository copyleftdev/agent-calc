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
