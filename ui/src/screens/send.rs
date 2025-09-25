//=============================================================================
// File: src/screens/send.rs
//=============================================================================
use crate::components::address::Address;
use crate::components::pico::{
    Button, ButtonType, Card, CloseButton, CopyButton, Input, Modal, NoTitleModal,
};
use crate::components::qr_scanner::QrScanner;
use crate::components::qr_uploader::QrUploader;
use crate::AppState;
use crate::Screen;
use dioxus::prelude::*;
use neptune_types::address::ReceivingAddress;
use neptune_types::change_policy::ChangePolicy;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use neptune_types::output_format::OutputFormat;
use neptune_types::transaction_details::TransactionDetails;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use num_traits::Zero;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
struct Recipient {
    address: Rc<ReceivingAddress>,
    amount: NativeCurrencyAmount,
}
impl From<Recipient> for OutputFormat {
    fn from(r: Recipient) -> Self {
        ((*r.address).clone(), r.amount).into()
    }
}
#[derive(Clone, PartialEq, Debug)]
struct EditableRecipient {
    address_str: String,
    amount_str: String,
    address_error: Option<String>,
    amount_error: Option<String>,
}
impl EditableRecipient {
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

#[component]
fn EditableRecipientRow(
    index: usize,
    mut recipient: Signal<EditableRecipient>,
    on_delete: EventHandler<usize>,
    on_open_address_actions: EventHandler<usize>,
    can_delete: bool,
    is_active: bool,
    on_set_active: EventHandler<usize>,
    on_done_editing: EventHandler<()>,
    is_any_other_row_active: bool,
) -> Element {
    let network = use_context::<AppState>().network;
    let parsed_address = use_memo(move || {
        ReceivingAddress::from_bech32m(&recipient.read().address_str, network).ok()
    });
    let display_address = use_memo(move || match parsed_address() {
        Some(addr) => addr
            .to_display_bech32m_abbreviated(network)
            .unwrap_or(recipient.read().address_str.clone()),
        None => recipient.read().address_str.clone(),
    });
    let is_row_valid = use_memo(move || {
        let is_address_ok = parsed_address().is_some();
        let is_amount_ok = match NativeCurrencyAmount::coins_from_str(&recipient.read().amount_str)
        {
            Ok(amt) => amt > NativeCurrencyAmount::zero(),
            Err(_) => false,
        };
        is_address_ok && is_amount_ok
    });

    rsx! {
        div {
            class: if is_active { "recipient-row active" } else { "recipient-row" },
            style: "border: 1px solid var(--pico-form-element-border-color); border-radius: var(--pico-border-radius); padding: 1rem; margin-bottom: 1rem;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;",
                label {
                    style: "margin-bottom: 0;",
                    if is_active { "Recipient Address" } else { "Recipient" }
                }
                div {
                    style: "display: flex; gap: 0.5rem; align-items: center;",
                    if is_active {
                        Button {
                           button_type: ButtonType::Primary,
                           on_click: move |evt: MouseEvent| {
                               evt.stop_propagation();
                               on_done_editing.call(());
                           },
                           disabled: !is_row_valid(),
                           "Done"
                        }
                    } else {
                        Button {
                           button_type: ButtonType::Secondary,
                           outline: true,
                           on_click: move |evt: MouseEvent| {
                               evt.stop_propagation();
                               on_set_active.call(index);
                           },
                           disabled: is_any_other_row_active,
                           "Edit"
                        }
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
            }
            if is_active {
                div {
                    key: "active-form-{index}",
                    div {
                        Input {
                            label: "".to_string(),
                            name: "address_{index}",
                            placeholder: "Click to paste or scan an address...",
                            value: "{display_address}",
                            readonly: true,
                            on_click: move |event: MouseEvent| {
                                event.stop_propagation();
                                on_open_address_actions.call(index);
                            },
                            style: "cursor: pointer;"
                        }
                    }
                    if let Some(err) = &recipient.read().address_error {
                        small { style: "color: var(--pico-color-red-500);", "{err}" }
                    }
                    div {
                        style: "margin-top: 0.75rem;",
                        label { "Amount" }
                        Input {
                            label: "".to_string(),
                            name: "amount_{index}",
                            input_type: "number".to_string(),
                            placeholder: "0.0",
                            value: "{recipient.read().amount_str}",
                            readonly: false,
                            on_input: move |event: FormEvent| {
                                recipient.with_mut(|r| {
                                    r.amount_str = event.value().clone();
                                    match NativeCurrencyAmount::coins_from_str(&r.amount_str) {
                                        Ok(amt) if amt > NativeCurrencyAmount::zero() => r.amount_error = None,
                                        _ => r.amount_error = Some("Invalid amount".to_string()),
                                    }
                                });
                            }
                        }
                        if let Some(err) = &recipient.read().amount_error {
                            small { style: "color: var(--pico-color-red-500);", "{err}" }
                        }
                    }
                }
            } else {
                div {
                    key: "inactive-display-{index}",
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0.25rem;",
                    div {
                        if let Some(addr) = parsed_address() {
                             Address { address: Rc::new(addr) }
                        } else {
                             code { "{display_address}" }
                        }
                    }
                    strong { "{recipient.read().amount_str}" }
                }
            }
        }
    }
}

#[component]
pub fn SendScreen() -> Element {
    let network = use_context::<AppState>().network;

    #[derive(PartialEq, Clone, Copy)]
    enum WizardStep {
        AddRecipients,
        Review,
        Status,
    }
    let mut wizard_step = use_signal(|| WizardStep::AddRecipients);
    let mut api_response = use_signal::<
        Option<Result<(TransactionKernelId, TransactionDetails), ServerFnError>>,
    >(|| None);
    let mut recipients = use_signal(move || vec![Signal::new(EditableRecipient::default())]);
    let mut fee_str = use_signal(String::new);
    let mut active_row_index = use_signal::<Option<usize>>(|| Some(0));
    let mut is_address_actions_modal_open = use_signal(|| false);
    let mut action_target_index = use_signal::<Option<usize>>(|| None);
    let mut is_qr_scanner_modal_open = use_signal(|| false);
    let mut is_qr_upload_modal_open = use_signal(|| false);
    let mut show_error_modal = use_signal(|| false);
    let mut error_modal_message = use_signal(String::new);
    let mut show_duplicate_warning_modal = use_signal(|| false);
    let mut suppress_duplicate_warning = use_signal(|| false);
    let mut pending_address = use_signal::<Option<String>>(|| None);
    let mut fee_error = use_signal::<Option<String>>(|| None);

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
    let is_any_row_active = use_memo(move || active_row_index().is_some());
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

    let mut reset_screen = move || {
        recipients.set(vec![Signal::new(EditableRecipient::default())]);
        active_row_index.set(Some(0));
        fee_str.set(String::new());
        fee_error.set(None);
        api_response.set(None);
        suppress_duplicate_warning.set(false);
        wizard_step.set(WizardStep::AddRecipients);
    };

    let mut active_screen = use_context::<Signal<Screen>>();

    let mut handle_scanned_data = move |scanned_text: String| {
        if let Some(index) = action_target_index() {
            let mut target_recipient = recipients.read()[index];
            match ReceivingAddress::from_bech32m(&scanned_text, network) {
                Ok(_) => {
                    let is_duplicate = recipients.read().iter().enumerate().any(|(i, r)| {
                        if i == index {
                            false
                        } else {
                            r.read().address_str == scanned_text
                        }
                    });
                    if is_duplicate && !suppress_duplicate_warning() {
                        pending_address.set(Some(scanned_text));
                        show_duplicate_warning_modal.set(true);
                    } else {
                        target_recipient.with_mut(|r| {
                            r.address_str = scanned_text;
                            r.address_error = None;
                        });
                    }
                }
                Err(e) => {
                    error_modal_message.set(format!(
                        "Invalid Address from QR: {}.  found: {}",
                        e, scanned_text
                    ));
                    show_error_modal.set(true);
                }
            }
        }
    };

    rsx! {
        NoTitleModal {
            is_open: is_address_actions_modal_open,
            div {
                style: "display: flex; flex-direction: column; gap: 1rem;",
                h3 { "Set Address" },
                p {
                    if let Some(index) = action_target_index() {
                        "Choose an action for recipient number {index + 1}."
                    } else {
                        "Choose an action."
                    }
                },
                Button {
                    on_click: move |_| {
                        if let Some(_index) = action_target_index() {
                            spawn(async move {
                                if let Some(clipboard_text) = crate::compat::clipboard_get().await {
                                    handle_scanned_data(clipboard_text);
                                }
                            });
                        }
                        is_address_actions_modal_open.set(false);
                    },
                    "Paste Address"
                },
                Button {
                    on_click: move |_| {
                        is_address_actions_modal_open.set(false);
                        is_qr_scanner_modal_open.set(true);
                    },
                    "Scan QR Code"
                },
                Button {
                    on_click: move |_| {
                        is_address_actions_modal_open.set(false);
                        is_qr_upload_modal_open.set(true);
                    },
                    "Upload QR Image"
                },
                Button {
                    button_type: ButtonType::Secondary,
                    outline: true,
                    on_click: move |_| is_address_actions_modal_open.set(false),
                    "Cancel"
                }
            }
        },
        NoTitleModal {
            is_open: is_qr_scanner_modal_open,
            QrScanner {
                on_scan: move |data| {
                    handle_scanned_data(data);
                    is_qr_scanner_modal_open.set(false);
                },
                on_close: move |_| is_qr_scanner_modal_open.set(false),
            }
        },
        NoTitleModal {
            is_open: is_qr_upload_modal_open,
            QrUploader {
                on_scan: move |data| {
                    handle_scanned_data(data);
                    is_qr_upload_modal_open.set(false);
                },
                on_close: move |_| is_qr_upload_modal_open.set(false),
            }
        },
        Modal {
            is_open: show_error_modal,
            title: "Error".to_string(),
            p { "{error_modal_message}" },
            footer { Button { on_click: move |_| show_error_modal.set(false), "Close" } }
        },
        Modal {
            is_open: show_duplicate_warning_modal,
            title: "Duplicate Address".to_string(),
            p { "This address is already in the recipient list. Do you want to add it again?" },
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
            },
            footer {
                Button {
                    button_type: ButtonType::Secondary,
                    outline: true,
                    on_click: move |_| show_duplicate_warning_modal.set(false),
                    "Cancel"
                },
                Button {
                    on_click: move |_| {
                        if let (Some(addr), Some(index)) = (pending_address.take(), action_target_index()) {
                                let mut target_recipient = recipients.read()[index];
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
        },
        div {
            match wizard_step() {
                WizardStep::AddRecipients => rsx! {
                    Card {
                        h3 { "Add Recipients" },
                        for (i, recipient) in recipients.iter().enumerate() {
                            EditableRecipientRow {
                                key: "{i}",
                                index: i,
                                recipient: *recipient,
                                is_active: active_row_index() == Some(i),
                                on_delete: move |index_to_delete: usize| {
                                    if recipients.len() > 1 {
                                        if active_row_index() == Some(index_to_delete) {
                                            active_row_index.set(None);
                                        }
                                        recipients.write().remove(index_to_delete);
                                    }
                                },
                                on_open_address_actions: move |index: usize| {
                                    if active_row_index() == Some(index) {
                                        action_target_index.set(Some(index));
                                        is_address_actions_modal_open.set(true);
                                    }
                                },
                                on_set_active: move |index: usize| {
                                    active_row_index.set(Some(index));
                                },
                                on_done_editing: move |_| {
                                    active_row_index.set(None);
                                },
                                can_delete: recipients.len() > 1,
                                is_any_other_row_active: is_any_row_active() && active_row_index() != Some(i),
                            }
                        },
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-top: 1rem;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| {
                                    let new_index = recipients.len();
                                    recipients.write().push(Signal::new(EditableRecipient::default()));
                                    active_row_index.set(Some(new_index));
                                },
                                disabled: is_any_row_active(),
                                "Add Another Recipient"
                            },
                            if recipients.len() > 1 {
                                h5 { "Subtotal: {subtotal}" }
                            }
                        }
                    },
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
                        },
                        if let Some(err) = fee_error() {
                            small { style: "color: var(--pico-color-red-500);", "{err}" }
                        },
                        h4 { style: "margin-top: 1rem; text-align: right;", "Total Spend: {total_spend}" },
                        Button {
                            on_click: move |_| {
                                if is_form_fully_valid() {
                                    wizard_step.set(WizardStep::Review);
                                }
                            },
                            disabled: !is_form_fully_valid() || is_any_row_active(),
                            "Next: Review"
                        }
                    }
                },
                WizardStep::Review => rsx! {
                    Card {
                        h3 { "Review Transaction" },
                        p { "Please review the details below. This action cannot be undone." },
                        h5 { style: "margin-top: 1rem;", "Recipients:" },
                        table {
                            role: "grid",
                            tbody {
                                {recipients.read().iter().map(|recipient_signal| {
                                    let recipient = recipient_signal.read();
                                    let addr = Rc::new(ReceivingAddress::from_bech32m(&recipient.address_str, network).unwrap());
                                    let amount = NativeCurrencyAmount::coins_from_str(&recipient.amount_str).unwrap();
                                    rsx! {
                                        tr {
                                            td { Address { address: addr.clone() } }
                                            td { style: "text-align: right;", "{amount}" }
                                        }
                                    }
                                })}
                            }
                        },
                        hr {},
                        p {
                            strong { "Fee: " },
                            if fee_str().is_empty() { "0.0" } else { "{fee_str()}" }
                        },
                        p { strong { "Total Spend: " }, "{total_spend}" },
                        footer {
                            style: "display: flex; justify-content: space-between; margin-top: 1rem;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| wizard_step.set(WizardStep::AddRecipients),
                                "Back"
                            },
                            Button {
                                on_click: move |_| {
                                    spawn(async move {
                                        let outputs: Vec<OutputFormat> = recipients.read().iter().map(|recipient_signal| {
                                            let recipient = recipient_signal.read();
                                            let addr = ReceivingAddress::from_bech32m(&recipient.address_str, network).unwrap();
                                            let amount = NativeCurrencyAmount::coins_from_str(&recipient.amount_str).unwrap();
                                            OutputFormat::AddressAndAmount(addr, amount)
                                        }).collect();
                                        let fee: NativeCurrencyAmount = NativeCurrencyAmount::coins_from_str(&fee_str()).unwrap_or_default();
                                        let change_policy = ChangePolicy::default();
                                        let result = api::send(outputs, change_policy, fee).await;
                                        api_response.set(Some(result));
                                        wizard_step.set(WizardStep::Status);
                                    });
                                },
                                "Confirm & Send"
                            }
                        }
                    }
                },
                WizardStep::Status => rsx! {
                    if let Some(response_result) = api_response() {
                        Card {
                            h3 { "Transaction Status" },
                            match response_result {
                                Ok((kernel_id, _details)) => rsx! {
                                    p {
                                        style: "color: var(--pico-color-green-500);",
                                        "Transaction sent successfully!"
                                    },
                                    div {
                                        style: "display: flex; justify-content: space-between; align-items: center; margin-top: 1.5rem; margin-bottom: 1.5rem; padding: 0.75rem; border: 1px solid var(--pico-secondary-border); border-radius: var(--pico-border-radius);",
                                        strong { "Transaction ID" },
                                        div {
                                            style: "display: flex; align-items: center; gap: 0.5rem;",
                                            code { "{kernel_id}" },
                                            CopyButton { text_to_copy: kernel_id.to_string() }
                                        }
                                    },
                                    div {
                                        style: "display: flex; gap: 1rem; margin-top: 1.5rem; flex-wrap: wrap;",
                                        Button {
                                            button_type: ButtonType::Primary,
                                            outline: true,
                                            on_click: move |_| {
                                                active_screen.set(Screen::MempoolTx(kernel_id));
                                            },
                                            "View in Mempool"
                                        },
                                        Button {
                                            on_click: move |_| reset_screen(),
                                            "Send Another Transaction"
                                        }
                                    }
                                },
                                Err(err) => rsx! {
                                    h4 { style: "color: var(--pico-color-red-500);", "Error Sending Transaction" },
                                    p { "{err}" },
                                    div {
                                        style: "display: flex; gap: 1rem; margin-top: 1.5rem; flex-wrap: wrap;",
                                        Button {
                                            button_type: ButtonType::Secondary,
                                            outline: true,
                                            on_click: move |_| wizard_step.set(WizardStep::Review),
                                            "Back"
                                        },
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
        }
    }
}
