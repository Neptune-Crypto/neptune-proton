//=============================================================================
// File: src/screens/blockchain.rs
//=============================================================================
use crate::components::pico::{Card, Grid};
use neptune_types::block_selector::BlockSelector;
use crate::Screen;
use dioxus::prelude::*;

#[component]
pub fn BlockChainScreen() -> Element {
    // 1. `use_resource` takes an async block that will be run in the background.
    //    It immediately returns a `Resource` signal.
    let mut height_resource = use_resource(move || async move {
        // Your async API call goes here.
        api::block_height().await
    });
    let mut active_screen = use_context::<Signal<Screen>>();

    rsx! {
        // 2. The `rsx!` macro reads the current state of the `balance` signal.
        match &*height_resource.read() {
            // The resource is still loading or has not been run yet.
            None => {
                rsx! {
                    Card {
                        h3 { "Blockchain" }
                        p { "Loading..." }
                        progress {} // An indeterminate progress bar
                    }
                }
            }
            // The async task finished successfully.
            Some(Ok(height)) => {
                let owned_height = *height;
                rsx! {
                    Card {
                        h3 { "Blockchain" }
                        Grid {
                            div {
                                h4 { "Current Block Height" }
                                a {
                                    href: "#",
                                    onclick: move |_| {
                                        active_screen.set(Screen::Block(BlockSelector::Height(owned_height)));
                                    },
                                    "{height}"
                                }
                            }
                            div {
                                h4 { "Sync Status" }
                                p { "100% (Synced)" }
                            }
                        }
                    }
                }
            }
            // The async task returned an error.
            Some(Err(e)) => {
                rsx! {
                    Card {
                        h3 { "Error" }
                        p { "Failed to load: {e}" }
                        // You could add a "Retry" button here
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
