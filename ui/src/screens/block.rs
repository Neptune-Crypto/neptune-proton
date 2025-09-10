// src/screens/block.rs
use crate::components::pico::{Card, CopyButton};
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::{
    block_info::BlockInfo, block_selector::BlockSelector,
    native_currency_amount::NativeCurrencyAmount,
};
use twenty_first::tip5::Digest;

/// A small helper component to display a Digest with a label and copy button.
#[component]
fn DigestDisplay(digest: Digest, label: String, is_link: bool) -> Element {
    let mut active_screen = use_context::<Signal<Screen>>();
    let digest_str = digest.to_string();
    let abbreviated_digest = format!(
        "{}...{}",
        &digest_str[0..6],
        &digest_str[digest_str.len() - 4..]
    );

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; padding: 0.25rem 0; flex-wrap: wrap; gap: 0.5rem 1rem;",
            strong { "{label}:" }
            div {
                style: "display: flex; align-items: center; gap: 0.5rem;",
                if is_link {
                    a {
                        href: "#",
                        title: "{digest_str}",
                        onclick: move |_| {
                            active_screen.set(Screen::Block(BlockSelector::Digest(digest)));
                        },
                        "{abbreviated_digest}"
                    }
                } else {
                    code { title: "{digest_str}", "{abbreviated_digest}" }
                }
                CopyButton { text_to_copy: digest_str }
            }
        }
    }
}

#[component]
pub fn BlockScreen(selector: BlockSelector) -> Element {
    let mut block_resource = use_resource(move || async move {
        // The selector needs to be clonable to be passed into the async block
        api::block_info(selector.clone()).await
    });

    rsx! {
        match &*block_resource.read() {
            None => rsx! {
                Card {
                    h3 { "View Block" }
                    p { "Loading block details..." }
                    progress {}
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load block data: {e}" }
                    button { onclick: move |_| block_resource.restart(), "Retry" }
                }
            },
            Some(Ok(None)) => rsx! {
                 Card {
                    h3 { "Block Not Found" }
                    p { "The requested block was not found in the blockchain." }
                }
            },
            Some(Ok(Some(info))) => rsx! {
                Card {
                    h3 { "Block Details" }
                    h5 {
                        style: "text-align: center; margin-bottom: 1.5rem;",
                        "Height: {info.height}"
                    }

                    DigestDisplay { digest: info.digest, label: "Digest".to_string(), is_link: false }
                    DigestDisplay { digest: info.prev_block_digest, label: "Previous Digest".to_string(), is_link: true }

                    hr {}

                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1rem;",
                        div {
                           strong { "Timestamp" }
                           p { "{info.timestamp.standard_format()}" }
                        }
                        div {
                           strong { "Size (BFE)" }
                           p { "{info.size}" }
                        }
                        div {
                           strong { "Difficulty" }
                           p { "{info.difficulty}" }
                        }
                        div {
                           strong { "Proof of Work" }
                           p { "{info.cumulative_proof_of_work}" }
                        }
                        div {
                           strong { "Coinbase" }
                           p { "{info.coinbase_amount}" }
                        }
                         div {
                           strong { "Fee" }
                           p { "{info.fee}" }
                        }
                    }

                    hr {}

                    details {
                        summary { "Transaction Info" }
                        div {
                             style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1rem;",
                             div { strong { "Inputs:" } p { "{info.num_inputs}" } }
                             div { strong { "Outputs:" } p { "{info.num_outputs}" } }
                             div { strong { "Announcements:" } p { "{info.num_announcements}" } }
                        }
                    }

                    details {
                        summary { "Block Status" }
                        ul { style: "margin-top: 1rem;",
                            li { "Is Genesis: {info.is_genesis}" }
                            li { "Is Tip: {info.is_tip}" }
                            li { "Is Canonical: {info.is_canonical}" }
                            li { "Sibling Blocks: {info.sibling_blocks.len()}" }
                        }
                    }
                }
            }
        }
    }
}
