//=============================================================================
// File: src/screens/send.rs
//=============================================================================
use std::rc::Rc;
use std::str::FromStr;
use dioxus::prelude::*;
use crate::components::pico::{Button, ButtonType, Card, CopyButton, Grid, Input, Modal, NoTitleModal};
use neptune_types::address::{KeyType, ReceivingAddress};
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use crate::AppState;
use num_traits::Zero;
use dioxus::dioxus_core::SpawnIfAsync;

// --- Data Structures ---

/// A struct to hold the data for a single recipient with validated data.
#[derive(Clone, PartialEq)]
struct Recipient {
    address: Rc<ReceivingAddress>,
    amount: NativeCurrencyAmount,
}

/// A struct to hold the string data for a recipient row while it's being edited.
#[derive(Clone, PartialEq, Debug)]
struct EditableRecipient {
    address_str: String,
    amount_str: String,
    address_error: Option<String>,
    amount_error: Option<String>,
}

impl Default for EditableRecipient {
    fn default() -> Self {
        Self {
            address_str: String::new(),
            amount_str: String::new(),
            address_error: None,
            amount_error: None,
        }
    }
}

// --- Components ---

/// An editable row in the recipient grid.
#[component]
fn EditableRecipientRow(
    index: usize,
    mut recipient: Signal<EditableRecipient>,
    on_delete: EventHandler<usize>,
) -> Element {
    let network = use_context::<AppState>().network;

    let handle_paste_address = move || {
        spawn(async move {
            if let Ok(js_text) = wasm_bindgen_futures::JsFuture::from(web_sys::window().unwrap().navigator().clipboard().read_text()).await {
                let clipboard_text = js_text.as_string().unwrap_or_default();
                recipient.with_mut(|r| r.address_str = clipboard_text);
            }
        });
    };

    rsx! {
        div {
            class: "recipient-row",
            style: "border: 1px solid var(--pico-form-element-border-color); border-radius: var(--pico-border-radius); padding: 1rem; margin-bottom: 1rem;",

            // Address Row
            div {
                label { "Recipient Address" }
                div {
                    role: "group",
                    Input {
                        label: "".to_string(),
                        name: "address",
                        placeholder: "Paste or scan an address...",
                        value: "{recipient.read().address_str}",
                        on_input: move |event: FormEvent| {
                            recipient.with_mut(|r| r.address_str = event.value().clone());
                        }
                    }
                }
                if let Some(err) = &recipient.read().address_error {
                    small { style: "color: var(--pico-color-red-500);", "{err}" }
                }
            }

            // Amount and Delete Row
            Grid {
                div {
                    label { "Amount" }
                    Input {
                        label: "".to_string(),
                        name: "amount",
                        input_type: "number".to_string(),
                        placeholder: "0.0",
                        value: "{recipient.read().amount_str}",
                        on_input: move |event: FormEvent| {
                            recipient.with_mut(|r| r.amount_str = event.value().clone());
                        }
                    }
                    if let Some(err) = &recipient.read().amount_error {
                        small { style: "color: var(--pico-color-red-500);", "{err}" }
                    }
                }
                div {
                    style: "display: flex; align-items: flex-end; justify-content: flex-end; height: 100%;",
                    Button {
                        button_type: ButtonType::Secondary,
                        outline: true,
                        on_click: move |_| handle_paste_address(),
                        "Paste Address"
                    }
                    Button {
                        button_type: ButtonType::Contrast,
                        outline: true,
                        on_click: move |_| on_delete.call(index),
                        "Delete"
                    }
                }
            }
        }
    }
}


