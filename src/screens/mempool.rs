//=============================================================================
// File: src/screens/mempool.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[component]
pub fn MempoolScreen() -> Element {
    rsx! {
        Card {
            h2 { "Mempool Status" }
            p { "Awaiting confirmation:" }
            h3 { "3,450 transactions" }
        }
    }
}
