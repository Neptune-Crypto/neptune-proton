// In src/screens/send.rs

use crate::components::pico::{Button, ButtonType, Card, Grid, Input, Modal};
use dioxus::prelude::*;

#[component]
pub fn SendScreen() -> Element {
    // A signal to control the confirmation modal's visibility.
    let mut is_confirm_modal_open = use_signal(|| false);

    // **THE RECOMMENDED PATTERN**
    // 1. A signal to hold the result of our API call. It starts as None.
    let mut api_response = use_signal::<Option<String>>(|| None);

    rsx! {
        // This Modal component is now part of the SendScreen's layout.
        // It will only be visible when `is_confirm_modal_open` is true.
        Modal {
            is_open: is_confirm_modal_open,
            title: "Confirm Transaction",
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
                        is_confirm_modal_open.set(false);
                        // Clear any previous API response before making a new call.
                        api_response.set(None);

                        // 2. Spawn an async task to call the API.
                        spawn({
                            // Clone the signal so we can move it into the async block.
                            let mut api_response = api_response.clone();
                            async move {
                                // In a real app, you would gather data from the input fields.
                                // For now, we call the API with a placeholder message.
                                let result = api::echo("Transaction Sent!".to_string()).await;

                                // Format the result for display, handling both success and error.
                                let message = match result {
                                    Ok(msg) => format!("Success: {}", msg),
                                    Err(err) => format!("API Error: {}", err),
                                };

                                // 3. When the API call completes, update the signal.
                                //    Dioxus will automatically re-render any part of the
                                //    UI that reads this signal.
                                api_response.set(Some(message));
                            }
                        });
                    },
                    "Confirm & Send"
                }
            }
        }

        Card {
            h3 { "Send Funds" }
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

        // 4. Conditionally render the API response.
        //    This part of the UI will automatically update when the `api_response` signal changes.
        if let Some(response) = api_response() {
            Card {
                h3 { "Transaction Status" }
                p { "{response}" }
            }
        }
    }
}
