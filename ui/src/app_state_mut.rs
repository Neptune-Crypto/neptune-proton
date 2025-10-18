//! Defines the mutable, reactive state for the application's UI.

use api::fiat_currency::FiatCurrency;
use api::price_map::PriceMap;
use dioxus::prelude::*;

/// Represents the user's primary display currency choice.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DisplayCurrency {
    /// Display amounts in Neptune Cash (NPT).
    #[default]
    Npt,
    /// Display amounts in the specified fiat currency.
    Fiat(FiatCurrency),
}

/// A reactive state provided as a Dioxus context for mutable UI data.
///
/// This struct holds `Signal`s for any UI-related state that needs to change
/// and trigger automatic re-renders in the view. It is separate from the core,
/// immutable `AppState`.
#[derive(Clone, Copy)]
pub struct AppStateMut {
    /// A signal holding the latest fiat prices. `None` while loading.
    pub prices: Signal<Option<PriceMap>>,
    /// A signal holding the user's global currency display preference.
    pub display_currency: Signal<DisplayCurrency>,
}
