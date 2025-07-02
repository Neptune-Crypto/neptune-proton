//=============================================================================
// File: src/components/address.rs
//=============================================================================
use crate::components::pico::{Button, CopyButton, Modal, NoTitleModal};
use crate::AppState;
use dioxus::prelude::*;
use neptune_types::address::ReceivingAddress;
use std::rc::Rc;

#[derive(Props, PartialEq, Clone)]
pub struct AddressProps {
    pub address: Rc<ReceivingAddress>,
}

#[component]
pub fn Address(props: AddressProps) -> Element {
    let network = use_context::<AppState>().network;
    let mut is_modal_open = use_signal(|| false);
    let address = props.address.clone();

    let abbreviated_address = use_memo(move || {
        address
            .to_display_bech32m_abbreviated(network)
            .unwrap_or_else(|_| "Invalid Address".to_string())
    });

    let full_address = use_memo(move || {
        props
            .address
            .to_bech32m(network)
            .unwrap_or_else(|_| "Invalid Address".to_string())
    });

    rsx! {
        // --- Modal to display the full address ---
        NoTitleModal {
            is_open: is_modal_open,
            div {
                style: "display: flex; flex-direction: column; gap: 1rem; align-items: center;",
                h4 { "Full Address" }
                div {
                    style: "align-self: flex-end;",
                    CopyButton { text_to_copy: full_address() }
                }
                code {
                    style: "word-break: break-all; background-color: var(--pico-muted-background-color); padding: 1rem; border-radius: var(--pico-border-radius);",
                    "{full_address}"
                }
                Button {
                    on_click: move |_| is_modal_open.set(false),
                    "Close"
                }
            }
        }

        // --- The clickable abbreviated address display ---
        div {
            style: "cursor: pointer;",
            title: "Click to view full address",
            onclick: move |_| is_modal_open.set(true),
            code { "{abbreviated_address}" }
        }
    }
}
