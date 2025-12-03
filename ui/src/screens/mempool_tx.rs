// ui/src/screens/mempool_tx.rs
use dioxus::prelude::*;
use neptune_types::announcement::Announcement;
use neptune_types::mutator_set::addition_record::AdditionRecord;
use neptune_types::mutator_set::chunk::Chunk;
use neptune_types::mutator_set::chunk_dictionary::ChunkDictionary;
use neptune_types::mutator_set::removal_record::absolute_index_set::AbsoluteIndexSet;
use neptune_types::mutator_set::removal_record::RemovalRecord;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use num_traits::Zero;
use twenty_first::tip5::Digest;
use twenty_first::util_types::mmr::mmr_membership_proof::MmrMembershipProof;

use crate::components::pico::Card;
use crate::components::pico::CopyButton;
use crate::hooks::use_rpc_checker::use_rpc_checker;

// --- Helper & Sub-Components ---

#[component]
fn DigestDisplay(digest: Digest, label: String, abbreviated: Option<bool>) -> Element {
    // Use to_hex() instead of to_string()
    let digest_hex = digest.to_hex();
    let is_abbreviated = abbreviated.unwrap_or(true);
    let display_str = if is_abbreviated {
        format!(
            "{}...{}",
            &digest_hex[0..6],
            &digest_hex[digest_hex.len() - 4..]
        )
    } else {
        digest_hex.clone()
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; padding: 0.25rem 0;",
            strong {
                "{label}:"
            }
            div {
                style: "display: flex; align-items: center; gap: 0.5rem;",
                code {
                    title: "{digest_hex}",
                    "{display_str}"
                }
                CopyButton {
                    text_to_copy: &digest_hex,
                }
            }
        }
    }
}

#[component]
fn ChunkDisplay(chunk: Chunk) -> Element {
    let indices_str = chunk
        .relative_indices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        details {
            summary {
                "Chunk ({chunk.relative_indices.len()} indices)"
            }
            div {
                style: "padding: 0.5rem; margin-top: 0.5rem; background-color: var(--pico-secondary-background-color); border-radius: var(--pico-border-radius); font-size: 0.875em;",
                p {
                    style: "margin: 0; word-break: break-all;",
                    strong {
                        "Relative Indices: "
                    }
                    "{indices_str}"
                }
            }
        }
    }
}

#[component]
fn MmrMembershipProofDisplay(proof: MmrMembershipProof) -> Element {
    rsx! {
        details {
            summary {
                "MMR Membership Proof ({proof.authentication_path.len()} digests)"
            }
            div {
                style: "padding: 0.5rem; margin-top: 0.5rem; background-color: var(--pico-secondary-background-color); border-radius: var(--pico-border-radius);",
                for (i , digest) in proof.authentication_path.iter().enumerate() {
                    DigestDisplay {
                        label: format!("Digest {}", i),
                        digest: *digest,
                    }
                }
            }
        }
    }
}

#[component]
fn AbsoluteIndexSetDisplay(ais: AbsoluteIndexSet) -> Element {
    let absolute_indices = ais.to_vec();
    let indices_str = absolute_indices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    rsx! {
        div {
            style: "border: 1px solid var(--pico-muted-border-color); border-radius: var(--pico-border-radius); padding: 0.75rem; margin-bottom: 0.75rem;",
            details {
                summary {
                    "Absolute Index Set ({absolute_indices.len()} indices)"
                }
                div {
                    style: "padding: 0.5rem; margin-top: 0.5rem; background-color: var(--pico-secondary-background-color); border-radius: var(--pico-border-radius); font-size: 0.875em; word-break: break-all;",
                    "{indices_str}"
                }
            }
        }
    }
}

