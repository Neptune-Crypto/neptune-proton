//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use std::rc::Rc;

use dioxus::prelude::*;
use neptune_types::address::KeyType;
use neptune_types::address::ReceivingAddress;
use neptune_types::network::Network;

use crate::app_state::AppState;
use crate::components::address::Address;
use crate::components::empty_state::EmptyState;
use crate::components::pico::Button;
use crate::components::pico::ButtonType;
use crate::components::pico::Card;
use crate::components::pico::CopyButton;
use crate::components::pico::NoTitleModal;
use crate::components::qr_code::QrCode;
use crate::hooks::use_rpc_checker::use_rpc_checker;

// Embed the SVG content as a static string at compile time.
const ADDRESSES_EMPTY_SVG: &str = include_str!("../../assets/svg/addresses-empty.svg");

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

            // No explicit width needed, the single table layout handles it.
            td {


                "{key_type_str}"
            }

            td {


                Address {
                    address: address.clone(),
                    on_click: move |_| is_hovered.set(false),
                }
            }

            // Restore original style with min-width for the button group.
            td {
                style: "min-width: 150px; display: flex; align-items: center; justify-content: flex-end;",

                div {
                    style: {
                        format!(
                            "visibility: {}; margin: 0; font-size: 0.75em; --pico-form-element-spacing-vertical: 0.2rem; --pico-form-element-spacing-horizontal: 0.5rem;",
                            if is_hovered() { "visible" } else { "hidden" },
                        )
                    },
                    role: "group",
                    CopyButton {
                        text_to_copy: address.to_bech32m(network).unwrap(),
                    }
                    Button {
                        button_type: ButtonType::Contrast,
                        outline: true,
                        on_click: move |_| {
                            is_hovered.set(false);
                            on_qr_request.call(address.clone());
                        },
                        "QR"
                    }
                }
            }
        }
    }
}

#[component]
pub fn AddressesScreen() -> Element {
    let network = use_context::<AppState>().network;
    let mut rpc = use_rpc_checker(); // Initialize Hook

    let mut known_keys = use_resource(move || async move { api::known_keys().await });

    // Effect: Restarts the resource when connection is restored.
    let status_sig = rpc.status();
    use_effect(move || {
        if status_sig.read().is_connected() {
            known_keys.restart();
        }
    });

    rsx! {
        match &*known_keys.read() {
            None => rsx! {
                Card {

                    h3 {
                        "My Addresses"
                    }
                    p {

                        "Loading..."
                    }
                    progress {


                    }
                }
            },
            Some(result) if !rpc.check_result_ref(&result) => rsx! {
                Card {
                    h3 {
                        "My Addresses"
                    }
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 {
                        "Error"
                    }
                    p {

                        "Failed to load addresses: {e}"
                    }
                    button {
                        onclick: move |_| known_keys.restart(),
                        "Retry"
                    }
                }
            },
            Some(Ok(keys)) if keys.is_empty() => {
                rsx! {
                    Card {

                        h3 {

                            "My Addresses"
                        }
                        EmptyState {
                            title: "No Addresses Found".to_string(),
                            description: Some("Visit the Receive tab to generate a new address.".to_string()),
                            icon: rsx! {
                                // Inject the SVG string directly into the DOM
                                span {
                                    dangerous_inner_html: ADDRESSES_EMPTY_SVG,
                                    style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                                }
                            }
                        }
                    }
                }
            }
            Some(Ok(keys)) => {
                let mut qr_code_content = use_signal::<
                    Option<Rc<ReceivingAddress>>,
                >(|| None);
                let mut qr_modal_is_open = use_signal(|| false);
                let addresses: Vec<_> = keys
                    .iter()
                    .map(|key| key.to_address())
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

                        h3 {

                            "My Addresses"
                        }
                        // This div is the scrollable container for the table.
                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {

                                thead {

                                    tr {

                                        // The 'th' elements are now sticky to the top of the scrollable container.
                                        th {
                                            style: "position: sticky; top: 0; background: var(--pico-card-background-color);",
                                            "Type"
                                        }
                                        th {
                                            style: "position: sticky; top: 0; background: var(--pico-card-background-color);",
                                            "Address"
                                        }
                                        th {
                                            style: "position: sticky; top: 0; background: var(--pico-card-background-color); width: 1%;",
                                            ""
                                        }
                                    }
                                }
                                tbody {

                                    {
                                        addresses
                                            .into_iter()
                                            .map(|address| {
                                                let full_address_for_key = address.to_bech32m(network).unwrap();
                                                rsx! {
                                                    AddressRow {
                                                        key: "{full_address_for_key}",
                                                        address: Rc::clone(&address),
                                                        network,
                                                        on_qr_request: move |address: Rc<ReceivingAddress>| {
                                                            qr_code_content.set(Some(address));
                                                            qr_modal_is_open.set(true);
                                                        },
                                                    }
                                                }
                                            })
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
