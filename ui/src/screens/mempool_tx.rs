// src/screens/mempool_tx.rs
use crate::components::pico::{Card, CopyButton};
use dioxus::prelude::*;
use neptune_types::{
    native_currency_amount::NativeCurrencyAmount,
    transaction_kernel_id::TransactionKernelId,
    transaction_kernel::TransactionKernel,
};
use twenty_first::tip5::Digest;
use num_traits::Zero;

/// A small helper component to display a Digest with a label and copy button.
#[component]
fn DigestDisplay(digest: Digest, label: String) -> Element {
    let digest_str = digest.to_string();
    let abbreviated_digest = format!("{}...{}", &digest_str[0..6], &digest_str[digest_str.len() - 4..]);
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; padding: 0.25rem 0;",
            strong { "{label}:" }
            div {
                style: "display: flex; align-items: center; gap: 0.5rem;",
                code { title: "{digest_str}", "{abbreviated_digest}" }
                CopyButton { text_to_copy: &digest_str }
            }
        }
    }
}

// In src/screens/mempool_tx.rs

#[component]
pub fn MempoolTxScreen(tx_id: TransactionKernelId) -> Element {
    let mut mempool_tx = use_resource(move || async move {
        api::mempool_tx_kernel(tx_id).await
    });

    rsx! {
        match &*mempool_tx.read() {
            None => rsx! {
                // ... same as before
            },
            Some(Err(e)) => rsx! {
                // ... same as before
            },
            Some(Ok(None)) => rsx! {
                // ... same as before
            },
            Some(Ok(Some(kernel))) => {
                // --- THE FIX: Prepare complex strings outside the macro ---
                let inputs_str = std::fmt::format(format_args!("{:#?}", kernel.inputs));
                let outputs_str = std::fmt::format(format_args!("{:#?}", kernel.outputs));
                let announcements_str = std::fmt::format(format_args!("{:#?}", kernel.announcements));

                rsx! {
                    Card {
                        h3 { "Mempool Transaction Details" }
                        // --- Transaction ID Header (unchanged) ---
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem;",
                            h5 { style: "margin: 0;", "Transaction ID" }
                            div {
                                style: "display: flex; align-items: center; gap: 0.5rem;",
                                code { title: "{tx_id.to_string()}", "{tx_id}" }
                                CopyButton { text_to_copy: tx_id.to_string() }
                            }
                        }
                        hr {}
                        // --- Summary Section (unchanged) ---
                        h5 { style: "margin-top: 1rem; margin-bottom: 0.5rem;", "Summary" }
                        div {
                            style: "display: grid; grid-template-columns: auto 1fr; gap: 0.5rem 1rem; align-items: center;",
                            strong { "Timestamp:" }
                            span { "{kernel.timestamp.standard_format()}" }
                            strong { "Fee:" }
                            span { "{kernel.fee}" }
                            strong { "Coinbase:" }
                            span { "{kernel.coinbase.unwrap_or_else(NativeCurrencyAmount::zero)}" }
                            strong { "Inputs:" }
                            span { "{kernel.inputs.len()}" }
                            strong { "Outputs:" }
                            span { "{kernel.outputs.len()}" }
                            strong { "Announcements:" }
                            span { "{kernel.announcements.len()}" }
                        }
                        hr {}
                        // --- Details Section (unchanged) ---
                        h5 { style: "margin-top: 1rem; margin-bottom: 0.5rem;", "Details" }
                        DigestDisplay {
                            label: "Mutator Set Hash".to_string(),
                            digest: kernel.mutator_set_hash
                        }

                        // --- Collapsible Lists (Now using the prepared strings) ---
                        details {
                            summary { "Inputs ({kernel.inputs.len()})" }
                            pre {
                                style: "margin-top: 0.5rem; background-color: var(--pico-muted-background-color); padding: 0.5rem; border-radius: var(--pico-border-radius);",
                                code { "{inputs_str}" }
                            }
                        }
                         details {
                            summary { "Outputs ({kernel.outputs.len()})" }
                             pre {
                                style: "margin-top: 0.5rem; background-color: var(--pico-muted-background-color); padding: 0.5rem; border-radius: var(--pico-border-radius);",
                                code { "{outputs_str}" }
                            }
                        }
                        details {
                            summary { "Announcements ({kernel.announcements.len()})" }
                             pre {
                                style: "margin-top: 0.5rem; background-color: var(--pico-muted-background-color); padding: 0.5rem; border-radius: var(--pico-border-radius);",
                                code { "{announcements_str}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

