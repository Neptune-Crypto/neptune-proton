//=============================================================================
// File: src/screens/history.rs
//=============================================================================
use crate::components::block::Block;
use crate::components::pico::Card;
use dioxus::prelude::*;

use itertools::Itertools;
use neptune_types::block_height::BlockHeight;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::timestamp::Timestamp;
use num_traits::Zero;
use std::rc::Rc;
use twenty_first::tip5::Digest;

/// A new, self-contained component for rendering a single row in the history table.
#[component]
fn HistoryRow(
    digest: Digest,
    height: BlockHeight,
    timestamp: Timestamp,
    amount: NativeCurrencyAmount,
) -> Element {
    let digest = Rc::new(digest);
    let height = Rc::new(height);
    // This component now manages its own hover and copied state locally.
    let mut is_hovered = use_signal(|| false);

    let tx_type = if amount > NativeCurrencyAmount::zero() {
        "Received"
    } else {
        "Sent"
    };
    let date = timestamp.format("%Y-%m-%d");

    rsx! {
        tr {
            // When the mouse leaves, we reset both hover and copied states.
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),

            td {
                title: "{timestamp.standard_format()}",
                "{date}"
            }
            td { "{tx_type}" }
            td { "{amount} NPT" }
            td { Block{ block_digest: digest.clone(), height }}
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn HistoryScreen() -> Element {
    let mut history = use_resource(move || async move { api::history().await });

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
                let iter = utxos.iter().rev().chunk_by(|(digest, height, timestamp, _)| (digest, height, timestamp));
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
