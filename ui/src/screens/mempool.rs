//=============================================================================
// File: src/screens/mempool.rs
//=============================================================================
use crate::components::amount::Amount;
use crate::components::amount::AmountType;
use crate::components::pico::Card;
use crate::components::action_link::ActionLink;
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::mempool_transaction_info::MempoolTransactionInfo;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

use num_traits::CheckedSub;

// Enums to manage sorting state
#[derive(Clone, Copy, PartialEq)]
enum SortableColumn {
    Id,
    ProofType,
    Inputs,
    Outputs,
    BalanceEffect,
    Fee,
    Synced,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

// A helper function to safely calculate balance effect as a signed integer for sorting.
// We assume `NativeCurrencyAmount` is a tuple struct wrapping a u128, so we access with `.0`.
fn calculate_balance_effect(tx: &MempoolTransactionInfo) -> NativeCurrencyAmount {
    tx.positive_balance_effect
        .checked_sub(&tx.negative_balance_effect)
        .unwrap_or_default()
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
            style: "position: sticky; top: 0; background: var(--pico-card-background-color); cursor: pointer; white-space: nowrap; padding: 12px 4px;",
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

/// A self-contained component for rendering a single row in the mempool table.
#[component]
fn MempoolRow(tx: MempoolTransactionInfoReadOnly) -> Element {
    let active_screen = use_context::<Signal<Screen>>();
    let mut is_hovered = use_signal(|| false);

    // note: as of neptune-core v0.3.0, the negative and positive balance
    // effect fields are backwards.  ie:
    //    negative_balance_effect is the amount added to own wallet.
    //    positive_balance_effect is the amount removed from own wallet.
    // thus we subtract positive_balance_effect from positive_balance_effect to
    // obtain the balance delta.
    //
    // note that we cannot directly use subtraction to obtain a negative number
    // but we can add a negative number to do so. this is an inconsistency in NativeCurrencyAmount.
    let delta = tx.negative_balance_effect + -tx.positive_balance_effect;

    let balance_effect_display = rsx! {
        Amount {
            amount: delta,
            fixed: Some(AmountType::Current)
        }
    };

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
                style: "padding: 8px 4px;",
                ActionLink {
                    state: active_screen,
                    to: Screen::MempoolTx(tx.id),
                    "{abbreviated_tx_id}"
                }
            }
            td {
                style: "padding: 8px 4px;",
                "{tx.proof_type}"
            }
            td {
                style: "padding: 8px 4px;",
                "{tx.num_inputs}"
            }
            td {
                style: "padding: 8px 4px;",
                "{tx.num_outputs}"
            }
            td {
                style: "padding: 8px 4px;",
                {balance_effect_display}
            }
            td {
                style: "padding: 8px 4px;",
                Amount {
                    amount: tx.fee,
                    fixed: Some(AmountType::Current)
                }
            }
            td {
                style: "text-align: center; padding: 8px 4px;",
                if tx.synced {
                    "✅"
                } else {
                    "❌"
                }
            }
        }
    }
}

#[component]
pub fn MempoolScreen() -> Element {
    let mut mempool_overview =
        use_resource(move || async move { api::mempool_overview(0, 1000).await });

    // State for sorting
    let sort_column = use_signal(|| SortableColumn::Fee);
    let sort_direction = use_signal(|| SortDirection::Descending);

    // API Polling every 10 seconds
    // This effect runs once on component mount and starts a background task.
    use_effect(move || {
        // We need to clone the signal to move it into the async task.
        let mut mempool_overview = mempool_overview;
        spawn(async move {
            loop {
                crate::compat::sleep(Duration::from_secs(10)).await;
                mempool_overview.restart();
            }
        });
    });

    rsx! {
        match &*mempool_overview.read() {
            None => rsx! {
                Card {

                    h3 {

                        "Mempool"
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

                        "Failed to load mempool data: {e}"
                    }
                    button {
                        onclick: move |_| mempool_overview.restart(),
                        "Retry"
                    }
                }
            },
            Some(Ok(tx_list)) => {
                let mut sorted_txs = tx_list.clone();
                sorted_txs
                    .sort_by(|a, b| {
                        let ordering = match sort_column() {
                            SortableColumn::Id => a.id.cmp(&b.id),
                            SortableColumn::ProofType => {
                                a.proof_type.to_string().cmp(&b.proof_type.to_string())
                            }
                            SortableColumn::Inputs => a.num_inputs.cmp(&b.num_inputs),
                            SortableColumn::Outputs => a.num_outputs.cmp(&b.num_outputs),
                            SortableColumn::BalanceEffect => {
                                let bal_a = calculate_balance_effect(a);
                                let bal_b = calculate_balance_effect(b);
                                bal_a.cmp(&bal_b)
                            }
                            SortableColumn::Fee => a.fee.cmp(&b.fee),
                            SortableColumn::Synced => a.synced.cmp(&b.synced),
                        };
                        match sort_direction() {
                            SortDirection::Ascending => ordering,
                            SortDirection::Descending => ordering.reverse(),
                        }
                    });
                rsx! {
                    Card {

                        h3 {

                            "Mempool"
                        }
                        p {

                            "Transactions: {tx_list.len()}"
                        }
                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {

                                thead {

                                    tr {

                                        SortableHeader {
                                            title: "Id",
                                            column: SortableColumn::Id,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Proof",
                                            column: SortableColumn::ProofType,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Inputs",
                                            column: SortableColumn::Inputs,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Outputs",
                                            column: SortableColumn::Outputs,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Δ Balance",
                                            column: SortableColumn::BalanceEffect,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Fee",
                                            column: SortableColumn::Fee,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Synced",
                                            column: SortableColumn::Synced,
                                            sort_column,
                                            sort_direction,
                                        }
                                    }
                                }
                                tbody {

                                    {
                                        sorted_txs
                                            .into_iter()
                                            .map(|tx| {
                                                rsx! {
                                                    MempoolRow {
                                                        tx: MempoolTransactionInfoReadOnly(Rc::new(tx)),
                                                    }
                                                }
                                            })
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