#[component]
fn ChunkDictionaryDisplay(dictionary: ChunkDictionary) -> Element {
    rsx! {
        div {
            style: "border: 1px solid var(--pico-muted-border-color); border-radius: var(--pico-border-radius); padding: 0.75rem;",
            h6 {
                style: "margin: 0 0 0.5rem 0;",
                "Target Chunks ({dictionary.len()})"
            }
            if dictionary.is_empty() {
                p {
                    style: "font-style: italic;",
                    "No target chunks."
                }
            } else {
                for (i , (chunk_index , (proof , chunk))) in dictionary.iter().enumerate() {
                    div {
                        style: "border-top: 1px solid var(--pico-muted-border-color); padding: 0.75rem 0;",
                        p {
                            style: "margin: 0 0 0.5rem 0;",
                            strong {
                                "Entry {i}: "
                            }
                            span {
                                "Chunk at index "
                            }
                            code {
                                "{*chunk_index}"
                            }
                        }
                        ChunkDisplay {
                            chunk: chunk.clone(),
                        }
                        MmrMembershipProofDisplay {
                            proof: proof.clone(),
                        }
                    }
                }
            }
        }
    }
}

// --- Main Display Components ---

#[component]
fn AnnouncementDisplay(announcement: Announcement, index: usize) -> Element {
    let announcement_str = announcement.to_string();
    rsx! {
        div {
            class: "list-item",
            style: "margin-bottom: 0.75rem; padding: 0.75rem; border: 1px solid var(--pico-muted-border-color); border-radius: var(--pico-border-radius);",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;",
                strong {
                    "Announcement {index}"
                }
                CopyButton {
                    text_to_copy: announcement_str.clone(),
                }
            }
            pre {
                style: "background-color: var(--pico-secondary-background-color); padding: 0.5rem; border-radius: var(--pico-border-radius); max-height: 120px; overflow-y: auto; white-space: pre-wrap; word-break: break-all; margin: 0;",
                code {
                    "{announcement_str}"
                }
            }
        }
    }
}

#[component]
fn AdditionRecordDisplay(record: AdditionRecord, index: usize) -> Element {
    rsx! {
        div {
            class: "list-item",
            style: "margin-bottom: 0.75rem; padding: 0.5rem; border: 1px solid var(--pico-muted-border-color); border-radius: var(--pico-border-radius);",
            DigestDisplay {
                label: format!("Output {}", index),
                digest: record.canonical_commitment,
                abbreviated: false,
            }
        }
    }
}

#[component]
fn RemovalRecordDisplay(record: RemovalRecord, index: usize) -> Element {
    rsx! {
        div {
            class: "list-item",
            style: "margin-bottom: 1rem; padding: 0.75rem; border: 2px solid var(--pico-muted-border-color); border-radius: var(--pico-border-radius);",
            h5 {
                style: "margin-top: 0;",
                "Input {index}"
            }
            AbsoluteIndexSetDisplay {
                ais: record.absolute_indices,
            }
            ChunkDictionaryDisplay {
                dictionary: record.target_chunks.clone(),
            }
        }
    }
}

// --- Screen Component ---

