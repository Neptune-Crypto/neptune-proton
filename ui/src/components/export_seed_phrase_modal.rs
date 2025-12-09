//=============================================================================
// File: src/components/export_seed_phrase_modal.rs
//=============================================================================
use dioxus::prelude::*;
use neptune_types::secret_key_material::SecretKeyMaterial;

use crate::components::pico::Button;
use crate::components::pico::ButtonType;
use crate::components::pico::NoTitleModal;

#[derive(Clone, Copy, Debug, PartialEq)]
enum BackupStage {
    Instructions,
    DisplayingSeed,
}

#[component]
pub fn ExportSeedPhraseModal(is_open: Signal<bool>) -> Element {
    let mut stage = use_signal(|| BackupStage::Instructions);

    // Resource to fetch the seed phrase.
    // This automatically re-runs when 'stage' changes because stage() is read inside.
    let mut seed_words_resource = use_resource(move || async move {
        if stage() == BackupStage::Instructions {
            return Ok(None::<SecretKeyMaterial>);
        }

        match api::get_wallet_secret_key().await {
            Ok(secret) => Ok(Some(secret)),
            Err(e) => Err(e),
        }
    });

    // Reset the stage automatically whenever the modal closes.
    // This catches "Esc" keys and backdrop clicks handled by NoTitleModal.
    use_effect(move || {
        if !is_open() {
            stage.set(BackupStage::Instructions);
        }
    });

    let mut close_modal = move || {
        is_open.set(false);
    };

    rsx! {
        NoTitleModal {
            is_open: is_open,

            h5 {
                "⚠️ Export Seed Phrase"
            },

            match stage() {
                BackupStage::Instructions => rsx! {
                    div {
                        p { "Your **Secret Recovery Phrase** is the master key to your funds." }
                        p {
                            strong { "1. Prepare: " }
                            "Find a private location and something non-digital to write on (e.g., paper or metal)."
                        }
                        p {
                            strong { "2. Write Down: " }
                            "Write down the words in the exact order. Never type them into a computer."
                        }
                        p {
                            strong { "3. Security: " }
                            "Never share these words with anyone."
                        }
                    }
                },
                BackupStage::DisplayingSeed => rsx! {
                    match &*seed_words_resource.read() {
                        Some(Ok(Some(secret))) => rsx! {
                            div {
                                // card with 3 columns of seed words
                                style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; padding: 1rem; border-radius: var(--pico-border-radius); background: var(--pico-card-background-color); color: var(--pico-color); box-shadow: var(--pico-card-box-shadow);",
                                {
                                    secret.to_phrase().into_iter().enumerate().map(|(i, word)| {
                                        rsx! {
                                            div {
                                                key: "{i}",
                                                style: "text-align: left;",
                                                strong { "{i + 1}. " }
                                                "{word}"
                                            }
                                        }
                                    })
                                }
                            }
                            small {
                                style: "display: block; margin-top: 1rem; text-align: center; color: var(--pico-color-red-500); font-weight: bold;",
                                "🚨 VIEW IN PRIVATE! WRITE DOWN AND CLOSE IMMEDIATELY! 🚨"
                            }
                        },
                        Some(Err(e)) => rsx! {
                            div {
                                style: "color: var(--pico-color-red-500);",
                                p { "Error retrieving wallet secret:" }
                                pre { "{e}" }
                            }
                        },
                        _ => rsx! {
                            div {
                                style: "text-align: center;",
                                p { "Loading seed words..." }
                                progress {}
                            }
                        }
                    }
                }
            },

            footer {
                div {
                    style: "display: flex; justify-content: flex-end; gap: 1rem; margin-top: 1rem;",

                    Button {
                        button_type: ButtonType::Secondary,
                        outline: true,
                        on_click: move |_| close_modal(),
                        "Close"
                    }

                    if stage() == BackupStage::Instructions {
                        Button {
                            button_type: ButtonType::Primary,
                            on_click: move |_| {
                                stage.set(BackupStage::DisplayingSeed);
                                // The resource restart is triggered automatically because stage() is a dependency
                            },
                            "Display Seed Words"
                        }
                    }
                }
            }
        }
    }
}
