// ui/src/currency.rs
use api::fiat_amount::FiatAmount;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

/// Converts an NPT amount to a fiat amount using a given exchange rate.
/// Uses high-precision math to avoid floating-point errors.
pub fn npt_to_fiat(amount: &NativeCurrencyAmount, rate: &FiatAmount) -> FiatAmount {
    if rate.as_minor_units() == 0 {
        return FiatAmount::new_from_minor(0, rate.currency());
    }
    let npt_scaling_factor = NativeCurrencyAmount::coins(1).to_nau();
    let nau_big = BigInt::from(amount.to_nau());
    let rate_minor_big = BigInt::from(rate.as_minor_units());
    let scaling_factor_big = BigInt::from(npt_scaling_factor);

    let product = nau_big * rate_minor_big;
    let fiat_smallest_units_big = product / scaling_factor_big;
    let fiat_smallest_units = fiat_smallest_units_big.to_i64().unwrap_or(i64::MAX);

    FiatAmount::new_from_minor(fiat_smallest_units, rate.currency())
}

/// Converts a fiat amount to an NPT amount using a given exchange rate.
/// Uses high-precision math and returns an error if the rate is zero or the result overflows.
pub fn fiat_to_npt(
    fiat_amount: &FiatAmount,
    rate: &FiatAmount,
) -> Result<NativeCurrencyAmount, &'static str> {
    if rate.as_minor_units() == 0 {
        return Err("Exchange rate is zero.");
    }
    let npt_scaling_factor = NativeCurrencyAmount::coins(1).to_nau();
    let fiat_minor_big = BigInt::from(fiat_amount.as_minor_units());
    let scaling_factor_big = BigInt::from(npt_scaling_factor);
    let rate_minor_big = BigInt::from(rate.as_minor_units());

    if rate_minor_big.is_zero() {
        return Err("Exchange rate is zero.");
    }

    let product = fiat_minor_big * scaling_factor_big;
    let nau_big = product / rate_minor_big;
    if let Some(nau) = nau_big.to_i128() {
        Ok(NativeCurrencyAmount::from_nau(nau))
    } else {
        Err("Exceeds maximum NPT supply of 42,000,000")
    }
}