#[component]
pub fn MempoolTxScreen(tx_id: TransactionKernelId) -> Element {
    let mut rpc = use_rpc_checker(); // Initialize Hook

    let mut mempool_tx = use_resource(move || async move { api::mempool_tx_kernel(tx_id).await });

    // Effect: Restarts the resource when connection is restored.
    let status_sig = rpc.status();
    use_effect(move || {
        if status_sig.read().is_connected() {
            mempool_tx.restart();
        }
    });

    rsx! {
        match &*mempool_tx.read() {
            None => rsx! {
                div {
                    style: "text-align: center; padding: 2rem;",
                    h4 {
                        "Loading transaction details..."
                    }
                }
            },
            // check if neptune-core rpc connection lost
            Some(result) if !rpc.check_result_ref(&result) => rsx! {
                // modal ConnectionLost is displayed by rpc.check_result_ref
                Card {
                    h3 {
                        "Mempool Transaction Details"
                    }
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 {
                        style: "color: var(--pico-color-red-500);",
                        "Error"
                    }
                    p {
                        "Could not fetch transaction details from the mempool."
                    }
                    hr {
                    }
                    h5 {
                        "Details:"
                    }
                    code {
                        "{e}"
                    }
                }
            },
            Some(Ok(None)) => rsx! {
                Card {
                    h3 {
                        "Not Found"
                    }
                    p {
                        "Transaction with ID was not found in the mempool:"
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem; margin-top: 1rem;",
                        code {
                            title: "{tx_id.to_string()}",
                            "{tx_id}"
                        }
                        CopyButton {
                            text_to_copy: tx_id.to_string(),
                        }
                    }
                }
            },
            Some(Ok(Some(kernel))) => {
                rsx! {
                    Card {
                        h3 {
                            "Mempool Transaction Details"
                        }
                        // --- Transaction ID Header ---
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem;",
                            h5 {
                                style: "margin: 0;",
                                "Transaction ID"
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 0.5rem;",
                                code {
                                    title: "{tx_id.to_string()}",
                                    "{tx_id}"
                                }
                                CopyButton {
                                    text_to_copy: tx_id.to_string(),
                                }
                            }
                        }
                        hr {
                        }

                        // --- SCROLLABLE CONTENT WRAPPER ---
                        div {
                            // flex: 1 takes up remaining space.
                            // overflow-y: auto enables scrolling.
                            // min-height: 0 ensures flex items can shrink properly in Firefox/Chrome.
                            style: "flex: 1; overflow-y: auto; min-height: 0; padding-right: 0.5rem;",

                            // --- Summary Section ---
                            h5 {
                                style: "margin-top: 1rem; margin-bottom: 0.5rem;",
                                "Summary"
                            }
                            div {
                                style: "display: grid; grid-template-columns: auto 1fr; gap: 0.5rem 1rem; align-items: center;",
                                strong {
                                    "Timestamp:"
                                }
                                span {
                                    "{kernel.timestamp.standard_format()}"
                                }
                                strong {
                                    "Fee:"
                                }
                                span {
                                    "{kernel.fee}"
                                }
                                strong {
                                    "Coinbase:"
                                }
                                span {
                                    "{kernel.coinbase.unwrap_or_else(NativeCurrencyAmount::zero)}"
                                }
                                strong {
                                    "Inputs:"
                                }
                                span {
                                    "{kernel.inputs.len()}"
                                }
                                strong {
                                    "Outputs:"
                                }
                                span {
                                    "{kernel.outputs.len()}"
                                }
                                strong {
                                    "Announcements:"
                                }
                                span {
                                    "{kernel.announcements.len()}"
                                }
                            }
                            hr {
                            }
                            // --- Details Section ---
                            h5 {
                                style: "margin-top: 1rem; margin-bottom: 0.5rem;",
                                "Details"
                            }
                            DigestDisplay {
                                label: "Mutator Set Hash".to_string(),
                                digest: kernel.mutator_set_hash,
                            }
                            // --- Collapsible Lists ---
                            details {
                                summary {
                                    "Inputs ({kernel.inputs.len()})"
                                }
                                div {
                                    class: "list-container",
                                    style: "margin-top: 0.5rem; padding-left: 1rem;",
                                    for (i , input) in kernel.inputs.iter().enumerate() {
                                        RemovalRecordDisplay {
                                            record: input.clone(),
                                            index: i,
                                        }
                                    }
                                }
                            }
                            details {
                                summary {
                                    "Outputs ({kernel.outputs.len()})"
                                }
                                div {
                                    class: "list-container",
                                    style: "margin-top: 0.5rem; padding-left: 1rem;",
                                    for (i , output) in kernel.outputs.iter().enumerate() {
                                        AdditionRecordDisplay {
                                            record: *output,
                                            index: i,
                                        }
                                    }
                                }
                            }
                            details {
                                summary {
                                    "Announcements ({kernel.announcements.len()})"
                                }
                                div {
                                    class: "list-container",
                                    style: "margin-top: 0.5rem; padding-left: 1rem;",
                                    for (i , announcement) in kernel.announcements.iter().enumerate() {
                                        AnnouncementDisplay {
                                            announcement: announcement.clone(),
                                            index: i,
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
}