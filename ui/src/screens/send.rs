//=============================================================================
// File: src/screens/send.rs
//=============================================================================
use crate::components::pico::{
    Button, ButtonType, Card, CloseButton, CopyButton, Grid, Input, Modal, NoTitleModal,
};
use crate::AppState;
use dioxus::dioxus_core::SpawnIfAsync;
use dioxus::prelude::*;
use neptune_types::address::{KeyType, ReceivingAddress};
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use num_traits::Zero;
use std::collections::HashSet;
use std::rc::Rc;
use std::str::FromStr;

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

impl EditableRecipient {
    /// Checks if the current editable data is valid.
    fn is_valid(&self, network: Network) -> bool {
        ReceivingAddress::from_bech32m(&self.address_str, network).is_ok()
            && match NativeCurrencyAmount::coins_from_str(&self.amount_str) {
                Ok(amt) => amt > NativeCurrencyAmount::zero(),
                Err(_) => false,
            }
    }
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

/// An editable row in the recipient grid.
#[component]
fn EditableRecipientRow(
    index: usize,
    mut recipient: Signal<EditableRecipient>,
    on_delete: EventHandler<usize>,
    on_open_address_actions: EventHandler<usize>,
    can_delete: bool,
    is_active: bool,
    on_set_active: EventHandler<usize>,
) -> Element {
    let network = use_context::<AppState>().network;

    // Show the abbreviated address if valid, otherwise show the raw input.
    let display_address = use_memo(move || {
        let r = recipient.read();
        match ReceivingAddress::from_bech32m(&r.address_str, network) {
            Ok(addr) => addr
                .to_display_bech32m_abbreviated(network)
                .unwrap_or(r.address_str.clone()),
            Err(_) => r.address_str.clone(),
        }
    });

    rsx! {
        div {
            class: if is_active { "recipient-row active" } else { "recipient-row" },
            style: "border: 1px solid var(--pico-form-element-border-color); border-radius: var(--pico-border-radius); padding: 1rem; margin-bottom: 1rem;",

            // Address Row
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;",
                label {
                    style: "margin-bottom: 0;",
                    "Recipient Address"
                }
                if can_delete {
                    CloseButton {
                        on_click: move |event: MouseEvent| {
                            event.stop_propagation();
                            on_delete.call(index);
                        }
                    }
                }
            }
            div {
                // This input is now just a trigger for the actions modal.
                Input {
                    label: "".to_string(),
                    name: "address_{index}",
                    placeholder: "Click to paste or scan an address...",
                    value: "{display_address}",
                    readonly: true,
                    on_click: move |event: MouseEvent| {
                        if is_active {
                            event.stop_propagation();
                            on_open_address_actions.call(index);
                        }
                    },
                    style: if is_active { "cursor: pointer;" } else { "cursor: not-allowed;" }
                }
            }
            if let Some(err) = &recipient.read().address_error {
                small { style: "color: var(--pico-color-red-500);", "{err}" }
            }


            // Amount Row
            div {
                style: "display: flex; gap: 1rem; align-items: flex-end;",
                div {
                    style: "flex-grow: 1;",
                    label { "Amount" }
                    Input {
                        label: "".to_string(),
                        name: "amount_{index}",
                        input_type: "number".to_string(),
                        placeholder: "0.0",
                        value: "{recipient.read().amount_str}",
                        readonly: !is_active,
                        on_input: move |event: FormEvent| {
                            if is_active {
                                recipient.with_mut(|r| {
                                    r.amount_str = event.value().clone();
                                    // Real-time validation
                                    match NativeCurrencyAmount::coins_from_str(&r.amount_str) {
                                        Ok(amt) if amt > NativeCurrencyAmount::zero() => r.amount_error = None,
                                        _ => r.amount_error = Some("Invalid amount".to_string()),
                                    }
                                });
                            }
                        }
                    }
                    if let Some(err) = &recipient.read().amount_error {
                        small { style: "color: var(--pico-color-red-500);", "{err}" }
                    }
                }
                if !is_active {
                    div {
                        Button {
                           button_type: ButtonType::Secondary,
                           outline: true,
                           on_click: move |evt: MouseEvent| {
                               evt.stop_propagation();
                               on_set_active.call(index);
                           },
                           "Edit"
                        }
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
    let mut recipients = use_signal(move || vec![Signal::new(EditableRecipient::default())]);
    let mut fee_str = use_signal(String::new);
    let mut api_response = use_signal::<Option<String>>(|| None);
    let mut active_row_index = use_signal(|| 0);

    // --- Modal State ---
    let mut is_address_actions_modal_open = use_signal(|| false);
    let mut action_target_index = use_signal::<Option<usize>>(|| None);
    let mut is_qr_modal_open = use_signal(|| false);
    let mut show_error_modal = use_signal(|| false);
    let mut error_modal_message = use_signal(String::new);
    let mut show_duplicate_warning_modal = use_signal(|| false);
    let mut suppress_duplicate_warning = use_signal(|| false);
    let mut pending_address = use_signal::<Option<String>>(|| None);


    // --- Validation State ---
    let mut fee_error = use_signal::<Option<String>>(|| None);

    // --- Derived State ---
    let subtotal = use_memo(move || {
        recipients
            .read()
            .iter()
            .fold(NativeCurrencyAmount::zero(), |acc, r| {
                let amount = NativeCurrencyAmount::coins_from_str(&r.read().amount_str)
                    .unwrap_or_else(|_| NativeCurrencyAmount::zero());
                acc + amount
            })
    });
    let total_spend = use_memo(move || {
        let fee = NativeCurrencyAmount::coins_from_str(&fee_str())
            .unwrap_or_else(|_| NativeCurrencyAmount::zero());
        subtotal() + fee
    });
    let is_active_row_valid = use_memo(move || {
        recipients
            .read()
            .get(active_row_index())
            .map_or(false, |r| r.read().is_valid(network))
    });
    let is_form_fully_valid = use_memo(move || {
        let recs = recipients.read();
        if recs.is_empty() {
            return false;
        }
        let all_recipients_valid = recs.iter().all(|r| r.read().is_valid(network));
        let fee_is_valid =
            fee_str.read().is_empty() || NativeCurrencyAmount::coins_from_str(&fee_str()).is_ok();
        all_recipients_valid && fee_is_valid
    });

    // --- Event Handlers ---
    let mut reset_screen = move || {
        recipients.set(vec![Signal::new(EditableRecipient::default())]);
        active_row_index.set(0);
        fee_str.set(String::new());
        fee_error.set(None);
        api_response.set(None);
        suppress_duplicate_warning.set(false);
        wizard_step.set(WizardStep::AddRecipients);
    };

    rsx! {
        // --- Modals ---
        NoTitleModal {
            is_open: is_address_actions_modal_open,
            div {
                style: "display: flex; flex-direction: column; gap: 1rem;",
                h3 { "Set Address" }
                p {
                    if let Some(index) = action_target_index() {
                        "Choose an action for recipient number {index + 1}."
                    } else {
                        "Choose an action."
                    }
                }
                Button {
                    on_click: move |_| {
                        if let Some(index) = action_target_index() {
                            let mut target_recipient = recipients.read()[index];
                            spawn(async move {
                                if let Ok(js_text) = wasm_bindgen_futures::JsFuture::from(web_sys::window().unwrap().navigator().clipboard().read_text()).await {
                                    let clipboard_text = js_text.as_string().unwrap_or_default();
                                    // Validate before setting
                                    match ReceivingAddress::from_bech32m(&clipboard_text, network) {
                                        Ok(_) => {
                                            // Check for duplicates before committing the paste
                                            let is_duplicate = recipients.read().iter().enumerate().any(|(i, r)| {
                                                if i == index { false } else { r.read().address_str == clipboard_text }
                                            });

                                            if is_duplicate && !suppress_duplicate_warning() {
                                                pending_address.set(Some(clipboard_text));
                                                show_duplicate_warning_modal.set(true);
                                            } else {
                                                target_recipient.with_mut(|r| {
                                                    r.address_str = clipboard_text;
                                                    r.address_error = None;
                                                });
                                            }
                                        },
                                        Err(e) => {
                                            error_modal_message.set(format!("Invalid Address: {}", e));
                                            show_error_modal.set(true);
                                        }
                                    }
                                }
                            });
                        }
                        is_address_actions_modal_open.set(false);
                    },
                    "Paste Address"
                }
                Button {
                    on_click: move |_| {
                        is_address_actions_modal_open.set(false);
                        is_qr_modal_open.set(true);
                    },
                    "Scan QR Code"
                }
                 Button {
                    button_type: ButtonType::Secondary,
                    outline: true,
                    on_click: move |_| is_address_actions_modal_open.set(false),
                    "Cancel"
                }
            }
        }
        NoTitleModal {
            is_open: is_qr_modal_open,
            h3 { "QR Scanner Placeholder" }
            p {
                if let Some(index) = action_target_index() {
                    "When a QR code is scanned, its value should be set for recipient number {index + 1}."
                } else {""}
            }
            Button {
                on_click: move |_| is_qr_modal_open.set(false),
                "Close"
            }
        }
        Modal {
            is_open: show_error_modal,
            title: "Error".to_string(),
            p { "{error_modal_message}" }
            footer {
                Button { on_click: move |_| show_error_modal.set(false), "Close" }
            }
        }
        Modal {
            is_open: show_duplicate_warning_modal,
            title: "Duplicate Address".to_string(),
            p { "This address is already in the recipient list. Do you want to add it again?" }
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
                    on_click: move |_| show_duplicate_warning_modal.set(false),
                    "Cancel"
                }
                Button {
                    on_click: move |_| {
                        if let (Some(addr), Some(index)) = (pending_address.take(), action_target_index()) {
                             // Step 1: Get a copy of the signal and release the read lock.
                             let mut target_recipient = recipients.read()[index];

                             // Step 2: Now that the lock is released, modify the copy.
                             target_recipient.with_mut(|r| {
                                r.address_str = addr;
                                r.address_error = None;
                            });
                        }
                        show_duplicate_warning_modal.set(false);
                    },
                    "Proceed Anyway"
                }
            }
        }


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
                                is_active: active_row_index() == i,
                                on_delete: move |index_to_delete: usize| {
                                    if recipients.len() > 1 {
                                        if active_row_index() >= index_to_delete && active_row_index() > 0 {
                                            active_row_index.set(active_row_index() - 1);
                                        }
                                        recipients.write().remove(index_to_delete);
                                    }
                                },
                                on_open_address_actions: move |index: usize| {
                                    if active_row_index() == index {
                                        action_target_index.set(Some(index));
                                        is_address_actions_modal_open.set(true);
                                    }
                                },
                                on_set_active: move |index: usize| {
                                    if is_active_row_valid() {
                                        active_row_index.set(index);
                                    }
                                },
                                can_delete: recipients.len() > 1,
                            }
                        }

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-top: 1rem;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| {
                                    let new_index = recipients.len();
                                    recipients.write().push(Signal::new(EditableRecipient::default()));
                                    active_row_index.set(new_index);
                                },
                                disabled: !is_active_row_valid(),
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
                            on_input: move |event: FormEvent| {
                                fee_str.set(event.value().clone());
                                if NativeCurrencyAmount::coins_from_str(&event.value()).is_err() && !event.value().is_empty() {
                                    fee_error.set(Some("Invalid fee.".to_string()));
                                } else {
                                    fee_error.set(None);
                                }
                            },
                        }
                        if let Some(err) = fee_error() {
                            small { style: "color: var(--pico-color-red-500);", "{err}" }
                        }
                        h4 { style: "margin-top: 1rem; text-align: right;", "Total Spend: {total_spend}" }
                        Button {
                            on_click: move |_| {
                                if is_form_fully_valid() {
                                    wizard_step.set(WizardStep::Review);
                                }
                            },
                            disabled: !is_form_fully_valid(),
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
                                    // These unwraps are safe because we validated on the previous screen.
                                    let addr = ReceivingAddress::from_bech32m(&recipient.address_str, network).unwrap();
                                    let amount = NativeCurrencyAmount::coins_from_str(&recipient.amount_str).unwrap();
                                    rsx! {
                                        tr {
                                            td { code { "{addr.to_display_bech32m_abbreviated(network).unwrap()}" } }
                                            td { style: "text-align: right;", "{amount}" }
                                        }
                                    }
                                })}
                            }
                        }
                        hr {}
                        p {
                            strong { "Fee: " }
                            if fee_str().is_empty() {
                                "0.0"
                            } else {
                                "{fee_str()}"
                            }
                        }
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