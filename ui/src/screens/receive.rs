//=============================================================================
// File: src/screens/receive.rs
//=============================================================================
use std::rc::Rc;

use dioxus::prelude::*;
use neptune_types::address::KeyType;
use neptune_types::address::ReceivingAddress;
use serde::{Deserialize, Serialize}; // Needed for GenerationTask serialization

use crate::app_state::AppState;
use crate::components::pico::Button;
use crate::components::pico::ButtonType;
use crate::components::pico::Card;
use crate::components::pico::CopyButton;
use crate::components::qr_code::QrCode;
use crate::hooks::use_rpc_checker::use_rpc_checker;
use crate::hooks::use_rpc_checker::NeptuneRpcConnectionStatus;
use crate::ConnectionModal;

/// Helper structure to hold the parameters needed to generate a receiving address.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct GenerationTask {
    key_type: KeyType,
}

// Consolidated function that performs the actual RPC call, reusable by the button click and the watchdog.
async fn run_generation_task(task: GenerationTask) -> Result<ReceivingAddress, api::ApiError> {
    api::next_receiving_address(task.key_type).await
}

#[component]
pub fn ReceiveScreen() -> Element {
    let network = use_context::<AppState>().network;
    let rpc = use_rpc_checker(); // Initialize hook to track global connection status

    let mut receiving_address = use_signal::<Option<Rc<ReceivingAddress>>>(|| None);
    let mut is_generating = use_signal(|| false);
    let mut selected_key_type = use_signal(|| KeyType::Generation);
    let mut symmetric_warning_acknowledged = use_signal(|| false);

    // 1. Signal to store the pending Task for retry.
    let mut pending_task = use_signal::<Option<GenerationTask>>(|| None);

    // 2. Watchdog: Watches connection status and runs the pending task if possible.
    use_effect(move || {
        // Dependencies
        let binding = rpc.status();
        let status_read = binding.read();
        let task_option = *pending_task.read(); // Read the GenerationTask option
        let generating = is_generating();

        // Condition: Connection is restored AND we have a pending generation task
        // AND we are not already running a generation task (the spawned loop handles the delay).
        if *status_read == NeptuneRpcConnectionStatus::Connected
            && task_option.is_some()
            && !generating
        {
            // Set generating state
            is_generating.set(true);

            // Execute the saved generation task in a self-delaying loop
            spawn({
                let mut receiving_address = receiving_address;
                let mut is_generating = is_generating;
                let mut pending_task = pending_task;
                let mut rpc = rpc; // Capture immutable rpc

                async move {
                    if let Some(task) = task_option {
                        loop {
                            // Attempt the RPC call using the consolidated function
                            let new_addr_result = run_generation_task(task).await;

                            // Check Result and update global status
                            if rpc.check_result_ref(&new_addr_result) {
                                // SUCCESS path (or non-network error): Break the loop.
                                if let Ok(new_addr) = new_addr_result {
                                    receiving_address.set(Some(Rc::new(new_addr)));
                                }
                                break;
                            } else {
                                // RPC Check FAILED (Connection Lost/Refused)
                                // Schedule a delay within this task before trying again.
                                crate::compat::sleep(std::time::Duration::from_secs(3)).await;
                            }
                        }
                    } // End if let Some(task)

                    // Cleanup on successful generation/break:
                    pending_task.set(None);
                    is_generating.set(false);
                }
            });
        }
    });

    // Determine if the main generate button should be disabled.
    let generate_button_disabled = is_generating()
        || pending_task().is_some() // Task is pending retry
        || (selected_key_type() == KeyType::Symmetric && !symmetric_warning_acknowledged())
        || rpc.status().read().is_disconnected();

    rsx! {
        // Render the ConnectionModal based on global state
        ConnectionModal {}

        Card {
            h2 {
                "Receive Funds"
            }

            if let Some(address) = receiving_address() {
                // View to display AFTER an address has been generated
                div {
                    style: "text-align: center; padding-top: 1rem;",
                    if KeyType::from(&*address).is_symmetric() {
                        p {
                            strong {
                                style: "color: var(--pico-color-red-500);",
                                "Do not share with anyone."
                            }
                        }
                        p {
                            "This is a symmetric key/address."
                        }
                        p {
                            "Anyone possessing it can spend associated funds."
                        }
                    } else {
                        p {
                            "Share this address to receive funds."
                        }
                    }

                    QrCode {
                        data: address.to_display_bech32m(network).unwrap().to_uppercase(),
                        caption: "Scan the QR code to obtain the full address.".to_string(),
                    }

                    code {
                        style: "word-break: break-all; font-size: 0.9rem;",
                        "{address.to_bech32m_abbreviated(network).unwrap()}"
                    }
                    div {
                        style: "margin-top: 1.5rem; display: flex; justify-content: center; gap: 1rem;",
                        CopyButton {
                            text_to_copy: address.to_bech32m(network).unwrap(),
                        }
                        Button {
                            button_type: ButtonType::Secondary,
                            on_click: move |_| {
                                receiving_address.set(None);
                                symmetric_warning_acknowledged.set(false);
                            },
                            "Generate Another"
                        }
                    }
                }
            } else {
                // Initial view, before an address has been generated
                div {
                    style: "text-align: center; padding: 2rem;",

                    if pending_task().is_some() {
                        p {
                            strong {
                                style: "color: var(--pico-del-color);",
                                "Connection Lost. Retrying when connection is restored..."
                            }
                        }
                    }

                    p {
                        "Select Address Type:"
                    }
                    div {
                        style: "display: flex; justify-content: center; gap: 1rem; margin-bottom: 1.5rem;",
                        Button {
                            button_type: ButtonType::Secondary,
                            outline: selected_key_type() != KeyType::Generation,
                            on_click: move |_| selected_key_type.set(KeyType::Generation),
                            "Generation"
                        }
                        Button {
                            button_type: ButtonType::Secondary,
                            outline: selected_key_type() != KeyType::Symmetric,
                            on_click: move |_| selected_key_type.set(KeyType::Symmetric),
                            "Symmetric Key"
                        }
                    }

                    if selected_key_type() == KeyType::Symmetric {
                        div {
                            style: "max-width: 400px; margin: auto; margin-bottom: 1.5rem;",
                            fieldset {
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: "{symmetric_warning_acknowledged()}",
                                        oninput: move |evt| symmetric_warning_acknowledged.set(evt.value() == "true"),
                                    }
                                    "I understand that symmetric keys must only be used for sending between wallets I control. Sharing with others would enable them to spend my funds."
                                }
                            }
                        }
                    }

                    Button {
                        disabled: generate_button_disabled,
                        on_click: move |_| {
                            let task_to_gen = GenerationTask {
                                key_type: *selected_key_type.read()
                            };
                            is_generating.set(true);
                            pending_task.set(None); // Clear any old pending tasks

                            spawn({
                                let mut receiving_address = receiving_address;
                                let mut is_generating = is_generating;
                                let mut pending_task = pending_task;
                                let mut rpc = rpc;
                                async move {
                                    let new_addr_result = run_generation_task(task_to_gen).await; // CONSOLIDATED CALL

                                    // Check Result and update global status
                                    if rpc.check_result_ref(&new_addr_result) {
                                        // RPC Check passed (Connection is OK)
                                        if let Ok(new_addr) = new_addr_result {
                                            receiving_address.set(Some(Rc::new(new_addr)));
                                        }
                                    } else {
                                        // RPC Check FAILED (Connection Lost/Refused)
                                        // Save the task for the Watchdog (use_effect)
                                        pending_task.set(Some(task_to_gen));
                                    }
                                    is_generating.set(false);
                                }
                            });
                        },
                        if is_generating() {
                            "Generating..."
                        } else if pending_task().is_some() {
                             "Pending Retry..."
                        } else {
                            "Generate New Receiving Address"
                        }
                    }
                }
            }
        }
    }
}
