//=============================================================================
// File: src/screens/send.rs
//=============================================================================
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use api::fiat_amount::FiatAmount;
use api::fiat_currency::FiatCurrency;
use api::prefs::display_preference::DisplayPreference;
use dioxus::prelude::*;
use neptune_types::address::ReceivingAddress;
use neptune_types::change_policy::ChangePolicy;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use neptune_types::output_format::OutputFormat;
use neptune_types::transaction_details::TransactionDetails;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use num_traits::Zero;

use crate::components::address::Address;
use crate::components::amount::Amount;
use crate::components::amount::AmountType;
use crate::components::currency_amount_input::CurrencyAmountInput;
use crate::components::digest_display::DigestDisplay;
use crate::components::pico::Button;
use crate::components::pico::ButtonType;
use crate::components::pico::Card;
use crate::components::pico::CloseButton;
use crate::components::pico::CopyButton;
use crate::components::pico::Modal;
use crate::components::pico::NoTitleModal;
use crate::components::qr_scanner::QrScanner;
use crate::components::qr_uploader::QrUploader;
use crate::currency::fiat_to_npt;
use crate::currency::npt_to_fiat;
use crate::AppState;
use crate::AppStateMut;
use crate::Screen;

static NEXT_RECIPIENT_ID: AtomicU64 = AtomicU64::new(0);

const NPT_MAX_INTEGER_DIGITS: u8 = 8;
const NPT_MAX_DECIMAL_DIGITS: u8 = 8;
const FIAT_MAX_INTEGER_DIGITS: u8 = 12;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputKind {
    Npt,
    Fiat(FiatCurrency),
}

#[derive(Clone, PartialEq, Debug)]
pub struct SourcedAmount {
    pub source_value: String,
    pub source_kind: InputKind,
    pub display_value: String,
}

impl SourcedAmount {
    pub fn new(initial_kind: InputKind) -> Self {
        Self {
            source_value: "0".to_string(),
            source_kind: initial_kind,
            display_value: "0.0".to_string(),
        }
    }

    pub fn as_npt(&self, rate: &FiatAmount) -> Result<NativeCurrencyAmount, String> {
        match self.source_kind {
            InputKind::Npt => {
                NativeCurrencyAmount::coins_from_str(&self.source_value).map_err(|e| e.to_string())
            }
            InputKind::Fiat(fc) => {
                let fiat_amount =
                    FiatAmount::new_from_str(&self.source_value, fc).map_err(|e| e.to_string())?;
                fiat_to_npt(&fiat_amount, rate).map_err(|e| e.to_string())
            }
        }
    }

    pub fn as_fiat(&self, rate: &FiatAmount) -> Result<FiatAmount, String> {
        match self.source_kind {
            InputKind::Npt => {
                let npt = NativeCurrencyAmount::coins_from_str(&self.source_value)
                    .map_err(|e| e.to_string())?;
                Ok(npt_to_fiat(&npt, rate))
            }
            InputKind::Fiat(fc) => {
                assert_eq!(fc, rate.currency());
                Ok(FiatAmount::new_from_str(&self.source_value, fc).map_err(|e| e.to_string())?)
            }
        }
    }

    pub fn as_npt_or_zero(&self, rate: &FiatAmount) -> NativeCurrencyAmount {
        self.as_npt(rate).unwrap_or_default()
    }

    pub fn as_fiat_or_zero(&self, rate: &FiatAmount) -> FiatAmount {
        self.as_fiat(rate)
            .unwrap_or_else(|_| FiatAmount::new_from_minor(0, rate.currency()))
    }

    #[allow(dead_code)]
    pub fn as_needed(&self, as_fiat: bool, rate: &FiatAmount) -> Result<String, String> {
        if as_fiat {
            self.as_fiat(rate).map(|a| a.to_string())
        } else {
            self.as_npt(rate).map(|a| a.to_string())
        }
    }

