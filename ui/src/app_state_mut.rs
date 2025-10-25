//! Defines the mutable, reactive state for the application's UI.

use api::prefs::display_preference::DisplayPreference;
use api::price_map::PriceMap;
use dioxus::prelude::*;

/// A reactive state provided as a Dioxus context for mutable UI data.
///
/// This struct holds `Signal`s for any UI-related state that needs to change
/// and trigger automatic re-renders in the view. It is separate from the core,
/// immutable `AppState`.
#[derive(Clone, Copy)]
pub struct AppStateMut {
    /// A signal holding the latest fiat prices. `None` while loading.
    pub prices: Signal<Option<PriceMap>>,

    /// A single signal to manage the user's complete currency display preference.
    pub display_preference: Signal<DisplayPreference>,
}
