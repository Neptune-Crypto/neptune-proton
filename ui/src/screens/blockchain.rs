//=============================================================================
// File: src/screens/blockchain.rs
//=============================================================================
use crate::components::pico::Card;
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::block_selector::BlockSelector;
use twenty_first::prelude::Digest;

#[component]
pub fn BlockChainScreen() -> Element {
    let mut height_resource = use_resource(move || async move { api::block_height().await });
    let mut active_screen = use_context::<Signal<Screen>>();

    // Signal to hold the value of the text input
    let mut lookup_input = use_signal(String::new);

    rsx! {
        match &*height_resource.read() {
            None => {
                rsx! {
                    Card {
                        h3 { "Blockchain" }
                        p { "Loading..." }
                        progress {}
                    }
                }
            }
            Some(Ok(height)) => {
                let owned_height = *height;
                rsx! {
                    Card {
                        h3 { "Blockchain" }
                        h4 { "Current Block Height" }
                        a {
                            href: "#",
                            onclick: move |_| {
                                active_screen.set(Screen::Block(BlockSelector::Height(owned_height)));
                            },
                            "{height}"
                        }
                    }

                    // New card for looking up a block
                    Card {
                        h4 { "Block Lookup" }
                        p { "Provide a block height (number) or digest (hex string) to look up a block." }

                        form {
                            prevent_default: "onsubmit",
                            onsubmit: move |_| {
                                let input_str = lookup_input.read().trim().to_string();
                                if input_str.is_empty() {
                                    return;
                                }

                                // Try to parse as a height, otherwise fall back to parsing as a digest.
                                let selector = if let Ok(h) = input_str.parse::<u64>() {
                                    Some(BlockSelector::Height(h.into()))
                                } else if let Ok(d) = Digest::try_from_hex(&input_str) {
                                    Some(BlockSelector::Digest(d))
                                } else {
                                    dioxus_logger::tracing::warn!("Invalid block selector input: {}", input_str);
                                    None
                                };

                                if let Some(s) = selector {
                                    active_screen.set(Screen::Block(s));
                                }
                            },
                            // Use Pico's group role for a compact input/button layout
                            div {
                                role: "group",
                                input {
                                    r#type: "text",
                                    placeholder: "Enter block height or digest",
                                    oninput: move |event| lookup_input.set(event.value())
                                }
                                button { r#type: "submit", "Lookup" }
                            }
                        }

                        div {
                            style: "margin-top: 1rem;",
                            "Quick Lookup: "
                            a {
                                href: "#",
                                onclick: move |_| active_screen.set(Screen::Block(BlockSelector::Genesis)),
                                "Genesis Block"
                            }
                            " | "
                            a {
                                href: "#",
                                onclick: move |_| active_screen.set(Screen::Block(BlockSelector::Tip)),
                                "Tip"
                            }
                        }
                    }
                }
            }
            Some(Err(e)) => {
                rsx! {
                    Card {
                        h3 { "Error" }
                        p { "Failed to load: {e}" }
                        button {
                            onclick: move |_| height_resource.restart(),
                            "Retry"
                        }
                    }
                }
            }
        }
    }
}