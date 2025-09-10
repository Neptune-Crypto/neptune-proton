//=============================================================================
// File: src/components/address.rs
//=============================================================================
use crate::components::pico::{Button, CopyButton, Modal, NoTitleModal};
use crate::AppState;
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::address::ReceivingAddress;
use neptune_types::block_height::BlockHeight;
use neptune_types::block_selector::BlockSelector;
use std::rc::Rc;
use twenty_first::tip5::Digest;

#[derive(Props, PartialEq, Clone)]
pub struct BlockProps {
    pub block_digest: Rc<Digest>,
    pub height: Rc<BlockHeight>,
}

impl BlockProps {
    fn abbreviated(&self) -> String {
        truncate_with_ellipsis(&self.block_digest.to_hex())
    }
}

#[component]
pub fn Block(props: BlockProps) -> Element {
    let network = use_context::<AppState>().network;

    let props_clone = props.clone();
    let height = *props_clone.height;
    let digest = props_clone.block_digest.clone();
    let abbreviated = use_memo(move || props_clone.abbreviated());
    let mut active_screen = use_context::<Signal<Screen>>();

    rsx! {

        // --- The clickable abbreviated address display ---
        div {
            style: "cursor: pointer;",
            title: "{abbreviated}",
            onclick: move |_| {
                active_screen.set(Screen::Block(BlockSelector::Digest(*digest)));
            },
            code { "{height}" }
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
