use std::net;
//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use std::rc::Rc;
use dioxus::prelude::*;
use crate::components::pico::{Button, ButtonType, Card, CopyButton, NoTitleModal};
use crate::components::address::Address;
use neptune_types::address::{KeyType, ReceivingAddress};
use neptune_types::network::Network;
use crate::app_state::AppState;


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

    let mut known_keys = use_resource(move || async move {
        api::known_keys().await
    });


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
                let mut qr_code_content = use_signal::<Option<(String, String)>>(|| None);
                let mut qr_modal_is_open = use_signal(|| false);

                let addresses: Vec<_> = keys
                    .into_iter()
                    .rev()
                    .filter_map(|key| key.to_address())
                    .map(Rc::new)
                    .collect();

                rsx! {
                    NoTitleModal {
                        is_open: qr_modal_is_open,
                        div {
                            style: "text-align: center;",
                            if let Some((addr_abbrev, svg_data)) = qr_code_content() {
                                h3 { "Receiving Address" },
                                div { dangerous_inner_html: "{svg_data}" }
                                p { "{addr_abbrev}" }
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
                                                let abbrev_address = address.to_bech32m_abbreviated(network).unwrap();
                                                let full_address = if KeyType::from(&*address).is_generation() {
                                                    abbrev_address.clone()
                                                } else {
                                                     address.to_bech32m(network).unwrap()
                                                };

                                                use qrcode::QrCode;
                                                use qrcode::render::svg;
                                                if let Ok(code) = QrCode::new(full_address.as_bytes()) {
                                                    let image = code.render::<svg::Color>().build();
                                                    qr_code_content.set(Some((abbrev_address, image)));
                                                    qr_modal_is_open.set(true);
                                                } else {
                                                    dioxus_logger::tracing::warn!("QR code could not be created.  data len is: {}", full_address.as_bytes().len());
                                                }
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
