use agent_calc::{
    DecimalRounding, FinanceRequest, FinanceResponse, PriceValue, protocol::ExactRational,
};
use rug::Float;

const REFERENCE_PRECISION_BITS: u32 = 256;

fn decimal(value: &str) -> PriceValue {
    PriceValue::Decimal(value.to_owned())
}

fn high_precision_decimal(value: &str) -> Float {
    Float::with_val(
        REFERENCE_PRECISION_BITS,
        Float::parse(value).expect("valid reference decimal"),
    )
}

fn exact_rational_to_float(value: &ExactRational) -> Float {
    let numerator = high_precision_decimal(&value.numerator);
    let denominator = high_precision_decimal(&value.denominator);
    Float::with_val(REFERENCE_PRECISION_BITS, numerator / denominator)
}

#[test]
fn discounted_cash_flow_matches_independent_256_bit_calculation() {
    let response = FinanceRequest::DiscountedCashFlow {
        cash_flows: vec![decimal("100.00"), decimal("110.00"), decimal("121.00")],
        discount_rate: decimal("0.10"),
        terminal_growth_rate: Some(decimal("0.03")),
        decimal_places: 8,
        rounding_mode: DecimalRounding::HalfEven,
    }
    .evaluate();

    let actual = match response {
        FinanceResponse::PriceModel { model, .. } => exact_rational_to_float(&model.price),
        other => panic!("expected price model response, got {other:?}"),
    };

    let one = high_precision_decimal("1");
    let discount_rate = high_precision_decimal("0.10");
    let terminal_growth_rate = high_precision_decimal("0.03");
    let growth_factor = Float::with_val(REFERENCE_PRECISION_BITS, &one + &discount_rate);
    let mut discount_power = Float::with_val(REFERENCE_PRECISION_BITS, 1);
    let mut expected = Float::with_val(REFERENCE_PRECISION_BITS, 0);
    for cash_flow in ["100.00", "110.00", "121.00"] {
        discount_power *= &growth_factor;
        expected += Float::with_val(
            REFERENCE_PRECISION_BITS,
            high_precision_decimal(cash_flow) / &discount_power,
        );
    }

    let next_cash_flow = Float::with_val(
        REFERENCE_PRECISION_BITS,
        high_precision_decimal("121.00")
            * Float::with_val(REFERENCE_PRECISION_BITS, &one + &terminal_growth_rate),
    );
    let terminal_value = Float::with_val(
        REFERENCE_PRECISION_BITS,
        next_cash_flow
            / Float::with_val(
                REFERENCE_PRECISION_BITS,
                &discount_rate - &terminal_growth_rate,
            ),
    );
    expected += Float::with_val(REFERENCE_PRECISION_BITS, terminal_value / discount_power);

    let mut difference = actual - expected;
    difference.abs_mut();
    assert!(
        difference < high_precision_decimal("1e-60"),
        "256-bit DCF difference was {difference}"
    );
}
