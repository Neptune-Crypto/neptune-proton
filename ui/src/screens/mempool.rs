//=============================================================================
// File: src/screens/mempool.rs
//=============================================================================
use crate::components::pico::Card;
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::mempool_transaction_info::MempoolTransactionInfo;
use num_traits::CheckedSub;
use std::ops::Deref;
use std::rc::Rc;

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
fn MempoolRow(tx: MempoolTransactionInfoReadOnly) -> Element {
    // --- 2. GET THE SCREEN SIGNAL FROM THE CONTEXT ---
    let mut active_screen = use_context::<Signal<Screen>>();

    // This component now manages its own hover and copied state locally.
    let mut is_hovered = use_signal(|| false);

    let balance_effect = tx
        .positive_balance_effect
        .checked_sub(&tx.negative_balance_effect)
        .unwrap();

    // Create an abbreviated version of the tx id for a cleaner display
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

            // --- 3. MAKE THE TX ID A CLICKABLE LINK ---
            td {
                a {
                    href: "#",
                    title: "{tx_id_str}", // Show the full ID on hover
                    onclick: move |_| {
                        // Set the active screen to the detail view for this specific tx
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
    // ... the MempoolScreen component itself does not need any changes ...
    let mut mempool_overview =
        use_resource(move || async move { api::mempool_overview(0, 1000).await });

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
                Card {
                    h3 { "Mempool" }
                    p { "Transactions: {tx_list.len()}" }
                    table {
                        thead {
                            tr {
                                th { "Id" }
                                th { "Proof Type" }
                                th { "Inputs" }
                                th { "Outputs" }
                                th { "Balance Effect" }
                                th { "Fee" }
                            }
                        }
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
