// src/screens/block.rs
use crate::components::pico::{Card, CopyButton};
use dioxus::prelude::*;
use neptune_types::block_info::BlockInfo;
use neptune_types::block_selector::BlockSelector;
use twenty_first::tip5::Digest;

/// A small helper component to display a Digest with a label and copy button.
#[component]
fn DigestDisplay(
    digest: Digest,
    label: String,
    is_link: bool,
    current_selector: Signal<BlockSelector>,
) -> Element {
    let digest_str = digest.to_hex();
    let abbreviated_digest = format!(
        "{}...{}",
        &digest_str[0..12],
        &digest_str[digest_str.len() - 12..]
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
                            current_selector.set(BlockSelector::Digest(digest));
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
    let mut current_selector = use_signal(|| selector);
    let mut block_resource =
        use_resource(move || async move { api::block_info(current_selector()).await });
    let mut displayed_info = use_signal::<Option<BlockInfo>>(|| None);

    // FIX: This effect now ONLY reads from `block_resource` and WRITES to `displayed_info`.
    // It no longer reads from `displayed_info`, which resolves the infinite loop warning.
    use_effect(move || match block_resource.read().as_ref() {
        Some(Ok(Some(info))) => {
            displayed_info.set(Some(info.clone()));
        }
        Some(Err(_)) | Some(Ok(None)) => {
            displayed_info.set(None);
        }
        None => {}
    });

    if let Some(info) = displayed_info() {
        let is_loading = block_resource.read().is_none();
        let info_clone_prev = info.clone();
        let info_clone_next = info.clone();

        rsx! {
            Card {
                h3 { "Block Details" }
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",
                    button {
                        disabled: info.is_genesis || is_loading,
                        onclick: move |_| {
                            if !info_clone_prev.is_genesis {
                                current_selector.set(BlockSelector::Digest(info_clone_prev.prev_block_digest));
                            }
                        },
                        "❮ Previous Block"
                    }
                    h5 { style: "margin: 0;", "Height: {info.height}" }
                    button {
                        disabled: info.is_tip || is_loading,
                        onclick: move |_| {
                            if !info_clone_next.is_tip {
                                let next_height = info_clone_next.height + 1;
                                current_selector.set(BlockSelector::Height(next_height));
                            }
                        },
                        "Next Block ❯"
                    }
                }
                DigestDisplay { digest: info.digest, label: "Digest".to_string(), is_link: false, current_selector: current_selector }
                hr {}
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1rem;",
                    div { strong { "Timestamp" } p { "{info.timestamp.standard_format()}" } }
                    div { strong { "Size (BFE)" } p { "{info.size}" } }
                    div { strong { "Difficulty" } p { "{info.difficulty}" } }
                    div { strong { "Proof of Work" } p { "{info.cumulative_proof_of_work}" } }
                    div { strong { "Coinbase" } p { "{info.coinbase_amount}" } }
                    div { strong { "Fee" } p { "{info.fee}" } }
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
    } else {
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
                        p { "The requested block was not found." }
                    }
                },
                 _ => rsx! { p { " " } }
            }
        }
    }
}
