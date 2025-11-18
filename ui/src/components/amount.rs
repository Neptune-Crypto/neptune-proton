//! A component for displaying currency amounts with a toggle-on-hover feature.

use crate::app_state_mut::AppStateMut;
use api::prefs::display_preference::DisplayPreference;
use api::fiat_amount::FiatAmount;
use dioxus::prelude::*;
use neptune_types::native_currency_amount::NativeCurrencyAmount;

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum AmountType {
    #[default]
    Npt,
    Fiat,
    Current,
}

#[allow(dead_code)] // (Amount callers may use in the future)
#[derive(PartialEq, Clone, Copy, Debug, Default, strum::EnumIs)]
pub enum CurrencyFormat {
    Bare,
    Symbol,
    #[default]
    Code,
    SymbolAndCode,
}

impl CurrencyFormat {
    pub fn show_symbol(&self) -> bool {
        self.is_symbol() || self.is_symbol_and_code()
    }

    pub fn show_code(&self) -> bool {
        self.is_code() || self.is_symbol_and_code()
    }
}


/// A component that displays a currency amount and flips to an alternative
/// currency on hover or tap-and-hold. It now accepts an optional `fiat_equivalent`
/// to ensure precision for display values and is fully reactive to prop changes.
#[component]
#[allow(clippy::if_same_then_else)]
pub fn Amount(
    amount: NativeCurrencyAmount,
    #[props(optional)] fiat_equivalent: Option<FiatAmount>,
    #[props(optional)] fixed: Option<AmountType>,
    #[props(default)] format: CurrencyFormat,
) -> Element {
    let app_state_mut = use_context::<AppStateMut>();
    let mut is_flipped = use_signal(|| false);

    let prices = app_state_mut.prices.read();
    let preference = *app_state_mut.display_preference.read();

    // Derive display currencies from the new preference enum.
    let (main_currency_str, fiat_for_display) = match preference {
        DisplayPreference::NptOnly => ("NPT".to_string(), None),
        DisplayPreference::FiatEnabled {
            fiat,
            display_as_fiat,
            ..
        } => {
            match fixed {
                Some(AmountType::Npt) => ("NPT".to_string(), Some(fiat)),
                Some(AmountType::Fiat) => (fiat.code().to_string(), Some(fiat)),
                _ => {
                    let current_display_is_fiat = if is_flipped() {
                        !display_as_fiat
                    } else {
                        display_as_fiat
                    };
                    if current_display_is_fiat {
                        (fiat.code().to_string(), Some(fiat))
                    } else {
                        ("NPT".to_string(), Some(fiat))
                    }
                }
            }
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

    let format_npt = |amt: NativeCurrencyAmount| -> String {
        format!(
            "{}{}{}",
            // no NPT symbol exists yet afaik.  maybe one day.
            if format.show_symbol() { "" } else { "" },
            amt,
            if format.show_code() { " NPT" } else { "" },
        )
    };

    let format_fiat = |amt: FiatAmount| -> String {
        format!(
            "{}{}{}",
            if format.show_symbol() { amt.currency().symbol() } else { "" },
            amt,
            if format.show_code() { " ".to_owned() + amt.currency().code() } else { "".to_owned() },
        )
    };

    // Helper function to format an amount based on the currency string.
    let format_currency = |amt: NativeCurrencyAmount, currency_str: &str| -> String {
        if currency_str != "NPT" {
            if let Some(fc) = fiat_for_display {
                if let Some(fiat_val) = fiat_equivalent {
                    return format_fiat(fiat_val);
                }
                if let Some(price_map) = &*prices {
                    if let Some(price) = price_map.get(fc) {
                        let fiat_val = calculate_fiat_fallback(amt, price);
                        return format_fiat(fiat_val);
                    }
                }
            }
        }

        // Otherwise, NPT.
        format_npt(amt)
    };

    // It always shows the lossless amount. If fiat mode is enabled, it ALWAYS
    // shows the exchange rate, regardless of the currently displayed currency.
    let format_tooltip = |amt: NativeCurrencyAmount| -> String {
        let lossless_part = format!("{} NPT", amt.display_lossless());

        // Step 1: Check if fiat mode is enabled. If not, we're done.
        let currency_for_rate = match preference {
            DisplayPreference::FiatEnabled {
                fiat: global_fiat, ..
            } => {
                // Step 2: Prioritize the currency from the `fiat_equivalent` prop,
                // then fall back to the globally selected one.
                fiat_equivalent
                    .map(|fe| fe.currency())
                    .unwrap_or(global_fiat)
            }
            DisplayPreference::NptOnly => {
                // If in NPT-only mode, only ever show the lossless amount.
                return lossless_part;
            }
        };

        if let Some(price_map) = &*prices {
            if let Some(price) = price_map.get(currency_for_rate) {
                let rate_part = format!("1 NPT = {}", price.to_string_with_code());
                let amt_part = if let Some(fiat_amt) = fiat_equivalent {
                    fiat_amt.to_string_with_code()
                } else {
                    calculate_fiat_fallback(amt, price).to_string_with_code()
                };

                return format!("{}\n\n{}\n\n{}", lossless_part, amt_part, rate_part);
            }
        }

        // Fallback if price is not found for the specific currency
        lossless_part
    };

    let main_text = format_currency(amount, &main_currency_str);
    let tooltip_text = format_tooltip(amount);

    // Conditionally render based on whether fiat mode is enabled.
    if matches!(preference, DisplayPreference::FiatEnabled { .. }) {
        let true_if_flip = fixed.is_none();
        rsx! {
            span {
                onmouseenter: move |_| is_flipped.set(true_if_flip),
                onmouseleave: move |_| is_flipped.set(false),
                ontouchstart: move |_| is_flipped.set(true_if_flip),
                ontouchend: move |_| is_flipped.set(false),
                oncontextmenu: move |e| e.stop_propagation(),

                title: "{tooltip_text}",
                cursor: "pointer",
                "{main_text}"
            }
        }
    } else {
        // In NPT-only mode, render a simple span without hover effects.
        rsx! {
            span {
                title: "{tooltip_text}",
                "{main_text}"
            }
        }
    }
}
