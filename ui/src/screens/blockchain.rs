//=============================================================================
// File: src/screens/blockchain.rs
//=============================================================================
use crate::components::pico::{Card, Grid};
use dioxus::prelude::*;

#[component]
pub fn BlockChainScreen() -> Element {
    rsx! {
        Card {
            h2 { "Blockchain Status" }
            Grid {
                div {
                    h4 { "Current Block Height" }
                    p { "847,921" }
                }
                div {
                    h4 { "Sync Status" }
                    p { "100% (Synced)" }
                }
            }
        }
    }
}
