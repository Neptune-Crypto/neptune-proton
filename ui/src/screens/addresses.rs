//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[component]
pub fn AddressesScreen() -> Element {
    rsx! {
        Card {
            h2 { "My Addresses" }
            p { "List of generated addresses." }
            ul {
                li { code { "bc1qxy2k...fjhx0wlh" } }
                li { code { "bc1pavad...xrq32zch" } }
                li { code { "bc1q9zvl...2s73h2e6" } }
            }
        }
    }
}
