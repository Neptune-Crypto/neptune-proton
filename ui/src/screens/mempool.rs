//=============================================================================
// File: src/screens/mempool.rs
//=============================================================================
use crate::components::pico::Card;
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::mempool_transaction_info::MempoolTransactionInfo;
use num_traits::CheckedSub;
use std::cmp::Ordering;
use std::ops::Deref;
use std::rc::Rc;

// Enums to manage sorting state
#[derive(Clone, Copy, PartialEq)]
enum SortableColumn {
    Id,
    ProofType,
    Inputs,
    Outputs,
    BalanceEffect,
    Fee,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
struct MempoolTransactionInfoReadOnly(Rc<MempoolTransactionInfo>);

impl PartialEq for MempoolTransactionInfoReadOnly {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
impl Eq for MempoolTransactionInfoReadOnly {}

impl Deref for MempoolTransactionInfoReadOnly {
    type Target = MempoolTransactionInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A reusable component for sortable table headers
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
                    sort_direction.with_mut(|dir| *dir = match dir {
                        SortDirection::Ascending => SortDirection::Descending,
                        SortDirection::Descending => SortDirection::Ascending,
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

/// A self-contained component for rendering a single row in the mempool table.
#[component]
fn MempoolRow(tx: MempoolTransactionInfoReadOnly) -> Element {
    let mut active_screen = use_context::<Signal<Screen>>();
    let mut is_hovered = use_signal(|| false);

    let balance_effect = tx
        .positive_balance_effect
        .checked_sub(&tx.negative_balance_effect)
        .unwrap();

    let tx_id_str = tx.id.to_string();
    let abbreviated_tx_id = format!(
        "{}...{}",
        &tx_id_str[0..6],
        &tx_id_str[tx_id_str.len() - 4..]
    );

    rsx! {
        tr {
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),

            td {
                a {
                    href: "#",
                    title: "{tx_id_str}",
                    onclick: move |_| {
                        active_screen.set(Screen::MempoolTx(tx.id.clone()));
                    },
                    "{abbreviated_tx_id}"
                }
            }
            td { "{tx.proof_type}" }
            td { "{tx.num_inputs}" }
            td { "{tx.num_outputs}" }
            td { "{balance_effect}" }
            td { "{tx.fee}" }
        }
    }
}

#[component]
pub fn MempoolScreen() -> Element {
    let mut mempool_overview =
        use_resource(move || async move { api::mempool_overview(0, 1000).await });

    // State for sorting
    let mut sort_column = use_signal(|| SortableColumn::Fee);
    let mut sort_direction = use_signal(|| SortDirection::Descending);

    rsx! {
        match &*mempool_overview.read() {
            None => rsx! {
                Card {
                    h3 { "Mempool" }
                    p { "Loading..." }
                    progress {}
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load mempool data: {e}" }
                    button { onclick: move |_| mempool_overview.restart(), "Retry" }
                }
            },
            Some(Ok(tx_list)) => {
                let mut sorted_txs = tx_list.clone();

                // Apply sorting based on the current state
                sorted_txs.sort_by(|a, b| {
                    let ordering = match sort_column() {
                        // Reverted to direct comparison now that `Ord` is implemented
                        SortableColumn::Id => a.id.cmp(&b.id),
                        SortableColumn::ProofType => a.proof_type.to_string().cmp(&b.proof_type.to_string()),
                        SortableColumn::Inputs => a.num_inputs.cmp(&b.num_inputs),
                        SortableColumn::Outputs => a.num_outputs.cmp(&b.num_outputs),
                        SortableColumn::BalanceEffect => {
                            let bal_a = a.positive_balance_effect.checked_sub(&a.negative_balance_effect).unwrap();
                            let bal_b = b.positive_balance_effect.checked_sub(&b.negative_balance_effect).unwrap();
                            bal_a.cmp(&bal_b)
                        },
                        SortableColumn::Fee => a.fee.cmp(&b.fee),
                    };

                    match sort_direction() {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });

                rsx! {
                    Card {
                        h3 { "Mempool" }
                        p { "Transactions: {tx_list.len()}" }

                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {
                                thead {
                                    tr {
                                        SortableHeader { title: "Id", column: SortableColumn::Id, sort_column, sort_direction }
                                        SortableHeader { title: "Proof Type", column: SortableColumn::ProofType, sort_column, sort_direction }
                                        SortableHeader { title: "Inputs", column: SortableColumn::Inputs, sort_column, sort_direction }
                                        SortableHeader { title: "Outputs", column: SortableColumn::Outputs, sort_column, sort_direction }
                                        SortableHeader { title: "Balance Effect", column: SortableColumn::BalanceEffect, sort_column, sort_direction }
                                        SortableHeader { title: "Fee", column: SortableColumn::Fee, sort_column, sort_direction }
                                    }
                                }
                                tbody {
                                    {sorted_txs.into_iter().map(|tx| {
                                        rsx! {
                                            MempoolRow { tx: MempoolTransactionInfoReadOnly(Rc::new(tx)) }
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
}
