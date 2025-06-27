//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn AddressesScreen() -> Element {
    rsx! {
        Card {
            h3 { "My Addresses" }
            p { "List of generated addresses." }
            ul {
                li { code { "nolgaqxy2k...fjhx0wlh" } }
                li { code { "nolgab4vad...xrq32zch" } }
                li { code { "nolgaq9zvl...2s73h2e6" } }
            }
        }
    }
}
