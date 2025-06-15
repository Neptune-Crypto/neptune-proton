//=============================================================================
// File: src/screens/balance.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[component]
pub fn BalanceScreen() -> Element {
    rsx! {
        Card {
            h2 { "Current Balance" }
            h1 { "0.0042 NPT" }
            p { "$281,400.65 USD" }
        }
        Card {
            h3 { "Asset Allocation" }
            progress { value: "75", max: "100" }
        }
    }
}
