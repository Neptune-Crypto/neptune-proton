//=============================================================================
// File: src/components/address.rs
//=============================================================================
use crate::components::pico::{Button, CopyButton, Modal, NoTitleModal};
use crate::AppState;
use dioxus::prelude::*;
use neptune_types::address::ReceivingAddress;
use std::rc::Rc;
use twenty_first::tip5::Digest;

#[derive(Props, PartialEq, Clone)]
pub struct BlockProps {
    pub block_digest: Rc<Digest>,
}

impl BlockProps {
    fn abbreviated(&self) -> String {
        truncate_with_ellipsis(&self.block_digest.to_hex())
    }
}

#[component]
pub fn Block(props: BlockProps) -> Element {
    let network = use_context::<AppState>().network;
    let mut is_modal_open = use_signal(|| false);

    let props_clone = props.clone();
    let abbreviated = use_memo(move || props_clone.abbreviated());
    let full_digest = use_memo(move || props.block_digest.to_hex());

    rsx! {

        NoTitleModal {
            is_open: is_modal_open,
            div {
                style: "display: flex; flex-direction: column; align-items: center; text-align: center",
                // This flex container will center the buttons and add a gap between them.
                div {
                    style: "display: flex; justify-content: center; gap: 0.5rem;",
                    CopyButton { text_to_copy: full_digest() }
                    Button {
                        on_click: move |_| is_modal_open.set(false),
                        "Close"
                    }
                }
                h4 {
                    style: "margin-top: 1rem; margin-bottom: 0rem;",
                    "Full Block Digest"
                }
                code {
                    style: "text-align: left; word-break: break-all; background-color: var(--pico-muted-background-color); padding: 1rem; border-radius: var(--pico-border-radius); width: 100%; margin-bottom: 1rem;", // Gap after the code block
                    "{full_digest}"
                }
            }
        }

        // --- The clickable abbreviated address display ---
        div {
            style: "cursor: pointer;",
            title: "Click to view full block digest",
            onclick: move |_| is_modal_open.set(true),
            code { "{abbreviated}" }
        }
    }
}

/// Truncates a string to the first 4 and last 4 characters, joined by "..."
/// If the string is 8 characters or fewer, it's returned unchanged.
fn truncate_with_ellipsis(s: &str) -> String {
    // First, get the count of characters, which is different from byte length for UTF-8.
    let char_count = s.chars().count();

    // If the string is not long enough to need truncation, return it as a new String.
    if char_count <= 8 {
        return s.to_string();
    }

    // Get an iterator of the characters, take the first 4, and collect into a String.
    let first_part: String = s.chars().take(4).collect();

    // To get the last 4, skip the first (char_count - 4) characters.
    let last_part: String = s.chars().skip(char_count - 4).collect();

    // Use the format! macro to combine the parts.
    format!("{}...{}", first_part, last_part)
}