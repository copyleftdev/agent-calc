use agent_calc::{Expr, FinanceRequest, FinanceResponse};
use proptest::prelude::*;

fn integer(value: i32) -> Expr {
    Expr::Integer {
        value: value.to_string(),
    }
}

fn rational(numerator: i32, denominator: i32) -> Expr {
    Expr::Rational {
        numerator: numerator.to_string(),
        denominator: denominator.to_string(),
    }
}

fn exact_display(response: FinanceResponse) -> String {
    match response {
        FinanceResponse::Value { exact, .. } => exact.display,
        FinanceResponse::Error { reason, .. } => panic!("expected finance value: {reason}"),
        other => panic!("unexpected response variant: {other:?}"),
    }
}

proptest! {
    #[test]
    fn future_then_present_value_round_trips(
        principal in -10_000i32..10_000,
        rate_numerator in -9i32..20,
        rate_denominator in 1i32..20,
        periods in 0u32..8,
    ) {
        prop_assume!(rate_numerator != -rate_denominator || periods == 0);
        let rate = rational(rate_numerator, rate_denominator);
        let future_response = FinanceRequest::FutureValue {
            present_value: integer(principal),
            rate: rate.clone(),
            periods,
            decimal_places: 8,
        }.evaluate();

        let future = match future_response {
            FinanceResponse::Value { exact, .. } => Expr::Rational {
                numerator: exact.numerator,
                denominator: exact.denominator,
            },
            FinanceResponse::Error { reason, .. } => panic!("expected future value: {reason}"),
            other => panic!("unexpected response variant: {other:?}"),
        };

        let present = exact_display(FinanceRequest::PresentValue {
            future_value: future,
            rate,
            periods,
            decimal_places: 8,
        }.evaluate());

        prop_assert_eq!(present, principal.to_string());
    }

    #[test]
    fn zero_rate_npv_is_sum_of_cash_flows(
        a in -10_000i32..10_000,
        b in -10_000i32..10_000,
        c in -10_000i32..10_000,
    ) {
        let npv = exact_display(FinanceRequest::NetPresentValue {
            rate: integer(0),
            cash_flows: vec![integer(a), integer(b), integer(c)],
            decimal_places: 8,
        }.evaluate());

        prop_assert_eq!(npv, (a + b + c).to_string());
    }

    #[test]
    fn discount_factor_at_period_zero_is_one(rate_numerator in -100i32..100, rate_denominator in 1i32..100) {
        let discount = exact_display(FinanceRequest::DiscountFactor {
            rate: rational(rate_numerator, rate_denominator),
            period: 0,
            decimal_places: 8,
        }.evaluate());

        prop_assert_eq!(discount, "1");
    }
}
