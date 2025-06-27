//=============================================================================
// File: src/screens/mempool.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[component]
pub fn MempoolScreen() -> Element {
    rsx! {
        Card {
            h3 { "Mempool" }
            p { "Awaiting confirmation:" }
            h4 { "3,450 transactions" }
        }
    }
}
