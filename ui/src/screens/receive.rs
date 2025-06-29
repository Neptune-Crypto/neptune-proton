//=============================================================================
// File: src/screens/receive.rs
//=============================================================================
use crate::components::pico::{Button, Card};
use dioxus::prelude::*;
use neptune_types::address::KeyType;
use neptune_types::address::ReceivingAddress;
use neptune_types::network::Network;
use std::rc::Rc;

#[component]
pub fn ReceiveScreen() -> Element {
    // A signal to hold the generated address. Starts as None.
    let mut receiving_address = use_signal::<Option<Rc<ReceivingAddress>>>(|| None);
    let mut is_generating = use_signal(|| false);

    rsx! {
        Card {
            h2 { "Receive Funds" }

            // Conditionally render the content based on whether an address has been generated.
            if let Some(address) = receiving_address() {
                // View to display AFTER an address has been generated
                div {
                    style: "text-align: center; padding-top: 1rem;",
                    p { "Share this address to receive funds." }

                    // QR Code Figure
                    figure {
                        style: "margin-top: 1rem; margin-bottom: 1rem;",
                        // Generate the QR code on the fly as an SVG string
                        {
                            use qrcode::QrCode;
                            use qrcode::render::svg;
                            let full_address = address.to_display_bech32m_abbreviated(Network::Main).unwrap();
                            let code = QrCode::new(full_address.as_bytes()).unwrap();
                            let image = code.render::<svg::Color>()
                                .min_dimensions(200, 200) // Ensure it's not too small
                                .build();
                            rsx!{ div { dangerous_inner_html: "{image}" } }
                        }
                    }

                    // Display the abbreviated address
                    code {
                        style: "word-break: break-all; font-size: 0.9rem;",
                        "{address.to_bech32m_abbreviated(Network::Main).unwrap()}"
                    }

                    // Copy button
                    div {
                        style: "margin-top: 1.5rem;",
                        Button {
                            on_click: move |_| {
                                let full_address = address.to_bech32m(Network::Main).unwrap();
                                #[cfg(target_arch = "wasm32")]
                                {
                                    if let Some(clipboard) = web_sys::window().and_then(|win| Some(win.navigator().clipboard())) {
                                        let promise = clipboard.write_text(&full_address);
                                        wasm_bindgen_futures::spawn_local(async move {
                                            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                        });
                                    }
                                }
                            },
                            "Copy"
                        }
                    }
                }
            } else {
                // Initial view, before an address has been generated
                div {
                    style: "text-align: center; padding: 2rem;",
                    Button {
                        disabled: is_generating(),
                        on_click: move |_| {
                            // Set a flag to disable the button while the async task runs
                            is_generating.set(true);
                            spawn({
                                let mut receiving_address = receiving_address.clone();
                                let mut is_generating = is_generating.clone();
                                async move {
                                    // This is where you would call your API.
                                    // For now, we simulate a delay and a result.
                                    // let new_addr = api::generate_new_address().await.unwrap();
                                    let new_addr = api::next_receiving_address(KeyType::Generation).await.unwrap();

                                    // Update the signal with the new address
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
