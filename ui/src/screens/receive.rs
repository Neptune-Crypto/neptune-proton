//=============================================================================
// File: src/screens/receive.rs
//=============================================================================
use crate::components::pico::{Button, ButtonType, Card, CopyButton};
use dioxus::prelude::*;
use neptune_types::address::KeyType;
use neptune_types::address::ReceivingAddress;
use neptune_types::network::Network;
use std::rc::Rc;

#[component]
pub fn ReceiveScreen() -> Element {
    let mut receiving_address = use_signal::<Option<Rc<ReceivingAddress>>>(|| None);
    let mut is_generating = use_signal(|| false);
    let mut selected_key_type = use_signal(|| KeyType::Generation);
    // **NEW**: A signal to track the acknowledgment checkbox.
    let mut symmetric_warning_acknowledged = use_signal(|| false);

    // Determine if the main generate button should be disabled.
    let generate_button_disabled = is_generating()
        || (selected_key_type() == KeyType::Symmetric && !symmetric_warning_acknowledged());

    rsx! {
        Card {
            h2 { "Receive Funds" }

            if let Some(address) = receiving_address() {
                // View to display AFTER an address has been generated
                div {
                    style: "text-align: center; padding-top: 1rem;",
                    if KeyType::from(&*address).is_symmetric() {
                        p{ strong { style: "color: var(--pico-color-red-500);", "Do not share with anyone."} }
                        p{ "This is a symmetric key/address." }
                        p{ "Anyone possessing it can spend associated funds." }
                    } else {
                        p{ "Share this address to receive funds." }
                    }

                    figure {
                        style: "margin-top: 1rem; margin-bottom: 1rem;",
                        {
                            use qrcode::QrCode;
                            use qrcode::render::svg;
                            let full_address = if KeyType::from(&*address).is_generation() {
                                address.to_display_bech32m_abbreviated(Network::Main).unwrap()
                            } else {
                                address.to_display_bech32m(Network::Main).unwrap()
                            };
                            match QrCode::new(full_address.as_bytes()) {
                                Ok(code) => {
                                    let image = code.render::<svg::Color>()
                                        .min_dimensions(200, 200)
                                        .build();
                                    rsx!{ div { dangerous_inner_html: "{image}" } }
                                },
                                Err(_) => rsx!{ p { style: "color: red;", "Address too long for QR code" } }
                            }
                        }
                    }
                    code {
                        style: "word-break: break-all; font-size: 0.9rem;",
                        "{address.to_bech32m_abbreviated(Network::Main).unwrap()}"
                    }
                    div {
                        style: "margin-top: 1.5rem; display: flex; justify-content: center; gap: 1rem;",
                        CopyButton {
                            text_to_copy: address.to_bech32m(Network::Main).unwrap()
                        }
                        Button {
                            button_type: ButtonType::Secondary,
                            on_click: move |_| {
                                receiving_address.set(None);
                                symmetric_warning_acknowledged.set(false); // Reset checkbox state
                            },
                            "Generate Another"
                        }
                    }
                }
            } else {
                // Initial view, before an address has been generated
                div {
                    style: "text-align: center; padding: 2rem;",

                    p { "Select Address Type:" }
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

                    // **NEW**: Conditionally render the warning and checkbox.
                    if selected_key_type() == KeyType::Symmetric {
                        div {
                            style: "max-width: 400px; margin: auto; margin-bottom: 1.5rem;",
                            fieldset {
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: "{symmetric_warning_acknowledged()}",
                                        oninput: move |evt| symmetric_warning_acknowledged.set(evt.value() == "true")
                                    }
                                    "I understand that symmetric keys must only be used for sending between wallets I control. Sharing with others would enable them to spend my funds."
                                }
                            }
                        }
                    }

                    Button {
                        // **MODIFIED**: Use the new disabled logic.
                        disabled: generate_button_disabled,
                        on_click: move |_| {
                            is_generating.set(true);
                            spawn({
                                let mut receiving_address = receiving_address.clone();
                                let mut is_generating = is_generating.clone();
                                let key_type_to_gen = selected_key_type.read().clone();
                                async move {
                                    let new_addr = api::next_receiving_address(key_type_to_gen).await.unwrap();
                                    receiving_address.set(Some(Rc::new(new_addr)));
                                    is_generating.set(false);
                                }
                            });
                        },
                        if is_generating() {
                            "Generating..."
                        } else {
                            "Generate New Receiving Address"
                        }
                    }
                }
            }
        }
    }
}
