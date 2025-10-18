//! A component for displaying currency amounts with a toggle-on-hover feature.

use crate::app_state_mut::{AppStateMut, DisplayCurrency};
use api::{fiat_amount::FiatAmount, fiat_currency::FiatCurrency};
use dioxus::prelude::*;
use neptune_types::native_currency_amount::NativeCurrencyAmount;

/// A component that displays a currency amount and flips to an alternative
/// currency on hover or tap-and-hold. It now accepts an optional `fiat_equivalent`
/// to ensure precision for display values and is fully reactive to prop changes.
#[component]
pub fn Amount(
    amount: NativeCurrencyAmount,
    #[props(optional)] fiat_equivalent: Option<FiatAmount>,
) -> Element {
    let app_state_mut = use_context::<AppStateMut>();
    let mut is_flipped = use_signal(|| false);

    // **MODIFICATION**: Removed use_memo. This logic now runs on every render,
    // ensuring the component reacts instantly to changes in its `amount` prop.
    let prices = app_state_mut.prices.read();
    let primary_currency = *app_state_mut.display_currency.read();

    let (main_currency, alternate_currency) = if is_flipped() {
        match primary_currency {
            DisplayCurrency::Npt => (
                DisplayCurrency::Fiat(FiatCurrency::USD),
                DisplayCurrency::Npt,
            ),
            DisplayCurrency::Fiat(_) => (DisplayCurrency::Npt, primary_currency),
        }
    } else {
        match primary_currency {
            DisplayCurrency::Npt => (
                DisplayCurrency::Npt,
                DisplayCurrency::Fiat(FiatCurrency::USD),
            ),
            DisplayCurrency::Fiat(_) => (primary_currency, DisplayCurrency::Npt),
        }
    };

    // Helper function to calculate the fiat value using precise integer math.
    let calculate_fiat_fallback = |amt: NativeCurrencyAmount, price: FiatAmount| -> FiatAmount {
        let npt_minor_units = amt.to_nau();
        let price_minor_units = price.as_minor_units() as i128;
        let conversion_factor = NativeCurrencyAmount::coins(1).to_nau();
        let final_fiat_minor_units = (npt_minor_units * price_minor_units) / conversion_factor;
        FiatAmount::new_from_minor(final_fiat_minor_units as i64, price.currency())
    };

    // Helper function to format an amount based on the currency type.
    let format_currency = |amt: NativeCurrencyAmount, currency: DisplayCurrency| -> String {
        match currency {
            DisplayCurrency::Npt => format!("{} NPT", amt),
            DisplayCurrency::Fiat(fiat) => {
                if let Some(fiat_val) = fiat_equivalent {
                    return fiat_val.to_string_with_symbol();
                }
                if let Some(price_map) = &*prices {
                    if let Some(price) = price_map.get(fiat) {
                        return calculate_fiat_fallback(amt, price).to_string_with_symbol();
                    }
                }
                format!("{} NPT", amt)
            }
        }
    };

    // Helper function for the tooltip text, with a specific "unavailable" message.
    let format_tooltip = |amt: NativeCurrencyAmount, currency: DisplayCurrency| -> String {
        match currency {
            DisplayCurrency::Npt => format!("{} NPT", amt),
            DisplayCurrency::Fiat(fiat) => {
                if let Some(fiat_val) = fiat_equivalent {
                    return fiat_val.to_string_with_symbol();
                }
                if let Some(price_map) = &*prices {
                    if let Some(price) = price_map.get(fiat) {
                        return calculate_fiat_fallback(amt, price).to_string_with_symbol();
                    }
                }
                format!("{} Price Unavailable", fiat.code())
            }
        }
    };

    let (main_text, tooltip_text) = (
        format_currency(amount, main_currency),
        format_tooltip(amount, alternate_currency),
    );

    rsx! {
        span {
            onmouseenter: move |_| is_flipped.set(true),
            onmouseleave: move |_| is_flipped.set(false),
            ontouchstart: move |_| is_flipped.set(true),
            ontouchend: move |_| is_flipped.set(false),
            oncontextmenu: move |e| e.stop_propagation(),

            title: "{tooltip_text}",
            cursor: "pointer",
            "{main_text}"
        }
    }
}

