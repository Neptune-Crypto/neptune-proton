//=============================================================================
// File: src/screens/receive.rs
//=============================================================================
use crate::components::pico::{Button, Card};
use dioxus::prelude::*;

#[component]
pub fn ReceiveScreen() -> Element {
    rsx! {
        Card {
            h2 { "Receive Address" }
            p { "Share this address to receive funds." }
            figure {
                div {
                    style: "width: 150px; height: 150px; background: #eee; margin: auto;",
                }
                figcaption {
                    "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"
                }
            }
            Button { "Copy Address" }
        }
    }
}
