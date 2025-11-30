//=============================================================================
// File: src/screens/history.rs
//=============================================================================
use std::rc::Rc;

use dioxus::prelude::*;
use itertools::Itertools;
use neptune_types::block_height::BlockHeight;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::timestamp::Timestamp;
use num_traits::Zero;
use twenty_first::tip5::Digest;

use crate::components::amount::Amount;
use crate::components::block::Block;
use crate::components::pico::Card;

// Enums to manage sorting state
#[derive(Clone, Copy, PartialEq)]
enum SortableColumn {
    Date,
    Type,
    Amount,
    Block,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

// A reusable component for sortable table headers
#[component]
fn SortableHeader(
    title: &'static str,
    column: SortableColumn,
    sort_column: Signal<SortableColumn>,
    sort_direction: Signal<SortDirection>,
) -> Element {
    let (arrow_char, is_active) = if *sort_column.read() == column {
        (
            match *sort_direction.read() {
                SortDirection::Ascending => "▲",
                SortDirection::Descending => "▼",
            },
            true,
        )
    } else {
        ("\u{00A0}", false)
    };

    rsx! {
        th {
            style: "position: sticky; top: 0; background: var(--pico-card-background-color); cursor: pointer; white-space: nowrap;",
            onclick: move |_| {
                if is_active {
                    sort_direction
                        .with_mut(|dir| {
                            *dir = match dir {
                                SortDirection::Ascending => SortDirection::Descending,
                                SortDirection::Descending => SortDirection::Ascending,
                            };
                        });
                } else {
                    sort_column.set(column);
                    sort_direction.set(SortDirection::Ascending);
                }
            },
            "{title}"
            span {
                style: "display: inline-block; width: 1.2em; text-align: right;",
                "{arrow_char}"
            }
        }
    }
}

/// A self-contained component for rendering a single row in the history table.
#[component]
fn HistoryRow(
    digest: Digest,
    height: BlockHeight,
    timestamp: Timestamp,
    amount: NativeCurrencyAmount,
) -> Element {
    let digest = Rc::new(digest);
    let height = Rc::new(height);
    let mut is_hovered = use_signal(|| false);

    let tx_type = if amount > NativeCurrencyAmount::zero() {
        "Received"
    } else {
        "Sent"
    };
    let date = timestamp.format("%Y-%m-%d");

    rsx! {
        tr {
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),

            td {
                title: "{timestamp.standard_format()}",
                "{date}"
            }
            td {


                "{tx_type}"
            }
            td {
                style: "min-width: 21ch",

                Amount {
                    amount,
                }
            }
            td {


                Block {
                    block_digest: digest.clone(),
                    height,
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn HistoryScreen() -> Element {
    let mut history = use_resource(move || async move { api::history().await });

    // State for sorting
    let sort_column = use_signal(|| SortableColumn::Date);
    let sort_direction = use_signal(|| SortDirection::Descending);

    rsx! {
        match &*history.read() {
            None => rsx! {
                Card {

                    h3 {

                        "History"
                    }
                    p {

                        "Loading..."
                    }
                    progress {


                    }
                }
            },
            Some(Err(e)) => rsx! {
                Card {

                    h3 {

                        "Error"
                    }
                    p {

                        "Failed to load history: {e}"
                    }
                    button {
                        onclick: move |_| history.restart(),
                        "Retry"
                    }
                }
            },
            Some(Ok(utxos)) => {
                let iter = utxos
                    .iter()
                    .rev()
                    .chunk_by(|(digest, height, timestamp, _)| (digest, height, timestamp));
                let mut block_summaries: Vec<_> = iter
                    .into_iter()
                    .map(|(key, group)| {
                        let (digest, height, timestamp) = key;
                        let amount_sum: NativeCurrencyAmount = group
                            .map(|(.., amount)| *amount)
                            .sum();
                        (*digest, *height, *timestamp, amount_sum)
                    })
                    .collect();
                block_summaries
                    .sort_by(|a, b| {
                        let ordering = match sort_column() {
                            SortableColumn::Date => a.2.cmp(&b.2),
                            SortableColumn::Type => {
                                let type_a = if a.3 > NativeCurrencyAmount::zero() {
                                    "Received"
                                } else {
                                    "Sent"
                                };
                                let type_b = if b.3 > NativeCurrencyAmount::zero() {
                                    "Received"
                                } else {
                                    "Sent"
                                };
                                type_a.cmp(type_b)
                            }
                            SortableColumn::Amount => a.3.cmp(&b.3),
                            SortableColumn::Block => a.1.cmp(&b.1),
                        };
                        match sort_direction() {
                            SortDirection::Ascending => ordering,
                            SortDirection::Descending => ordering.reverse(),
                        }
                    });
                rsx! {
                    Card {

                        h3 {

                            "History"
                        }
                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {

                                thead {

                                    tr {

                                        SortableHeader {
                                            title: "Date",
                                            column: SortableColumn::Date,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Type",
                                            column: SortableColumn::Type,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Amount",
                                            column: SortableColumn::Amount,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Block",
                                            column: SortableColumn::Block,
                                            sort_column,
                                            sort_direction,
                                        }
                                    }
                                }
                                tbody {

                                    {
                                        block_summaries
                                            .into_iter()
                                            .map(|(digest, height, timestamp, amount)| {
                                                rsx! {
                                                    HistoryRow {
                                                        digest,
                                                        height,
                                                        timestamp,
                                                        amount,
                                                    }
                                                }
                                            })
                                    }
                                }
                            }
                        }
                        p {
                            style: "margin-top: 0.5rem",

                            em {

                                "Note: Unconfirmed transactions will appear once confirmed by the network."
                            }
                        }
                    }
                }
            }
        }
    }
}
