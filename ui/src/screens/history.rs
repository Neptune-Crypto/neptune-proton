//=============================================================================
// File: src/screens/history.rs
//=============================================================================
use crate::components::pico::Card;
use crate::components::pico::CopyButton;
use dioxus::prelude::*;

use neptune_types::block_height::BlockHeight;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use twenty_first::tip5::Digest;
use neptune_types::timestamp::Timestamp;
use crate::AppState;
use num_traits::Zero;
use itertools::Itertools;

/// A new, self-contained component for rendering a single row in the history table.
#[component]
fn HistoryRow(
    digest: Digest,
    height: BlockHeight,
    timestamp: Timestamp,
    amount: NativeCurrencyAmount,
) -> Element {
    // This component now manages its own hover and copied state locally.
    let mut is_hovered = use_signal(|| false);

    let tx_type = if amount > NativeCurrencyAmount::zero() { "Received" } else { "Sent" };
    let digest_abbrev = truncate_with_ellipsis(&digest.to_hex());

    rsx! {
        tr {
            // When the mouse leaves, we reset both hover and copied states.
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),

            td { "{timestamp.standard_format()}" }
            td { "{tx_type}" }
            td { "{amount} NPT" }
            td { code { "{digest_abbrev}" } }
            td {
                style: "min-width: 150px; text-align: right;",
                if is_hovered() {
                    // Use Pico's `role="group"` for horizontal button layout.
                    div {
                        style: "font-size: 0.8em",
                        role: "group",
                        CopyButton {
                            text_to_copy: digest.to_hex()
                        }
                    }
                }
            }
        }
    }
}


#[allow(non_snake_case)]
#[component]
pub fn HistoryScreen() -> Element {

    let network = use_context::<AppState>().network;

    let mut history = use_resource(move || async move {
        api::history().await
    });

    // Vec<(Digest, BlockHeight, Timestamp, NativeCurrencyAmount)

    rsx! {
        match &*history.read() {
            None => rsx! {
                Card {
                    h3 { "History" }
                    p { "Loading..." }
                    progress {}
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load history: {e}" }
                    button { onclick: move |_| history.restart(), "Retry" }
                }
            },
            Some(Ok(utxos)) => {
                let iter = utxos.iter().group_by(|(digest, height, timestamp, _)| (digest, height, timestamp));
                let block_summaries = iter.into_iter()
                    .map(|(key, group)| {
                        let (digest, height, timestamp) = key;
                        let amount_sum: NativeCurrencyAmount = group.map(|(.., amount)| *amount).sum();
                        (digest, height, timestamp, amount_sum )
                    });

                rsx! {
                    Card {
                        h3 { "History" }
                        table {
                            thead { tr {
                                th { "Date" }
                                th { "Type" }
                                th { "Amount" }
                                th { "Block" }
                                th { "" }
                            }}
                            tbody {

                                {block_summaries.map(|(digest, height, timestamp, amount)| {
                                    rsx! {
                                        HistoryRow {
                                            digest: *digest,
                                            height: *height,
                                            timestamp: *timestamp,
                                            amount,
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