    pub fn as_needed_or_zero(&self, as_fiat: bool, rate: &FiatAmount) -> String {
        if as_fiat {
            self.as_fiat_or_zero(rate).to_string()
        } else {
            self.as_npt_or_zero(rate).to_string()
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct EditableRecipient {
    id: u64,
    address_str: String,
    amount: SourcedAmount,
    address_error: Option<String>,
    amount_error: Option<String>,
}

impl EditableRecipient {
    fn is_valid(&self, network: Network, rate: &FiatAmount) -> bool {
        ReceivingAddress::from_bech32m(&self.address_str, network).is_ok()
            && self.amount.as_npt_or_zero(rate) > NativeCurrencyAmount::zero()
    }
}

impl Default for EditableRecipient {
    fn default() -> Self {
        Self {
            id: NEXT_RECIPIENT_ID.fetch_add(1, Ordering::Relaxed),
            address_str: String::new(),
            amount: SourcedAmount::new(InputKind::Npt),
            address_error: None,
            amount_error: None,
        }
    }
}

#[component]
#[allow(clippy::too_many_arguments)]
fn EditableRecipientRow(
    index: usize,
    recipient: Signal<EditableRecipient>,
    on_delete: EventHandler<usize>,
    on_open_address_actions: EventHandler<usize>,
    popup_setter: Signal<Option<Element>>,
    can_delete: bool,
    is_active: bool,
    on_set_active: EventHandler<usize>,
    on_done_editing: EventHandler<()>,
    is_any_other_row_active: bool,
    on_amount_input: EventHandler<(usize, String)>,
    on_currency_toggle: EventHandler<usize>,
) -> Element {
    let app_state = use_context::<AppState>();
    let app_state_mut = use_context::<AppStateMut>();
    let network = app_state.network;

    let (fiat_currency, rate, display_as_fiat, fiat_mode_active) =
        match *app_state_mut.display_preference.read() {
            DisplayPreference::FiatEnabled {
                fiat,
                display_as_fiat,
                ..
            } => {
                let price = app_state_mut
                    .prices
                    .read()
                    .as_ref()
                    .and_then(|p| p.get(fiat))
                    .unwrap_or_else(|| FiatAmount::new_from_minor(0, fiat));
                (fiat, Rc::new(price), display_as_fiat, true)
            }
            DisplayPreference::NptOnly => (
                FiatCurrency::USD,
                Rc::new(FiatAmount::new_from_minor(0, FiatCurrency::USD)),
                false,
                false,
            ),
        };

    let show_fiat_toggle = fiat_mode_active && rate.as_minor_units() != 0;
    let parsed_address = use_memo(move || {
        ReceivingAddress::from_bech32m(&recipient.read().address_str, network).ok()
    });
    let display_address = use_memo(move || {
        parsed_address().map_or(recipient.read().address_str.clone(), |addr| {
            addr.to_display_bech32m_abbreviated(network)
                .unwrap_or_else(|_| recipient.read().address_str.clone())
        })
    });

    let (amount_label, max_integers, max_decimals) = if !display_as_fiat {
        (
            "Amount (NPT)".to_string(),
            NPT_MAX_INTEGER_DIGITS,
            NPT_MAX_DECIMAL_DIGITS,
        )
    } else {
        (
            format!("Amount ({})", fiat_currency.code()),
            FIAT_MAX_INTEGER_DIGITS,
            fiat_currency.decimals(),
        )
    };

    rsx! {
        div {
            class: if is_active { "recipient-row active" } else { "recipient-row" },
            style: "border: 1px solid var(--pico-form-element-border-color); border-radius: var(--pico-border-radius); padding: 0.5rem; margin-bottom: 0.5rem;",
            if is_active {
                div {
                    key: "active-state-{index}",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;",
                        label {
                            style: "margin-bottom: 0; align-self: flex-end;",
                            "Recipient Address"
                        }
                        div {
                            style: "display: flex; gap: 0.5rem;",
                            Button {
                                button_type: ButtonType::Primary,
                                on_click: move |e: MouseEvent| {
                                    e.stop_propagation();
                                    on_done_editing.call(())
                                },
                                disabled: !recipient.read().is_valid(network, &rate),
                                style: "padding-top: 0.25rem; padding-bottom: 0.25rem;".to_string(),
                                "Done"
                            }
                            if can_delete {
                                CloseButton {
                                    on_click: move |_| on_delete.call(index),
                                }
                            }
                        }
                    }
                    div {
                        key: "active-form-{index}",
                        div {


                            input {
                                class: "pico-input",
                                r#type: "text",
                                placeholder: "Click to paste or scan an address...",
                                value: "{display_address}",
                                readonly: true,
                                onclick: move |_| on_open_address_actions.call(index),
                                style: "cursor: pointer;",
                            }
                        }
                        if let Some(err) = &recipient.read().address_error {
                            small {
                                style: "color: var(--pico-color-red-500);",
                                "{err}"
                            }
                        }

                        div {
                            style: "margin-top: 0;",
                            label {


                                "{amount_label}"
                            }
                            div {
                                style: "display: flex; align-items: stretch; gap: 0.5rem; height: 3rem",
                                CurrencyAmountInput {
                                    value: recipient.read().amount.display_value.clone(),
                                    on_input: move |v| on_amount_input.call((index, v)),
                                    popup_state: popup_setter,
                                    max_integers,
                                    max_decimals,
                                    placeholder: "0.0".to_string(),
                                }
                                if show_fiat_toggle {
                                    Button {
                                        button_type: ButtonType::Secondary,
                                        title: format!(
                                            "Toggle between Neptune Cash (NPT) and {} ({})",
                                            fiat_currency.name(),
                                            fiat_currency.code().to_string(),
                                        ),
                                        outline: true,
                                        style: "width: 5rem; margin-bottom: 0; flex-shrink: 0;",
                                        on_click: move |_| on_currency_toggle.call(index),
                                        {
                                            if display_as_fiat {
                                                fiat_currency.code().to_string()
                                            } else {
                                                "NPT".to_string()
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(err) = &recipient.read().amount_error {
                                small {
                                    style: "color: var(--pico-color-red-500); display: block; margin-top: 0.25rem;",
                                    "{err}"
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    key: "inactive-state-{index}",
                    style: "display: flex; justify-content: space-between; align-items: center; width: 100%;",
                    div {
                        style: "flex-grow: 1; min-width: 0;",
                        if let Some(addr) = parsed_address() {
                            Address {
                                address: Rc::new(addr),
                            }
                        } else {
                            code {


                                "{display_address}"
                            }
                        }
                    }
                    div {
                        style: "text-align: right; margin: 0 1rem; white-space: nowrap;",
                        Amount {
                            amount: recipient.read().amount.as_npt_or_zero(&rate),
                            fiat_equivalent: {
                                let amount = &recipient.read().amount;
                                match amount.source_kind {
                                    InputKind::Npt => None,
                                    InputKind::Fiat(_) => Some(amount.as_fiat_or_zero(&rate)),
                                }
                            },
                        }
                    }
                    div {
                        style: "display: flex; justify-content: flex-end; gap: 0.5rem; align-items: center; width: 8rem;",
                        Button {
                            button_type: ButtonType::Secondary,
                            outline: true,
                            on_click: move |_| on_set_active.call(index),
                            disabled: is_any_other_row_active,
                            "Edit"
                        }
                        if can_delete {
                            CloseButton {
                                on_click: move |_| on_delete.call(index),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SendScreen() -> Element {
    let app_state = use_context::<AppState>();
    let mut app_state_mut = use_context::<AppStateMut>();
    let network = app_state.network;

    let (fiat_currency, rate_rc, display_as_fiat, fiat_mode_active) =
        match *app_state_mut.display_preference.read() {
            DisplayPreference::FiatEnabled {
                fiat,
                display_as_fiat,
                ..
            } => {
                let price = app_state_mut
                    .prices
                    .read()
                    .as_ref()
                    .and_then(|p| p.get(fiat))
                    .unwrap_or_else(|| FiatAmount::new_from_minor(0, fiat));
                (fiat, Rc::new(price), display_as_fiat, true)
            }
            DisplayPreference::NptOnly => (
                FiatCurrency::USD,
                Rc::new(FiatAmount::new_from_minor(0, FiatCurrency::USD)),
                false,
                false,
            ),
        };

    #[derive(PartialEq, Clone, Copy)]
    enum WizardStep {
        AddRecipients,
        EnterFee,
        Review,
        Status,
    }
    let mut wizard_step = use_signal(|| WizardStep::AddRecipients);
    let mut api_response = use_signal::<
        Option<Result<(TransactionKernelId, TransactionDetails), api::ApiError>>,
    >(|| None);
    let mut recipients = use_signal(move || {
        let initial_kind = if display_as_fiat {
            InputKind::Fiat(fiat_currency)
        } else {
            InputKind::Npt
        };
        let initial_amount = SourcedAmount::new(initial_kind);
        vec![Signal::new(EditableRecipient {
            amount: initial_amount,
            ..Default::default()
        })]
    });
    let mut fee_input = use_signal(move || {
        SourcedAmount::new(if display_as_fiat {
            InputKind::Fiat(fiat_currency)
        } else {
            InputKind::Npt
        })
    });
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
    let popup_slot = use_signal::<Option<Element>>(|| None);

    let is_any_row_active = use_memo(move || active_row_index().is_some());
    let are_recipients_valid = {
        let rate = rate_rc.clone();
        use_memo(move || {
            !recipients.read().is_empty()
                && recipients
                    .read()
                    .iter()
                    .all(|r| r.read().is_valid(network, &rate))
        })
    };
    let is_fee_valid = {
        let rate = rate_rc.clone();
        use_memo(move || fee_input.read().as_npt(&rate).is_ok())
    };

    let subtotals = {
        let rate = rate_rc.clone();
        use_memo(move || {
            recipients.read().iter().fold(
                (
                    NativeCurrencyAmount::zero(),
                    FiatAmount::new_from_minor(0, fiat_currency),
                ),
                |(npt_acc, fiat_acc), r| {
                    let amt = &r.read().amount;
                    let npt = amt.as_npt_or_zero(&rate);
                    let fiat = amt.as_fiat_or_zero(&rate);
                    (npt_acc + npt, fiat_acc + fiat)
                },
            )
        })
    };

    let mut reset_screen = move || {
        let initial_kind = if display_as_fiat {
            InputKind::Fiat(fiat_currency)
        } else {
            InputKind::Npt
        };
        recipients.set(vec![Signal::new(EditableRecipient {
            amount: SourcedAmount::new(initial_kind),
            ..Default::default()
        })]);
        active_row_index.set(Some(0));
        fee_input.set(SourcedAmount::new(initial_kind));
        fee_error.set(None);
        api_response.set(None);
        suppress_duplicate_warning.set(false);
        wizard_step.set(WizardStep::AddRecipients);
    };

    let mut active_screen = use_context::<Signal<Screen>>();

    let mut handle_scanned_data = move |scanned_text: String| {
        if let Some(index) = action_target_index() {
            if ReceivingAddress::from_bech32m(&scanned_text, network).is_ok() {
                let is_duplicate = recipients
                    .read()
                    .iter()
                    .enumerate()
                    .any(|(i, r)| i != index && r.read().address_str == scanned_text);
                if is_duplicate && !suppress_duplicate_warning() {
                    pending_address.set(Some(scanned_text));
                    show_duplicate_warning_modal.set(true);
                } else if let Ok(mut recs) = recipients.try_write() {
                    if let Some(target_recipient) = recs.get_mut(index) {
                        target_recipient.with_mut(|r| {
                            r.address_str = scanned_text;
                            r.address_error = None;
                        });
                    }
                }
            } else {
                error_modal_message.set("Invalid Address from QR.".to_string());
                show_error_modal.set(true);
            }
        }
    };

    fn kinds_match(kind1: InputKind, kind2: InputKind) -> bool {
        match (kind1, kind2) {
            (InputKind::Npt, InputKind::Npt) => true,
            (InputKind::Fiat(fc1), InputKind::Fiat(fc2)) => fc1 == fc2,
            _ => false,
        }
    }

    let update_recipient_value = {
        let mut recipients = recipients;
        let rate = rate_rc.clone();

        move |(index, new_value): (usize, String)| {
            if let Ok(mut recs) = recipients.try_write() {
                if let Some(recipient) = recs.get_mut(index) {
                    recipient.with_mut(|r| {
                        let new_source_kind = if !display_as_fiat {
                            InputKind::Npt
                        } else {
                            InputKind::Fiat(fiat_currency)
                        };

                        if !kinds_match(r.amount.source_kind, new_source_kind) {
                            r.amount.source_kind = new_source_kind;
                        }

                        // The input component now directly controls the display value.
                        r.amount.source_value = new_value.clone();
                        r.amount.display_value = new_value;

                        match r.amount.as_npt(&rate) {
                            Ok(amt) if amt.is_zero() && !r.amount.source_value.is_empty() => {
                                r.amount_error = Some("Amount must be > 0.".to_string())
                            }
                            Ok(_) => r.amount_error = None,
                            Err(e) if !r.amount.source_value.is_empty() => r.amount_error = Some(e),
                            _ => r.amount_error = None,
                        }
                    });
                }
            }
        }
    };

    let on_recipient_currency_toggle = {
        let rate = rate_rc.clone();
        move |index: usize| {
            app_state_mut.display_preference.with_mut(|pref| {
                if let DisplayPreference::FiatEnabled {
                    display_as_fiat, ..
                } = pref
                {
                    *display_as_fiat = !*display_as_fiat;
                }
            });
            let new_display_as_fiat = !display_as_fiat;
            if let Ok(mut recs) = recipients.try_write() {
                if let Some(recipient) = recs.get_mut(index) {
                    recipient.with_mut(|r| {
                        r.amount.display_value =
                            r.amount.as_needed_or_zero(new_display_as_fiat, &rate);
                    });
                }
            }
        }
    };

    rsx! {
        {popup_slot()}

        NoTitleModal {
            is_open: is_address_actions_modal_open,
            div {
                style: "display: flex; flex-direction: column; gap: 1rem;",
                h3 {


                    "Set Address"
                }
                p {


                    {
                        if let Some(index) = action_target_index() {
                            format!("Choose an action for recipient number {}.", index + 1)
                        } else {
                            "Choose an action.".to_string()
                        }
                    }
                }
                Button {
                    on_click: move |_| {
                        if action_target_index().is_some() {
                            spawn(async move {
                                if let Some(ct) = crate::compat::clipboard_get().await {
                                    handle_scanned_data(ct);
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
                        is_qr_scanner_modal_open.set(true);
                    },
                    "Scan QR Code"
                }
                Button {
                    on_click: move |_| {
                        is_address_actions_modal_open.set(false);
                        is_qr_upload_modal_open.set(true);
                    },
                    "Upload QR Image"
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
            is_open: is_qr_scanner_modal_open,
            QrScanner {
                on_scan: move |d| {
                    handle_scanned_data(d);
                    is_qr_scanner_modal_open.set(false);
                },
                on_close: move |_| is_qr_scanner_modal_open.set(false),
            }
        }
        NoTitleModal {
            is_open: is_qr_upload_modal_open,
            QrUploader {
                on_scan: move |d| {
                    handle_scanned_data(d);
                    is_qr_upload_modal_open.set(false);
                },
                on_close: move |_| is_qr_upload_modal_open.set(false),
            }
        }

        Modal {
            is_open: show_error_modal,
            title: "Error".to_string(),
            p {


                "{error_modal_message}"
            }
            footer {


                Button {
                    on_click: move |_| show_error_modal.set(false),
                    "Close"
                }
            }
        }
        Modal {
            is_open: show_duplicate_warning_modal,
            title: "Duplicate Address".to_string(),
            p {


                "This address is already in the recipient list. Do you want to add it again?"
            }
            div {
                style: "margin-top: 1rem; margin-bottom: 1rem;",
                label {


                    input {
                        r#type: "checkbox",
                        checked: "{suppress_duplicate_warning}",
                        oninput: move |e| suppress_duplicate_warning.set(e.value() == "true"),
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
                        if let (Some(addr), Some(index)) = (
                            pending_address.take(),
                            action_target_index(),
                        ) {
                            if let Ok(mut recs) = recipients.try_write() {
                                if let Some(target) = recs.get_mut(index) {
                                    target
                                        .with_mut(|r| {
                                            r.address_str = addr;
                                            r.address_error = None;
                                        });
                                }
                            }
                        }
                        show_duplicate_warning_modal.set(false);
                    },
                    "Proceed Anyway"
                }
            }
        }

        div {


            match wizard_step() {
                WizardStep::AddRecipients => rsx! {
                    div {
                        style: "display: flex; flex-direction: column; height: 75vh;",
                        h3 {
                            style: "margin: 0 0 0.5rem 0; padding: 0 0.5rem;",
                            "Add Recipients"
                        }
                        div {
                            style: "flex-grow: 0; overflow-y: auto; padding: 0 0.5rem;",
                            Card {

                                for (i , recipient) in recipients.iter().enumerate() {
                                    EditableRecipientRow {
                                        key: "{recipient.read().id}",
                                        index: i,
                                        recipient: *recipient,
                                        on_delete: move |idx| {
                                            if recipients.read().len() > 1 {
                                                if let Ok(mut recs) = recipients.try_write() {
                                                    recs.remove(idx);
                                                }
                                                if active_row_index() == Some(idx) {
                                                    active_row_index.set(None);
                                                }
                                            }
                                        },
                                        on_open_address_actions: move |idx| {
                                            if active_row_index() == Some(idx) {
                                                action_target_index.set(Some(idx));
                                                is_address_actions_modal_open.set(true);
                                            }
                                        },
                                        popup_setter: popup_slot,
                                        can_delete: recipients.read().len() > 1,
                                        is_active: active_row_index() == Some(i),
                                        on_set_active: move |idx| active_row_index.set(Some(idx)),
                                        on_done_editing: move |_| active_row_index.set(None),
                                        is_any_other_row_active: is_any_row_active() && active_row_index() != Some(i),
                                        on_amount_input: update_recipient_value.clone(),
                                        on_currency_toggle: on_recipient_currency_toggle.clone(),
                                    }
                                }
                            }
                        }
                        div {
                            style: "flex-shrink: 1; margin-top: 1rem; padding: 0.25rem 1rem; background: var(--pico-background-color); border-top: 1px solid var(--pico-muted-border-color); display: flex; justify-content: space-between; align-items: center;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| {
                                    if let Ok(mut recs) = recipients.try_write() {
                                        let initial_kind = if display_as_fiat {
                                            InputKind::Fiat(fiat_currency)
                                        } else {
                                            InputKind::Npt
                                        };
                                        recs.insert(
                                            0,
                                            Signal::new(EditableRecipient {
                                                amount: SourcedAmount::new(initial_kind),
                                                ..Default::default()
                                            }),
                                        );
                                    }
                                    active_row_index.set(Some(0));
                                },
                                disabled: is_any_row_active(),
                                "Add Another Recipient"
                            }
                            div {
                                style: "text-align: right; line-height: 1.2;",
                                div {

                                    Amount {
                                        amount: subtotals().0,
                                        fiat_equivalent: Some(subtotals().1),
                                        fixed: Some(AmountType::Npt),
                                    }
                                }
                                if fiat_mode_active {
                                    small {
                                        style: "color: var(--pico-muted-color);",
                                        Amount {
                                            amount: subtotals().0,
                                            fiat_equivalent: Some(subtotals().1),
                                            fixed: Some(AmountType::Fiat),
                                        }
                                    }
                                }
                            }
                            Button {
                                on_click: move |_| {
                                    if are_recipients_valid() {
                                        wizard_step.set(WizardStep::EnterFee);
                                    }
                                },
                                disabled: !are_recipients_valid() || is_any_row_active(),
                                "Next: Set Fee"
                            }
                        }
                    }
                },
                WizardStep::EnterFee => rsx! {
                    {
                        let rate = rate_rc.clone();
                        let (fee_max_integers, fee_max_decimals) = match fee_input.read().source_kind {
                            InputKind::Npt => (NPT_MAX_INTEGER_DIGITS, NPT_MAX_DECIMAL_DIGITS),
                            InputKind::Fiat(fc) => (FIAT_MAX_INTEGER_DIGITS, fc.decimals()),
                        };
                        let (fee_npt, fee_fiat) = {
                            let amt = fee_input.read();
                            let npt = amt.as_npt_or_zero(&rate);
                            let fiat = amt.as_fiat_or_zero(&rate);
                            (npt, fiat)
                        };
                        let subtotal_npt = subtotals().0;
                        let subtotal_fiat = subtotals().1;
                        let total_spend_npt = subtotal_npt + fee_npt;
                        let total_spend_fiat = subtotal_fiat + fee_fiat;
                        rsx! {
                            Card {

                                h3 {

                                    "Set Fee"
                                }
                                p {

                                    span {

                                        "Subtotal: "
                                    }
                                    Amount {
                                        amount: subtotal_npt,
                                        fiat_equivalent: Some(subtotal_fiat),
                                        fixed: Some(AmountType::Npt),
                                    }
                                    if fiat_mode_active {
                                        span {
                                            style: "color: var(--pico-muted-color);",
                                            span {

                                                " ("
                                            }
                                            Amount {
                                                amount: subtotal_npt,
                                                fiat_equivalent: Some(subtotal_fiat),
                                                fixed: Some(AmountType::Fiat),
                                            }
                                            span {

                                                ")"
                                            }
                                        }
                                    }
                                }
                                hr {


                                }
                                label {

                                    if !display_as_fiat {
                                        "Fee (NPT)"
                                    } else {
                                        "Fee ({fiat_currency.code()})"
                                    }
                                }
                                div {
                                    style: "display: flex; align-items: stretch; gap: 0.5rem; height: 3rem;",
                                    CurrencyAmountInput {
                                        value: fee_input.read().display_value.clone(),
                                        on_input: {
                                            let rate = rate_rc.clone();
                                            move |sanitized_value: String| {
                                                if let Ok(mut fi) = fee_input.try_write() {
                                                    let new_source_kind = if !display_as_fiat {
                                                        InputKind::Npt
                                                    } else {
                                                        InputKind::Fiat(fiat_currency)
                                                    };
                                                    if !kinds_match(fi.source_kind, new_source_kind) {
                                                        fi.source_kind = new_source_kind;
                                                    }
                                                    fi.source_value = sanitized_value.clone();
                                                    fi.display_value = sanitized_value;
                                                    match fi.as_npt(&rate) {
                                                        Ok(_) => fee_error.set(None),
                                                        Err(e) if !fi.source_value.is_empty() => fee_error.set(Some(e)),
                                                        _ => fee_error.set(None),
                                                    }
                                                }
                                            }
                                        },
                                        popup_state: popup_slot,
                                        max_integers: fee_max_integers,
                                        max_decimals: fee_max_decimals,
                                        placeholder: "0.0".to_string(),
                                    }
                                    if fiat_mode_active {
                                        Button {
                                            button_type: ButtonType::Secondary,
                                            outline: true,
                                            style: "width: 5rem; margin-bottom: 0;",
                                            on_click: {
                                                let rate = rate_rc.clone();
                                                move |_| {
                                                    app_state_mut
                                                        .display_preference
                                                        .with_mut(|pref| {
                                                            if let DisplayPreference::FiatEnabled { display_as_fiat, .. } = pref {
                                                                *display_as_fiat = !*display_as_fiat;
                                                            }
                                                        });
                                                    let new_display_as_fiat = !display_as_fiat;
                                                    if let Ok(mut fi) = fee_input.try_write() {
                                                        fi.display_value = fi.as_needed_or_zero(new_display_as_fiat, &rate);
                                                    }
                                                }
                                            },
                                            {
                                                if display_as_fiat {
                                                    fiat_currency.code().to_string()
                                                } else {
                                                    "NPT".to_string()
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some(err) = fee_error() {
                                    small {
                                        style: "color: var(--pico-color-red-500); display: block; margin-top: 0.25rem;",
                                        "{err}"
                                    }
                                }
                                div {
                                    style: "margin-top: 1rem; text-align: right;",
                                    h4 {

                                        "Total Spend"
                                    }
                                    Amount {
                                        amount: total_spend_npt,
                                        fiat_equivalent: Some(total_spend_fiat),
                                        fixed: Some(AmountType::Npt),
                                    }
                                    if fiat_mode_active {
                                        div {
                                            style: "color: var(--pico-muted-color); font-size: 0.9em;",
                                            Amount {
                                                amount: total_spend_npt,
                                                fiat_equivalent: Some(total_spend_fiat),
                                                fixed: Some(AmountType::Fiat),
                                            }
                                        }
                                    }
                                }
                                footer {
                                    style: "flex-shrink: 1; display: flex; justify-content: space-between;",

                                    Button {
                                        button_type: ButtonType::Secondary,
                                        outline: true,
                                        on_click: move |_| wizard_step.set(WizardStep::AddRecipients),
                                        "Back"
                                    }
                                    Button {
                                        on_click: move |_| wizard_step.set(WizardStep::Review),
                                        disabled: !is_fee_valid(),
                                        "Next: Review"
                                    }
                                }
                            }
                        }
                    }
                },
                WizardStep::Review => rsx! {
                    {
                        let rate = rate_rc.clone();
                        let fee_npt = fee_input.read().as_npt_or_zero(&rate);
                        let total_spend_npt = subtotals().0 + fee_npt;
                        let fiat_fee_display = fee_input.read().as_fiat_or_zero(&rate);
                        let fiat_total_display = subtotals().1 + fiat_fee_display;
                        rsx! {
                            Card {

                                h3 {

                                    "Review Transaction"
                                }
                                p {

                                    "Please review the details below. This action cannot be undone."
                                }
                                h5 {
                                    style: "margin-top: 1rem;",
                                    "Recipients:"
                                }
                                table {
                                    role: "grid",
                                    tbody {

                                        for recipient_signal in recipients.read().iter() {
                                            {
                                                let recipient = recipient_signal.read();
                                                let final_npt_amount = recipient.amount.as_npt_or_zero(&rate);
                                                let fiat_equiv = Some(recipient.amount.as_fiat_or_zero(&rate));
                                                let addr = Rc::new(
                                                    ReceivingAddress::from_bech32m(&recipient.address_str, network).unwrap(),
                                                );
                                                rsx! {
                                                    tr {

                                                        td {

                                                            Address {
                                                                address: addr.clone(),
                                                            }
                                                        }
                                                        td {
                                                            style: "text-align: right;",
                                                            Amount {
                                                                amount: final_npt_amount,
                                                                fiat_equivalent: fiat_equiv,
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div {
                                    style: "text-align: right; margin-top: 1rem;",
                                    strong {

                                        "Fee: "
                                    }
                                    Amount {
                                        amount: fee_npt,
                                        fiat_equivalent: Some(fiat_fee_display),
                                        fixed: Some(AmountType::Npt),
                                    }
                                    if fiat_mode_active {
                                        span {
                                            style: "color: var(--pico-muted-color); font-weight: normal; margin-left: 0.5rem;",
                                            "("
                                            Amount {
                                                amount: fee_npt,
                                                fiat_equivalent: Some(fiat_fee_display),
                                                fixed: Some(AmountType::Fiat),
                                            }
                                            ")"
                                        }
                                    }
                                }
                                div {
                                    style: "text-align: right; margin-top: 0.5rem; font-weight: bold; border-top: 1px solid var(--pico-secondary-border); padding-top: 0.5rem; display: grid; grid-template-columns: 1fr auto;",
                                    span {
                                        style: "justify-self: end; padding-right: 1rem;",
                                        "Total Spend: "
                                    }
                                    div {

                                        Amount {
                                            amount: total_spend_npt,
                                            fiat_equivalent: Some(fiat_total_display),
                                            fixed: Some(AmountType::Npt),
                                        }
                                        if fiat_mode_active {
                                            small {

                                                span {
                                                    style: "color: var(--pico-muted-color); font-weight: normal; margin-left: 0.5rem;",
                                                    " ("
                                                    Amount {
                                                        amount: total_spend_npt,
                                                        fiat_equivalent: Some(fiat_total_display),
                                                        fixed: Some(AmountType::Fiat),
                                                    }
                                                    " est.)"
                                                }
                                            }
                                        }
                                    }
                                }
                                footer {
                                    style: "flex-shrink: 1; display: flex; justify-content: space-between;",

                                    Button {
                                        button_type: ButtonType::Secondary,
                                        outline: true,
                                        on_click: move |_| wizard_step.set(WizardStep::EnterFee),
                                        "Back"
                                    }
                                    Button {
                                        on_click: {
                                            let rate = rate_rc.clone();
                                            move |_| {
                                                let network = network;
                                                let recipients = recipients;
                                                let fee_input = fee_input;
                                                let mut api_response = api_response;
                                                let mut wizard_step = wizard_step;
                                                let rate = rate.clone();
                                                spawn(async move {
                                                    let outputs: Vec<OutputFormat> = recipients
                                                        .read()
                                                        .iter()
                                                        .map(|rs| {
                                                            let r = rs.read();
                                                            let addr = ReceivingAddress::from_bech32m(
                                                                    &r.address_str,
                                                                    network,
                                                                )
                                                                .unwrap();
                                                            let amount = r.amount.as_npt_or_zero(&rate);
                                                            OutputFormat::AddressAndAmount(addr, amount)
                                                        })
                                                        .collect();
                                                    let fee = fee_input.read().as_npt_or_zero(&rate);
                                                    let result = api::send(outputs, ChangePolicy::default(), fee).await;
                                                    api_response.set(Some(result));
                                                    wizard_step.set(WizardStep::Status);
                                                });
                                            }
                                        },
                                        "Confirm & Send"
                                    }
                                }
                            }
                        }
                    }
                },
                WizardStep::Status => rsx! {
                    if let Some(response_result) = api_response.read().as_ref() {
                        Card {
                            h3 { "Transaction Status" }

                            match response_result {
                                Ok((kernel_id, _details)) => {
                                    let kernel_id_clone = kernel_id.clone();

                                    rsx! {
                                        p {
                                            style: "color: var(--pico-color-green-500);",
                                            "Transaction sent successfully!"
                                        }
                                        div {
                                            style: "display: flex; justify-content: space-between; align-items: center; margin-top: 1.5rem; margin-bottom: 1.5rem; padding: 0.75rem; border: 1px solid var(--pico-secondary-border); border-radius: var(--pico-border-radius);",
                                            strong { "Transaction ID" }
                                            DigestDisplay {
                                                digest: (*kernel_id).into(),
                                                as_code: true,
                                            }
                                        }
                                        div {
                                            style: "display: flex; gap: 1rem; margin-top: 1.5rem; flex-wrap: wrap;",
                                            Button {
                                                button_type: ButtonType::Primary,
                                                outline: true,
                                                on_click: move |evt: Event<MouseData>| {
                                                    evt.prevent_default();
                                                    active_screen.set(Screen::MempoolTx(kernel_id_clone));
                                                },
                                                "View in Mempool"
                                            }
                                            Button {
                                                on_click: move |_| reset_screen(),
                                                "Send Another Transaction"
                                            }
                                        }
                                    }
                                },
                                Err(err) => rsx! {
                                    h4 {
                                        style: "color: var(--pico-color-red-500);",
                                        "Error Sending Transaction"
                                    }
                                    p { "{err}" }
                                    div {
                                        style: "display: flex; gap: 1rem; margin-top: 1.5rem; flex-wrap: wrap;",
                                        Button {
                                            button_type: ButtonType::Secondary,
                                            outline: true,
                                            on_click: move |_| wizard_step.set(WizardStep::Review),
                                            "Back"
                                        }
                                        Button {
                                            on_click: move |_| reset_screen(),
                                            "Send Another Transaction"
                                        }
                                    }
                                },
                            }
                        }
                    } else {
                        // The signal is still None (loading)
                        Card {
                            h3 { "Sending Transaction..." }
                            p { "Please wait." }
                            progress { }
                        }
                    }
                },
            }
        }
    }
}
