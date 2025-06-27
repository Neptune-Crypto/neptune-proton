//=============================================================================
// File: src/screens/receive.rs
//=============================================================================
use crate::components::pico::{Button, Card};
use dioxus::prelude::*;

#[component]
pub fn ReceiveScreen() -> Element {
    rsx! {
        Card {
            h3 { "Receive Address" }
            p { "Share this address to receive funds." }
            figure {
                div {
                    style: "width: 150px; height: 150px; background: #eee; margin: auto;",
                }
                figcaption {
                    "nolgaqxy2k...hx0wlh"
                }
            }
            Button { "Copy Address" }
        }
    }
}