#[component]
pub fn SendScreen() -> Element {
    let network = use_context::<AppState>().network;

    // --- Wizard State ---
    #[derive(PartialEq, Clone, Copy)]
    enum WizardStep {
        AddRecipients,
        Review,
        Status,
    }
    let mut wizard_step = use_signal(|| WizardStep::AddRecipients);

    // --- Main State ---
    // 1. Create the signal for the first row at the top level.
    let first_recipient = use_signal(EditableRecipient::default);
    // 2. Initialize the list signal with a vector containing the first signal.
    let mut recipients = use_signal(|| vec![first_recipient]);

    let mut fee_str = use_signal(String::new);
    let mut fee_error = use_signal::<Option<String>>(|| None);
    let mut api_response = use_signal::<Option<String>>(|| None);

    // --- Derived State ---
    let subtotal = use_memo(move || {
        recipients.read().iter().fold(NativeCurrencyAmount::zero(), |acc, r| {
            let amount = NativeCurrencyAmount::coins_from_str(&r.read().amount_str).unwrap_or_else(|_| NativeCurrencyAmount::zero());
            acc + amount
        })
    });
    let total_spend = use_memo(move || {
        let fee = NativeCurrencyAmount::coins_from_str(&fee_str()).unwrap_or_else(|_| NativeCurrencyAmount::zero());
        subtotal() + fee
    });

    // --- Event Handlers ---
    let mut reset_screen = move || {
        recipients.set(vec![use_signal(EditableRecipient::default)]);
        fee_str.set(String::new());
        fee_error.set(None);
        api_response.set(None);
        wizard_step.set(WizardStep::AddRecipients);
    };

    rsx! {
        // --- Wizard Content ---
        div {
            match wizard_step() {
                WizardStep::AddRecipients => rsx! {
                    Card {
                        h3 { "Add Recipients" }
                        for (i, recipient) in recipients.iter().enumerate() {
                            EditableRecipientRow {
                                key: "{i}",
                                index: i,
                                recipient: *recipient,
                                on_delete: move |index_to_delete: usize| {
                                    if recipients.len() > 1 {
                                        recipients.write().remove(index_to_delete);
                                    }
                                }
                            }
                        }

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-top: 1rem;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| {
                                    recipients.push(use_signal(EditableRecipient::default));
                                },
                                "Add Another Recipient"
                            }
                            if recipients.len() > 1 {
                                h5 { "Subtotal: {subtotal}" }
                            }
                        }
                    }
                    Card {
                        Input {
                            label: "Fee".to_string(),
                            name: "fee",
                            input_type: "number".to_string(),
                            placeholder: "0.0",
                            value: "{fee_str}",
                            on_input: move |event: FormEvent| fee_str.set(event.value().clone()),
                        }
                        if let Some(err) = fee_error() {
                            small { style: "color: var(--pico-color-red-500);", "{err}" }
                        }
                        h4 { style: "margin-top: 1rem; text-align: right;", "Total Spend: {total_spend}" }
                        Button {
                            on_click: move |_| {
                                let mut all_valid = true;
                                for i in 0..recipients.read().len() {
                                    let mut r = recipients.read()[i];
                                    let mut recipient = r.write();
                                    // Validate Address
                                    match ReceivingAddress::from_bech32m(&recipient.address_str, network) {
                                        Ok(_) => recipient.address_error = None,
                                        Err(e) => {
                                            recipient.address_error = Some(e.to_string());
                                            all_valid = false;
                                        }
                                    }
                                    // Validate Amount
                                    match NativeCurrencyAmount::coins_from_str(&recipient.amount_str) {
                                        Ok(amt) if amt > NativeCurrencyAmount::zero() => recipient.amount_error = None,
                                        _ => {
                                            recipient.amount_error = Some("Invalid amount".to_string());
                                            all_valid = false;
                                        }
                                    }
                                }

                                if NativeCurrencyAmount::coins_from_str(&fee_str()).is_err() {
                                    fee_error.set(Some("Invalid fee.".to_string()));
                                    all_valid = false;
                                } else {
                                    fee_error.set(None);
                                }

                                if all_valid {
                                    wizard_step.set(WizardStep::Review);
                                }
                            },
                            "Next: Review"
                        }
                    }
                },
                WizardStep::Review => rsx! {
                    Card {
                        h3 { "Review Transaction" }
                        p { "Please review the details below. This action cannot be undone." }
                        h5 { style: "margin-top: 1rem;", "Recipients:" }
                        table {
                            role: "grid",
                            tbody {
                                {recipients.read().iter().map(|recipient_signal| {
                                    let recipient = recipient_signal.read();
                                    let addr = ReceivingAddress::from_bech32m(&recipient.address_str, network).unwrap();
                                    let amount = NativeCurrencyAmount::coins_from_str(&recipient.amount_str).unwrap();
                                    rsx!{
                                        tr {
                                            td { code { "{addr.to_display_bech32m_abbreviated(network).unwrap()}" } }
                                            td { style: "text-align: right;", "{amount}" }
                                        }
                                    }
                                })}
                            }
                        }
                        hr {}
                        p { strong { "Fee: " } "{fee_str()}" }
                        p { strong { "Total Spend: " } "{total_spend}" }
                        footer {
                            style: "display: flex; justify-content: space-between; margin-top: 1rem;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| wizard_step.set(WizardStep::AddRecipients),
                                "Back"
                            }
                            Button {
                                on_click: move |_| {
                                    spawn(async move {
                                        // TODO: Replace with actual server call `api::send_transaction(...)`
                                        let result: Result<String, String> = Ok("Transaction Sent!".to_string());
                                        let message = match result {
                                            Ok(msg) => format!("Success: {}", msg),
                                            Err(err) => format!("API Error: {}", err),
                                        };
                                        api_response.set(Some(message));
                                        wizard_step.set(WizardStep::Status);
                                    });
                                },
                                "Confirm & Send"
                            }
                        }
                    }
                },
                WizardStep::Status => rsx! {
                    if let Some(response) = api_response() {
                        Card {
                            h3 { "Transaction Status" }
                            p { "{response}" }
                            Button {
                                on_click: move |_| reset_screen(),
                                "Send Another Transaction"
                            }
                        }
                    }
                }
            }
        }
    }
}
