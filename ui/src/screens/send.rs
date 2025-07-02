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


/// A struct to hold the data for a single recipient in our transaction.
#[derive(Clone, PartialEq)]
struct Recipient {
    address: Rc<ReceivingAddress>,
    amount: NativeCurrencyAmount,
}

/// A sub-component for displaying a single recipient in the grid.
#[component]
fn RecipientRow(
    // The index of this row in the parent's list.
    index: usize,
    // The recipient data to display.
    recipient: Recipient,
    // Event handler to notify the parent when the delete button is clicked.
    on_delete: EventHandler<usize>,
    // Event handler to notify the parent when the edit button is clicked.
    on_edit: EventHandler<usize>,
    // Flag to disable the buttons.
    disabled: bool,
) -> Element {
    let full_address = recipient.address.to_bech32m(Network::Main).unwrap();
    let abbreviated_address = recipient.address.to_display_bech32m_abbreviated(Network::Main).unwrap();

    rsx! {
        tr {
            td {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        code { "{abbreviated_address}" }
                        div {
                            role: "group",
                            CopyButton { text_to_copy: full_address }
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| on_edit.call(index),
                                disabled,
                                "Edit"
                            }
                            Button {
                                button_type: ButtonType::Contrast,
                                outline: true,
                                on_click: move |_| on_delete.call(index),
                                disabled,
                                "X"
                            }
                        }
                    }
                    div {
                        style: "font-weight: bold;",
                        "{recipient.amount}"
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

    // --- Form State ---
    let mut current_address = use_signal::<Option<Rc<ReceivingAddress>>>(|| None);
    let mut current_amount = use_signal(String::new);
    let mut current_fee = use_signal(String::new);
    let mut editing_index = use_signal::<Option<usize>>(|| None);

    // --- Validation State ---
    let mut address_error = use_signal::<Option<String>>(|| None);
    let mut amount_error = use_signal::<Option<String>>(|| None);
    let mut fee_error = use_signal::<Option<String>>(|| None);

    // --- Main State ---
    let mut recipients = use_signal::<Vec<Recipient>>(Vec::new);
    let mut api_response = use_signal::<Option<String>>(|| None);
    let mut is_form_disabled = use_signal(|| false);

    // --- Modal State ---
    let mut is_confirm_modal_open = use_signal(|| false);
    let mut is_qr_modal_open = use_signal(|| false);
    let mut show_duplicate_warning_modal = use_signal(|| false);
    let mut suppress_duplicate_warning = use_signal(|| false);
    let mut pending_recipient = use_signal::<Option<Recipient>>(|| None);

    // --- Derived State ---
    let subtotal = use_memo(move || {
        recipients.read().iter().map(|r| r.amount).sum::<NativeCurrencyAmount>()
    });
    let total_spend = use_memo(move || {
        let fee = NativeCurrencyAmount::coins_from_str(&current_fee()).unwrap_or_else(|_| NativeCurrencyAmount::zero());
        subtotal() + fee
    });

    // --- Event Handlers ---

    let handle_paste_address = move || {
        spawn(async move {
            let clipboard = web_sys::window().unwrap().navigator().clipboard();
            let promise = clipboard.read_text();

            if let Ok(js_text) = wasm_bindgen_futures::JsFuture::from(promise).await {
                let clipboard_text = js_text.as_string().unwrap_or_default();

                match ReceivingAddress::from_bech32m(&clipboard_text, network) {
                    Ok(addr) => {
                        *current_address.write() = Some(Rc::new(addr));
                        *address_error.write() = None;
                    }
                    Err(err) => {
                        *current_address.write() = None;
                        *address_error.write() = Some(format!("Invalid Address: {}", err));
                    }
                }
            }
        });
    };

    let mut clear_form = move || {
        current_address.set(None);
        current_amount.set(String::new());
        address_error.set(None);
        amount_error.set(None);
        editing_index.set(None);
    };

    let mut reset_screen = move || {
        clear_form();
        recipients.set(Vec::new());
        current_fee.set(String::new());
        api_response.set(None);
        is_form_disabled.set(false);
        suppress_duplicate_warning.set(false);
        wizard_step.set(WizardStep::AddRecipients);
    };

    let mut add_or_update_recipient = move |recipient_to_add: Recipient| {
        if let Some(index) = editing_index() {
            recipients.with_mut(|r| r[index] = recipient_to_add);
        } else {
            recipients.push(recipient_to_add);
        }
        clear_form();
    };

    let mut handle_add_recipient = move || {
        let mut is_valid = true;

        if current_address().is_none() {
            address_error.set(Some("Address is required.".to_string()));
            is_valid = false;
        }
        let amount = match NativeCurrencyAmount::coins_from_str(&current_amount()) {
            Ok(amt) if amt > NativeCurrencyAmount::zero() => {
                amount_error.set(None);
                Some(amt)
            },
            _ => {
                amount_error.set(Some("Invalid amount.".to_string()));
                is_valid = false;
                None
            }
        };

        if !is_valid { return; }

        let new_recipient = Recipient {
            address: current_address().unwrap(),
            amount: amount.unwrap(),
        };

        if !suppress_duplicate_warning() {
            let is_duplicate = recipients.read().iter().enumerate().any(|(i, r)| {
                if Some(i) == editing_index() {
                    false
                } else {
                    r.address == new_recipient.address
                }
            });

            if is_duplicate {
                pending_recipient.set(Some(new_recipient));
                show_duplicate_warning_modal.set(true);
                return;
            }
        }
        add_or_update_recipient(new_recipient);
    };

    let address_value = if let Some(addr) = current_address() {
        addr.to_display_bech32m_abbreviated(network).unwrap()
    } else {
        "".to_string()
    };

    rsx! {
        // --- Modals ---
        Modal {
            is_open: is_confirm_modal_open,
            title: "Confirm Transaction".to_string(),
            p { "Please review the details below. This action cannot be undone." }
            h5 { style: "margin-top: 1rem;", "Recipients:" }
            table {
                role: "grid",
                tbody {
                    {recipients.read().iter().map(|recipient| rsx!{
                        tr {
                            td { code { "{recipient.address.to_display_bech32m_abbreviated(network).unwrap()}" } }
                            td { style: "text-align: right;", "{recipient.amount}" }
                        }
                    })}
                }
            }
            hr {}
            p { strong { "Fee: " } "{current_fee()}" }
            p { strong { "Total Spend: " } "{total_spend}" }
            footer {
                Button {
                    button_type: ButtonType::Secondary,
                    outline: true,
                    on_click: move |_| is_confirm_modal_open.set(false),
                    "Cancel"
                }
                Button {
                    on_click: move |_| {
                        is_confirm_modal_open.set(false);
                        api_response.set(None);
                        spawn(async move {
                            let result: Result<String, String> = Ok("Transaction Sent!".to_string());
                            let message = match result {
                                Ok(msg) => {
                                    is_form_disabled.set(true);
                                    wizard_step.set(WizardStep::Status);
                                    format!("Success: {}", msg)
                                },
                                Err(err) => format!("API Error: {}", err),
                            };
                            api_response.set(Some(message));
                        });
                    },
                    "Confirm & Send"
                }
            }
        }
        NoTitleModal {
            is_open: is_qr_modal_open,
            h3 { "QR Scanner Placeholder" }
            p { "The QR scanner would appear here." }
        }
        Modal {
            is_open: show_duplicate_warning_modal,
            title: "Duplicate Address".to_string(),
            p { "This address is already in the recipient list. Are you sure you want to proceed?" }
            div {
                style: "margin-top: 1rem; margin-bottom: 1rem;",
                label {
                    input {
                        r#type: "checkbox",
                        checked: "{suppress_duplicate_warning}",
                        oninput: move |event| suppress_duplicate_warning.set(event.value() == "true"),
                    }
                    "Don't ask me again"
                }
            }
            footer {
                Button {
                    button_type: ButtonType::Secondary,
                    outline: true,
                    on_click: move |_| {
                        show_duplicate_warning_modal.set(false);
                        pending_recipient.set(None);
                    },
                    "Cancel"
                }
                Button {
                    on_click: move |_| {
                        show_duplicate_warning_modal.set(false);
                        if let Some(recipient) = pending_recipient.take() {
                            add_or_update_recipient(recipient);
                        }
                    },
                    "Confirm"
                }
            }
        }

        // --- Wizard Content ---
        div {
            match wizard_step() {
                WizardStep::AddRecipients => rsx! {
                    Card {
                        h3 { "Send Funds" }
                        div {
                            label { "Recipient Address" }
                            div {
                                role: "group",
                                Input {
                                    label: "".to_string(),
                                    name: "address_display".to_string(),
                                    placeholder: "Paste or scan an address...".to_string(),
                                    value: address_value,
                                    readonly: true,
                                }
                                Button {
                                    button_type: ButtonType::Secondary,
                                    outline: true,
                                    on_click: move|_| handle_paste_address(),
                                    "Paste Address"
                                }
                                Button {
                                    button_type: ButtonType::Secondary,
                                    outline: true,
                                    on_click: move |_| is_qr_modal_open.set(true),
                                    "Scan QR"
                                }
                            }
                            if let Some(err) = address_error() {
                                small { style: "color: var(--pico-color-red-500);", "{err}" }
                            }
                            Input {
                                label: "Amount".to_string(),
                                name: "amount",
                                input_type: "number".to_string(),
                                placeholder: "0.0".to_string(),
                                value: "{current_amount}",
                                on_input: move |event: FormEvent| current_amount.set(event.value().clone()),
                            }
                            if let Some(err) = amount_error() {
                                small { style: "color: var(--pico-color-red-500);", "{err}" }
                            }
                            div {
                                style: "margin-top: 1rem;",
                                Button {
                                    on_click: move |_| handle_add_recipient(),
                                    if editing_index().is_some() { "Update Recipient" } else { "Add Recipient" }
                                }
                            }
                        }
                    }
                    if !recipients.read().is_empty() {
                        Card {
                            h4 { "Recipients" }
                            table {
                                tbody {
                                    {recipients.iter().enumerate().map(|(i, recipient)| rsx!{
                                        RecipientRow {
                                            key: "{i}",
                                            index: i,
                                            recipient: recipient.clone(),
                                            disabled: false,
                                            on_delete: move |index: usize| { recipients.write().remove(index); },
                                            on_edit: move |index| {
                                                let recipient_to_edit: Recipient = Clone::clone(&recipients.read()[index]);
                                                current_address.set(Some(recipient_to_edit.address));
                                                current_amount.set(recipient_to_edit.amount.to_string());
                                                editing_index.set(Some(index));
                                            }
                                        }
                                    })}
                                }
                            }
                            if recipients.read().len() > 1 {
                                hr {}
                                h5 { style: "text-align: right;", "Subtotal: {subtotal}" }
                            }
                        }
                    }
                    Card {
                        Input {
                            label: "Fee".to_string(),
                            name: "fee",
                            input_type: "number".to_string(),
                            placeholder: "0.0".to_string(),
                            value: "{current_fee}",
                            on_input: move |event: FormEvent| current_fee.set(event.value().clone()),
                        }
                        if let Some(err) = fee_error() {
                            small { style: "color: var(--pico-color-red-500);", "{err}" }
                        }
                        h4 { style: "margin-top: 1rem; text-align: right;", "Total Spend: {total_spend}" }
                        Button {
                            on_click: move |_| {
                                let mut is_valid = true;
                                if recipients.read().is_empty() { is_valid = false; }
                                if NativeCurrencyAmount::coins_from_str(&current_fee()).is_err() {
                                    fee_error.set(Some("Invalid fee.".to_string()));
                                    is_valid = false;
                                } else { fee_error.set(None); }
                                if is_valid { wizard_step.set(WizardStep::Review); }
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
                                {recipients.read().iter().map(|recipient| rsx!{
                                    tr {
                                        td { code { "{recipient.address.to_display_bech32m_abbreviated(network).unwrap()}" } }
                                        td { style: "text-align: right;", "{recipient.amount}" }
                                    }
                                })}
                            }
                        }
                        hr {}
                        p { strong { "Fee: " } "{current_fee()}" }
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
                                on_click: move |_| is_confirm_modal_open.set(true),
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
