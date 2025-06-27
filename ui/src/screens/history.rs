//=============================================================================
// File: src/screens/history.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn HistoryScreen() -> Element {
    rsx! {
        Card {
            h3 { "History" }
            table {
                thead { tr {
                    th { "Date" }
                    th { "Type" }
                    th { "Amount" }
                    th { "TXID" }
                }}
                tbody {
                    tr {
                        td { "2025-06-14" }
                        td { "Sent" }
                        td { "-0.01 NPT" }
                        td { code { "a1b2c3d4..." } }
                    }
                    tr {
                        td { "2025-06-12" }
                        td { "Received" }
                        td { "+0.05 NPT" }
                        td { code { "e5f6g7h8..." } }
                    }
                }
            }
        }
    }
}
