//=============================================================================
// File: src/screens/send.rs
//=============================================================================
use crate::components::pico::{Button, ButtonType, Card, Grid, Input, Modal};
use dioxus::prelude::*;

#[component]
pub fn SendScreen() -> Element {
    // Signal to control the confirmation modal's visibility.
    let mut is_confirm_modal_open = use_signal(|| false);

    rsx! {
        // This Modal component is now part of the SendScreen's layout.
        // It will only be visible when `is_confirm_modal_open` is true.
        Modal {
            is_open: is_confirm_modal_open,
            h2 { "Confirm Transaction" }
            p { "Please review the details below. This action cannot be undone." }
            footer {
                Button {
                    button_type: ButtonType::Secondary,
                    outline: true,
                    on_click: move |_| is_confirm_modal_open.set(false),
                    "Cancel"
                }
                Button {
                    on_click: move |_| {
                        // In a real app, the transaction broadcast logic would go here.
                        is_confirm_modal_open.set(false);
                    },
                    "Confirm & Send"
                }
            }
        }

        Card {
            h2 { "Send Funds" }
            // No longer a form, just a div for layout and capturing inputs.
            div {
                Input {
                    label: "Recipient Address".to_string(),
                    name: "address".to_string(),
                    placeholder: "nolga...".to_string()
                }
                Grid {
                    Input {
                        label: "Amount".to_string(),
                        name: "amount".to_string(),
                        input_type: "number".to_string(),
                        placeholder: "0.001".to_string()
                    }
                    Input {
                        label: "Fee (sats/vB)".to_string(),
                        name: "fee".to_string(),
                        input_type: "number".to_string(),
                        placeholder: "15".to_string()
                    }
                }
                // This button now opens the modal for confirmation.
                Button {
                    on_click: move |_| is_confirm_modal_open.set(true),
                    "Review Transaction"
                }
            }
        }
    }
}
