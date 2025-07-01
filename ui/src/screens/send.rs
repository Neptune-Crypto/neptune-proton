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
) -> Element {
    let full_address = recipient.address.to_bech32m(Network::Main).unwrap();
    let abbreviated_address = recipient.address.to_display_bech32m_abbreviated(Network::Main).unwrap();

    rsx! {
        tr {
            td { code { "{abbreviated_address}" } }
            td { "{recipient.amount}" }
            td {
                style: "text-align: right;",
                div {
                    role: "group",
                    CopyButton { text_to_copy: full_address }
                    Button {
                        button_type: ButtonType::Secondary,
                        outline: true,
                        on_click: move |_| on_edit.call(index),
                        "Edit"
                    }
                    Button {
                        button_type: ButtonType::Contrast,
                        outline: true,
                        on_click: move |_| on_delete.call(index),
                        "X"
                    }
                }
            }
        }
    }
}


#[component]
pub fn SendScreen() -> Element {
    let network = use_context::<AppState>().network;

    // --- Form State ---
    // Holds the data currently being entered into the form.
    let mut current_address = use_signal::<Option<Rc<ReceivingAddress>>>(|| None);
    let mut current_amount = use_signal(String::new);
    let mut current_fee = use_signal(String::new);
    // Holds the index of the recipient being edited.
    let mut editing_index = use_signal::<Option<usize>>(|| None);

    // --- Validation State ---
    // Holds error messages for the form fields.
    let mut address_error = use_signal::<Option<String>>(|| None);
    let mut amount_error = use_signal::<Option<String>>(|| None);
    let mut fee_error = use_signal::<Option<String>>(|| None);

    // --- Main State ---
    // The list of recipients that have been added to the transaction.
    let mut recipients = use_signal::<Vec<Recipient>>(Vec::new);
    // The final API response after sending the transaction.
    let mut api_response = use_signal::<Option<String>>(|| None);

    // --- Modal State ---
    let mut is_confirm_modal_open = use_signal(|| false);
    let mut is_qr_modal_open = use_signal(|| false);
    // New state for the duplicate address warning feature
    let mut show_duplicate_warning_modal = use_signal(|| false);
    let mut suppress_duplicate_warning = use_signal(|| false);
    let mut pending_recipient = use_signal::<Option<Recipient>>(|| None);


    // --- Event Handlers ---

    let handle_paste_address = move |_| {
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

    // Resets the form to its initial state.
    let mut clear_form = move || {
        current_address.set(None);
        current_amount.set(String::new());
        address_error.set(None);
        amount_error.set(None);
        editing_index.set(None);
    };

    // Helper function to add/update a recipient to avoid duplicated code.
    let mut add_or_update_recipient = move |recipient_to_add: Recipient| {
        if let Some(index) = editing_index() {
            recipients.with_mut(|r| r[index] = recipient_to_add);
        } else {
            recipients.push(recipient_to_add);
        }
        clear_form();
    };


    // Handles adding a new recipient to the list.
    let mut handle_add_recipient = move || {
        let mut is_valid = true;

        // --- Validation ---
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

        // --- New Duplicate Check Logic ---
        if !suppress_duplicate_warning() {
            let is_duplicate = recipients.read().iter().enumerate().any(|(i, r)| {
                // Ignore the recipient if it's the one we are currently editing.
                if Some(i) == editing_index() {
                    false
                } else {
                    r.address == new_recipient.address
                }
            });

            if is_duplicate {
                // A duplicate was found, show the modal and wait for user action.
                pending_recipient.set(Some(new_recipient));
                show_duplicate_warning_modal.set(true);
                return;
            }
        }

        // If no duplicate is found (or warnings are suppressed), proceed directly.
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
            // TODO: Display transaction details here.
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
                            // TODO: Replace with actual server call `api::send_transaction(...)`
                            let result: Result<String, String> = Ok("Transaction Sent!".to_string());
                            let message = match result {
                                Ok(msg) => format!("Success: {}", msg),
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
            // TODO: Implement QR code scanning UI here
            h3 { "QR Scanner Placeholder" }
            p { "The QR scanner would appear here." }
        }
        // --- New Duplicate Address Warning Modal ---
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
                        oninput: move |event| {
                            suppress_duplicate_warning.set(event.value() == "true")
                        },
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
                        pending_recipient.set(None); // Important: Clear the pending data
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

        // --- Main Content ---
        Card {
            h3 { "Send Funds" }

            // --- Recipient Entry Form ---
            div {
                // Address Input Area
                label { "Recipient Address" }
                div {
                    role: "group",
                    // The address is not directly editable, only shown as an abbreviation.
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
                        on_click: handle_paste_address,
                        "Paste"
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

                // Amount Input
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

                // "Add Recipient" Button
                div {
                    style: "margin-top: 1rem;",
                    Button {
                        on_click: move |_| {
                            handle_add_recipient()
                        },
                        if editing_index().is_some() { "Update Recipient" } else { "Add Recipient" }
                    }
                }
            }
        }

        // --- Recipients Grid ---
        if !recipients.read().is_empty() {
            Card {
                h4 { "Recipients" }
                table {
                    thead {
                        tr {
                            th { "Address" }
                            th { "Amount" }
                            th {}
                        }
                    }
                    tbody {
                        {recipients.iter().enumerate().map(|(i, recipient)| rsx!{
                            RecipientRow {
                                key: "{i}",
                                index: i,
                                recipient: recipient.clone(),
                                on_delete: move |index| {
                                    recipients.write().remove(index);
                                },
                                on_edit: move |index| {
                                    let recipient_to_edit: Recipient = Clone::clone(&recipients.read()[index]);
                                    // let recipient_to_edit: Recipient = recipients.read()[index].clone();
                                    current_address.set(Some(recipient_to_edit.address));
                                    current_amount.set(recipient_to_edit.amount.to_string());
                                    editing_index.set(Some(index));
                                }
                            }
                        })}
                    }
                }
            }
        }

        // --- Fee and Final Send ---
        Card {
            Grid {
                Input {
                    label: "Fee".to_string(),
                    name: "fee",
                    input_type: "number".to_string(),
                    placeholder: "0.0".to_string(),
                    value: "{current_fee}",
                    on_input: move |event: FormEvent| current_fee.set(event.value().clone()),
                }
                div {}
            }
            if let Some(err) = fee_error() {
                small { style: "color: var(--pico-color-red-500);", "{err}" }
            }

            Button {
                on_click: move |_| {
                    // Final validation before opening confirm modal
                    let mut is_valid = true;
                    if recipients.read().is_empty() {
                        // TODO: Show an error message that at least one recipient is required.
                        is_valid = false;
                    }
                    if NativeCurrencyAmount::coins_from_str(&current_fee()).is_err() {
                        fee_error.set(Some("Invalid fee.".to_string()));
                        is_valid = false;
                    } else {
                        fee_error.set(None);
                    }

                    if is_valid {
                        is_confirm_modal_open.set(true);
                    }
                },
                "Review Transaction"
            }
        }

        // --- API Response Area ---
        if let Some(response) = api_response() {
            Card {
                h3 { "Transaction Status" }
                p { "{response}" }
            }
        }
    }
}
