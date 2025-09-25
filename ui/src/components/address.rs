//=============================================================================
// File: src/components/address.rs
//=============================================================================
use crate::components::pico::{Button, CopyButton, NoTitleModal};
use crate::components::qr_code::QrCode;
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

        NoTitleModal {
            is_open: is_modal_open,
            div {
                style: "display: flex; flex-direction: column; align-items: center; text-align: center",

                QrCode {
                    data: full_address().to_uppercase(),
                    caption: "Scan the QR code to obtain the full address.".to_string()
                }

                // This flex container will center the buttons and add a gap between them.
                div {
                    style: "display: flex; justify-content: center; gap: 0.5rem;",

                    CopyButton { text_to_copy: full_address() }
                    Button {
                        on_click: move |_| is_modal_open.set(false),
                        "Close"
                    }
                }
                h4 {
                    style: "margin-top: 1rem; margin-bottom: 0rem;",
                    "Full Address"
                }
                code {
                    style: "text-align: left; word-break: break-all; background-color: var(--pico-muted-background-color); padding: 1rem; border-radius: var(--pico-border-radius); width: 100%; margin-bottom: 1rem;", // Gap after the code block
                    "{full_address}"
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
