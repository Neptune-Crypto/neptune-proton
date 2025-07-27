//=============================================================================
// File: src/screens/mempool.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

use neptune_types::mempool_transaction_info::MempoolTransactionInfo;
use std::ops::Deref;
use std::rc::Rc;
use num_traits::CheckedSub;

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


/// A new, self-contained component for rendering a single row in the history table.
#[component]
fn MempoolRow(
    tx: MempoolTransactionInfoReadOnly
) -> Element {
    // This component now manages its own hover and copied state locally.
    let mut is_hovered = use_signal(|| false);

    let balance_effect = tx.positive_balance_effect.checked_sub(&tx.negative_balance_effect).unwrap();

    // let date = timestamp.format("%Y-%m-%d");

    rsx! {
        tr {
            // When the mouse leaves, we reset both hover and copied states.
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),

            // pub id: TransactionKernelId,
            // pub proof_type: TransactionProofType,
            // pub num_inputs: usize,
            // pub num_outputs: usize,
            // pub positive_balance_effect: NativeCurrencyAmount,
            // pub negative_balance_effect: NativeCurrencyAmount,
            // pub fee: NativeCurrencyAmount,
            // pub synced: bool,


            // td { "{tx.id}"
            td { "[tx id]" }
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

    // let network = use_context::<AppState>().network;

    let mut mempool_overview = use_resource(move || async move {
        api::mempool_overview(0, 1000).await
    });

    // Vec<(Digest, BlockHeight, Timestamp, NativeCurrencyAmount)

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
            Some(Ok(tx_list)) => rsx! {

                // pub id: TransactionKernelId,
                // pub proof_type: TransactionProofType,
                // pub num_inputs: usize,
                // pub num_outputs: usize,
                // pub positive_balance_effect: NativeCurrencyAmount,
                // pub negative_balance_effect: NativeCurrencyAmount,
                // pub fee: NativeCurrencyAmount,
                // pub synced: bool,

                Card {
                    h3 { "Mempool" }
                    p { "Transactions: {tx_list.len()}"}
                    table {
                        thead { tr {
                            th { "Id" }
                            th { "Proof Type" }
                            th { "Inputs" }
                            th { "Outputs" }
                            th { "Balance Effect" }
                            th { "Fee" }
                        }}
                        tbody {

                            {tx_list.into_iter().map(|tx| {
                                rsx! {
                                    MempoolRow { tx: MempoolTransactionInfoReadOnly(Rc::new(*tx)) }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
