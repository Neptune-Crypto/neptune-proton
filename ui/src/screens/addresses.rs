use std::net;
//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use crate::app_state::AppState;
use crate::components::address::Address;
use crate::components::qr_code::QrCode;
use crate::components::pico::{Button, ButtonType, Card, CopyButton, NoTitleModal};
use dioxus::prelude::*;
use neptune_types::address::{KeyType, ReceivingAddress};
use neptune_types::network::Network;
use std::rc::Rc;

/// A new, self-contained component for rendering a single row in the address table.
#[component]
fn AddressRow(
    address: Rc<ReceivingAddress>,
    on_qr_request: EventHandler<Rc<ReceivingAddress>>,
    network: Network,
) -> Element {
    // This component now manages its own hover and copied state locally.
    let mut is_hovered = use_signal(|| false);

    let key_type = KeyType::from(&*address);
    let key_type_str = key_type.to_string();

    rsx! {
        tr {
            // When the mouse leaves, we reset both hover and copied states.
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| {
                is_hovered.set(false);
            },

            td { "{key_type_str}" }
            td { Address { address: address.clone() } }

            td {
                style: "min-width: 150px; text-align: right;",
                if is_hovered() {
                    // Use Pico's `role="group"` for horizontal button layout.
                    div {
                        style: "font-size: 0.8em",
                        role: "group",
                        CopyButton {
                            text_to_copy: address.to_bech32m(network).unwrap()
                        }
                        Button {
                            button_type: ButtonType::Contrast,
                            outline: true,
                            on_click: move |_| {
                                on_qr_request.call(address.clone());
                            },
                            "QR"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AddressesScreen() -> Element {
    let network = use_context::<AppState>().network;

    let mut known_keys = use_resource(move || async move { api::known_keys().await });

    rsx! {
        match &*known_keys.read() {
            None => rsx! {
                Card {
                    h3 { "My Addresses" }
                    p { "Loading..." }
                    progress {}
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load addresses: {e}" }
                    button { onclick: move |_| known_keys.restart(), "Retry" }
                }
            },
            Some(Ok(keys)) => {
                let mut qr_code_content = use_signal::<Option<Rc<ReceivingAddress>>>(|| None);
                let mut qr_modal_is_open = use_signal(|| false);

                let addresses: Vec<_> = keys
                    .into_iter()
                    .rev()
                    .filter_map(|key| Some(key.to_address()))
                    .map(Rc::new)
                    .collect();

                rsx! {
                    NoTitleModal {
                        is_open: qr_modal_is_open,

                        if let Some(address) = qr_code_content() {
                            div {
                                style: "display: flex; flex-direction: column; align-items: center; text-align: center",

                                QrCode {
                                    data: address.to_bech32m(network).unwrap().to_uppercase(),
                                    caption: address.to_display_bech32m_abbreviated(network).unwrap(),
                                }
                            }
                        }
                    }
                    Card {
                        h3 { "My Addresses" }
                        table {
                            thead {
                                tr {
                                    th { "Type" }
                                    th { "Address" }
                                    th { style: "width: 1%;", "" }
                                }
                            }
                            tbody {
                                {addresses.into_iter().map(|address| {
                                    let full_address_for_key = address.to_bech32m(network).unwrap();
                                    rsx! {
                                        AddressRow {
                                            key: "{full_address_for_key}",
                                            address: Rc::clone(&address),
                                            network,
                                            on_qr_request: move |address: Rc<ReceivingAddress>| {
                                                qr_code_content.set(Some(address));
                                                qr_modal_is_open.set(true);
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}
