//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use crate::components::pico::Card;
use crate::components::pico::Modal;
use crate::components::pico::Grid;
use crate::components::pico::Button;
use crate::components::pico::ButtonType;
use dioxus::prelude::*;
use neptune_types::network::Network;
use neptune_types::address::AddressableKeyType;

#[allow(non_snake_case)]
#[component]
pub fn AddressesScreen() -> Element {

    let mut known_keys = use_resource(move || async move {
        // Your async API call goes here.
        api::known_keys().await
    });

    rsx! {

        match &*known_keys.read() {
            // The resource is still loading or has not been run yet.
            None => {
                rsx! {
                    Card {
                        h3 { "My Addresses" }
                        p { "Loading..." }
                        progress {} // An indeterminate progress bar
                    }
                }
            }

            // The async task returned an error.
            Some(Err(e)) => {
                rsx! {
                    Card {
                        h3 { "Error" }
                        p { "Failed to load addresses: {e}" }
                        // You could add a "Retry" button here
                        button {
                            onclick: move |_| known_keys.restart(),
                            "Retry"
                        }
                    }
                }
            }

            // The async task finished successfully.
            Some(Ok(keys)) => {
                // Signal to track which row index is currently being hovered.
                let mut hovered_index = use_signal::<Option<usize>>(|| None);

                // Signal to hold the QR code SVG string for the modal.
                let mut qr_code_content = use_signal::<Option<(String, String)>>(|| None);

                // CORRECTED: A dedicated signal to control the QR modal's visibility.
                let mut qr_modal_is_open = use_signal(|| false);

                // Signal to track which row's address was just copied.
                let mut copied_index = use_signal::<Option<usize>>(|| None);

                // Prepare the address data before rendering.
                let addresses: Vec<_> = keys
                    .into_iter()
                    .rev()
                    .filter_map(|key| key.to_address())
                    .collect();

                rsx! {
                    // Modal for displaying the QR Code. It only renders when `qr_code_content` has a value.
                    Modal {
                        is_open: qr_modal_is_open,
                        if let Some((addr_abbrev, svg_data)) = qr_code_content() {
                            h3 { "Address QR Code" }
                            div { dangerous_inner_html: "{svg_data}" }
                            p { dangerous_inner_html: "{addr_abbrev}" }
                        }
                    }

                    Card {
                        h3 { "My Addresses" }
                        table {
                            thead {
                                tr {
                                    th { "Type" }
                                    th { "Address" }
                                    th { style: "width: 1%;", "" } // Empty header for the actions column
                                }
                            }
                            tbody {
                                // Use `.iter().enumerate()` to get both the index and the address.
                                {addresses.into_iter().enumerate().map(|(i, address)| {
                                    let ktype = AddressableKeyType::from(&address).to_string();
                                    let addr_abbrev = address.to_display_bech32m_abbreviated(Network::Main).unwrap();
                                    let is_hovered = *hovered_index.read() == Some(i);
                                    let address_for_qr_button = address.clone();

                                    rsx! {
                                        // Set mouse enter/leave handlers on the table row.
                                        tr {
                                            onmouseenter: move |_| hovered_index.set(Some(i)),
                                            onmouseleave: move |_| hovered_index.set(None),

                                            td { "{ktype}" }
                                            td { code { "{addr_abbrev}" } }

                                            // Actions column, conditionally visible on hover.
                                            td {
                                                style: "min-width: 150px; text-align: right;", // Ensure buttons don't wrap
                                                if is_hovered {
                                                    // A grid to hold our two small buttons
                                                    Grid {
                                                        // Show "Copied!" message or the "Copy" button.
                                                        if *copied_index.read() == Some(i) {
                                                            Button {
                                                                button_type: ButtonType::Secondary,
                                                                disabled: true,
                                                                "Copied!"
                                                            }
                                                        } else {
                                                            Button {
                                                                button_type: ButtonType::Secondary,
                                                                outline: true,
                                                                on_click: move |_| {
                                                                    // Generate the full address ONLY when clicked.
                                                                    let full_address = address.to_bech32m(Network::Main).unwrap();

                                                                    // Use conditional compilation for clipboard logic
                                                                    #[cfg(target_arch = "wasm32")]
                                                                    {
                                                                        let window = web_sys::window().expect("should have a window in this context");
                                                                        let navigator = window.navigator();
                                                                        let clipboard = navigator.clipboard();
                                                                        let promise = clipboard.write_text(&full_address);
                                                                        wasm_bindgen_futures::spawn_local(async move {
                                                                            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                                                        });
                                                                    }

                                                                    #[cfg(not(target_arch = "wasm32"))]
                                                                    {
                                                                        // Placeholder for native clipboard logic
                                                                        // For a real desktop app, you'd use a crate like `arboard` here.
                                                                        eprintln!("Clipboard on native not implemented yet. Copied: {}", full_address);
                                                                    }

                                                                    // Show confirmation for this specific row.
                                                                    copied_index.set(Some(i));
                                                                },
                                                                "Copy"
                                                            }
                                                        }
                                                        Button {
                                                            button_type: ButtonType::Contrast,
                                                            outline: true,
                                                            on_click: move |_| {
                                                                // Generate the full address ONLY when clicked.
                                                                let full_address = address_for_qr_button.to_bech32m(Network::Main).unwrap();
                                                                let abbrev_address = address_for_qr_button.to_display_bech32m_abbreviated(Network::Main).unwrap();
                                                                // Generate the QR code data.
                                                                use qrcode::QrCode;
                                                                use qrcode::render::svg;
                                                                if let Ok(code) = QrCode::new(full_address.as_bytes()) {
                                                                    let image = code.render::<svg::Color>().build();
                                                                    // Set the signal to show the modal with the SVG content.
                                                                    qr_code_content.set(Some((abbrev_address, image)));
                                                                    qr_modal_is_open.set(true);
                                                                } else {
                                                                    dioxus_logger::tracing::warn!("QR code could not be created.  data len is: {}", full_address.as_bytes().len());
                                                                }
                                                            },
                                                            "QR"
                                                        }
                                                    }
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
