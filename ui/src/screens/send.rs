//=============================================================================
// File: src/screens/send.rs
//=============================================================================
use crate::components::address::Address;
use crate::components::pico::{
    Button, ButtonType, Card, CloseButton, CopyButton, Modal, NoTitleModal,
};
use crate::components::currency_amount_input::CurrencyAmountInput;
use crate::components::qr_scanner::QrScanner;
use crate::components::qr_uploader::QrUploader;
use crate::components::amount::Amount;
use crate::{AppState, AppStateMut};
use crate::Screen;
use crate::app_state_mut::DisplayCurrency;
use dioxus::prelude::*;
use api::fiat_amount::FiatAmount;
use api::fiat_currency::FiatCurrency;
use neptune_types::address::ReceivingAddress;
use neptune_types::change_policy::ChangePolicy;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use neptune_types::output_format::OutputFormat;
use neptune_types::transaction_details::TransactionDetails;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use num_traits::Zero;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

static NEXT_RECIPIENT_ID: AtomicU64 = AtomicU64::new(0);

// NPT max is 42,000,000 = 8 digits.
// up to 34 decimal digits are possible, but default display repr is 8 digits, and
// for the send screen we only support entering up to 8 digits to KISS.
// maybe reconsider later.
const NPT_MAX_INTEGER_DIGITS: u8 = 8;
const NPT_MAX_DECIMAL_DIGITS: u8 = 8;

// No fixed max for fiat.  12 seems pretty safe for known fiat currencies.
const FIAT_MAX_INTEGER_DIGITS: u8 = 12;


// --- START: Integer-based currency conversion helpers ---
// This function uses BigInt for its calculation to prevent overflow.
fn npt_to_fiat(amount: &NativeCurrencyAmount, rate: &FiatAmount) -> FiatAmount {
    if rate.as_minor_units() == 0 {
        return FiatAmount::new_from_minor(0, rate.currency());
    }
    let npt_scaling_factor = NativeCurrencyAmount::coins(1).to_nau();

    // Convert values to BigInt to perform overflow-safe multiplication.
    let nau_big = BigInt::from(amount.to_nau());
    let rate_minor_big = BigInt::from(rate.as_minor_units());
    let scaling_factor_big = BigInt::from(npt_scaling_factor);

    let product = nau_big * rate_minor_big;
    let fiat_smallest_units_big = product / scaling_factor_big;

    // Convert the result back to i64, defaulting to i64::MAX on the unlikely event of overflow.
    let fiat_smallest_units = fiat_smallest_units_big.to_i64().unwrap_or(i64::MAX);

    FiatAmount::new_from_minor(fiat_smallest_units, rate.currency())
}

// This function uses BigInt for robustness.
fn fiat_to_npt(fiat_amount: &FiatAmount, rate: &FiatAmount) -> Result<NativeCurrencyAmount, &'static str> {
    if rate.as_minor_units() == 0 {
        return Err("Exchange rate is zero.");
    }
    let npt_scaling_factor = NativeCurrencyAmount::coins(1).to_nau();

    // Use BigInt for all intermediate calculations.
    let fiat_minor_big = BigInt::from(fiat_amount.as_minor_units());
    let scaling_factor_big = BigInt::from(npt_scaling_factor);
    let rate_minor_big = BigInt::from(rate.as_minor_units());

    if rate_minor_big.is_zero() {
        return Err("Exchange rate is zero.");
    }

    let product = fiat_minor_big * scaling_factor_big;
    let nau_big = product / rate_minor_big;

    // Convert the BigInt result back to i128.
    if let Some(nau) = nau_big.to_i128() {
        Ok(NativeCurrencyAmount::from_nau(nau))
    } else {
        Err("Exceeds maximum NPT supply of 42,000,000")
    }
}
// --- END: Integer-based currency conversion helpers ---


// --- START: New "Source of Truth" types ---
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputKind {
    Npt,
    Fiat(FiatCurrency),
}

#[derive(Clone, PartialEq, Debug)]
pub struct SourcedAmount {
    pub source_value: String,
    pub source_kind: InputKind,
}

impl SourcedAmount {
    pub fn new(initial_kind: InputKind) -> Self {
        Self {
            source_value: String::new(),
            source_kind: initial_kind,
        }
    }
}

// This helper now takes the specific rate needed, not the whole map.
fn sourced_amount_to_npt(
    amount: &SourcedAmount,
    rate: &FiatAmount
) -> Result<NativeCurrencyAmount, String> {
    match amount.source_kind {
        InputKind::Npt => {
            if amount.source_value.len() > 30 { return Err("Input is too large.".to_string()); }
            NativeCurrencyAmount::coins_from_str(&amount.source_value).map_err(|e| e.to_string())
        },
        InputKind::Fiat(fc) => {
            if amount.source_value.len() > 30 { return Err("Input is too large.".to_string()); }
            if amount.source_value.is_empty() { return Ok(NativeCurrencyAmount::zero()); }
            // Ensure the provided rate is for the correct currency.
            if fc != rate.currency() {
                return Err(format!("Mismatched rate currency: expected {}, got {}", fc.code(), rate.currency().code()));
            }
            let fiat_amount = FiatAmount::new_from_str(&amount.source_value, fc).map_err(|e| e.to_string())?;
            fiat_to_npt(&fiat_amount, rate).map_err(|e| e.to_string())
        }
    }
}
// --- END: New "Source of Truth" types ---


#[derive(PartialEq, Clone, Copy, Debug)]
enum CurrencyMode { Npt, Fiat }

#[derive(Clone, PartialEq, Debug)]
struct EditableRecipient {
    id: u64,
    address_str: String,
    amount: SourcedAmount,
    address_error: Option<String>,
    amount_error: Option<String>,
}

impl EditableRecipient {
    // This now takes the specific rate needed.
    fn is_valid(&self, network: Network, rate: &FiatAmount) -> bool {
        ReceivingAddress::from_bech32m(&self.address_str, network).is_ok()
            && sourced_amount_to_npt(&self.amount, rate).map_or(false, |amt| amt > NativeCurrencyAmount::zero())
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

fn sanitize_and_format(new_value: String, max_integers: u8, max_decimals: u8) -> String {
    let mut sanitized = String::new();
    let mut has_decimal = false;
    let mut integer_digits = 0;
    let mut decimal_digits = 0;

    for ch in new_value.chars() {
        if ch.is_ascii_digit() {
            if has_decimal {
                if decimal_digits < max_decimals {
                    sanitized.push(ch);
                    decimal_digits += 1;
                }
            } else {
                if integer_digits < max_integers {
                    sanitized.push(ch);
                    integer_digits += 1;
                }
            }
        } else if ch == '.' && !has_decimal {
            sanitized.push(ch);
            has_decimal = true;
        }
    }
    sanitized
}

#[component]
fn EditableRecipientRow(
    index: usize,
    recipient: Signal<EditableRecipient>,
    preferred_currency_mode: Signal<CurrencyMode>,
    on_delete: EventHandler<usize>,
    on_open_address_actions: EventHandler<usize>,
    popup_setter: Signal<Option<Element>>,
    can_delete: bool,
    is_active: bool,
    on_set_active: EventHandler<usize>,
    on_done_editing: EventHandler<()>,
    is_any_other_row_active: bool,
    on_amount_input: EventHandler<(usize, String)>,
    on_keypad_press: EventHandler<(usize, String)>,
    on_currency_toggle: EventHandler<usize>,
) -> Element {
    let app_state = use_context::<AppState>();
    let app_state_mut = use_context::<AppStateMut>();
    let network = app_state.network;

    let (fiat_currency, rate) = if let DisplayCurrency::Fiat(fc) = *app_state_mut.display_currency.read() {
        let price = app_state_mut.prices.read().as_ref().and_then(|p| p.get(fc)).unwrap_or_else(|| FiatAmount::new_from_minor(0, fc));
        (fc, Rc::new(price))
    } else {
        let default_fiat = FiatCurrency::USD;
        let price = app_state_mut.prices.read().as_ref().and_then(|p| p.get(default_fiat)).unwrap_or_else(|| FiatAmount::new_from_minor(0, default_fiat));
        (default_fiat, Rc::new(price))
    };
    let show_fiat_toggle = rate.as_minor_units() != 0;

    let parsed_address = use_memo(move || ReceivingAddress::from_bech32m(&recipient.read().address_str, network).ok());
    let display_address = use_memo(move || parsed_address().map_or(recipient.read().address_str.clone(), |addr| addr.to_display_bech32m_abbreviated(network).unwrap_or_else(|_| recipient.read().address_str.clone())));

    let (amount_label, max_integers, max_decimals) = if *preferred_currency_mode.read() == CurrencyMode::Npt {
        ("Amount (NPT)".to_string(), NPT_MAX_INTEGER_DIGITS, NPT_MAX_DECIMAL_DIGITS)
    } else {
        (format!("Amount ({})", fiat_currency.code()), FIAT_MAX_INTEGER_DIGITS, fiat_currency.decimals())
    };

    let mut local_amount_str = use_signal(String::new);
    use_effect({
        let rate = rate.clone();
        move || {
            let r = recipient.read();
            let display_string = match r.amount.source_kind {
                InputKind::Npt => {
                    if *preferred_currency_mode.read() == CurrencyMode::Npt {
                        r.amount.source_value.clone()
                    } else {
                        let npt = NativeCurrencyAmount::coins_from_str(&r.amount.source_value).unwrap_or_default();
                        npt_to_fiat(&npt, &rate).to_string()
                    }
                },
                InputKind::Fiat(source_currency) => {
                    let display_mode_is_fiat = *preferred_currency_mode.read() == CurrencyMode::Fiat;
                    if display_mode_is_fiat && source_currency == fiat_currency {
                        r.amount.source_value.clone()
                    } else {
                        let npt = sourced_amount_to_npt(&r.amount, &rate).unwrap_or_default();
                        if *preferred_currency_mode.read() == CurrencyMode::Npt {
                            npt.display_n_decimals(NPT_MAX_DECIMAL_DIGITS as usize)
                        } else {
                            // This case handles converting from, say, EUR source to USD display
                            npt_to_fiat(&npt, &rate).to_string()
                        }
                    }
                }
            };
            local_amount_str.set(display_string);
        }
    });

    rsx! {
        div {
            class: if is_active { "recipient-row active" } else { "recipient-row" },
            style: "border: 1px solid var(--pico-form-element-border-color); border-radius: var(--pico-border-radius); padding: 0.5rem; margin-bottom: 0.5rem;",
            if is_active {
                div {
                    key: "active-state-{index}",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;",
                        label { style: "margin-bottom: 0;", "Recipient Address" },
                        div {
                            style: "display: flex; gap: 0.5rem; align-items: center;",
                            Button {
                                button_type: ButtonType::Primary,
                                on_click: move |e: MouseEvent| { e.stop_propagation(); on_done_editing.call(()) },
                                disabled: !recipient.read().is_valid(network, &rate),
                                "Done"
                            },
                            if can_delete { CloseButton { on_click: move |_| on_delete.call(index) } }
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
                                style: "cursor: pointer;"
                            }
                        }
                        if let Some(err) = &recipient.read().address_error { small { style: "color: var(--pico-color-red-500);", "{err}" } }

                        div {
                            style: "margin-top: 0.75rem;",
                            label { "{amount_label}" },
                            div {
                                style: "display: flex; align-items: center; gap: 0.5rem;",
                                CurrencyAmountInput {
                                    value: local_amount_str,
                                    on_input: move |sanitized_value: String| {
                                        on_amount_input.call((index, sanitized_value));
                                    },
                                    on_keypad_press: move |key: String| {
                                        on_keypad_press.call((index, key));
                                    },
                                    popup_state: popup_setter,
                                    max_integers: max_integers,
                                    max_decimals: max_decimals,
                                    placeholder: "0.0".to_string()
                                }
                                {
                                    show_fiat_toggle.then(|| rsx! {
                                        button {
                                            style: "width: 5rem; margin-bottom: 0; flex-shrink: 0;",
                                            onclick: move |_| on_currency_toggle.call(index),
                                            {
                                                if *preferred_currency_mode.read() == CurrencyMode::Npt {
                                                    fiat_currency.code().to_string()
                                                } else {
                                                    "NPT".to_string()
                                                }
                                            }
                                        }
                                    })
                                }
                            }
                            if let Some(err) = &recipient.read().amount_error { small { style: "color: var(--pico-color-red-500);", "{err}" } }
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
                            Address { address: Rc::new(addr) }
                        } else {
                            code { "{display_address}" }
                        }
                    },
                    div {
                        style: "text-align: right; margin: 0 1rem; white-space: nowrap;",
                        Amount { amount: sourced_amount_to_npt(&recipient.read().amount, &rate).unwrap_or_default() }
                    },
                    div {
                        style: "display: flex; justify-content: flex-end; gap: 0.5rem; align-items: center; width: 8rem;",
                        Button { button_type: ButtonType::Secondary, outline: true, on_click: move |_| on_set_active.call(index), disabled: is_any_other_row_active, "Edit" },
                        if can_delete { CloseButton { on_click: move |_| on_delete.call(index) } }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SendScreen() -> Element {
    let app_state = use_context::<AppState>();
    let app_state_mut = use_context::<AppStateMut>();
    let network = app_state.network;

    let (fiat_currency, rate_rc) = if let DisplayCurrency::Fiat(fc) = *app_state_mut.display_currency.read() {
        let price = app_state_mut.prices.read().as_ref().and_then(|p| p.get(fc)).unwrap_or_else(|| FiatAmount::new_from_minor(0, fc));
        (fc, Rc::new(price))
    } else {
        let default_fiat = FiatCurrency::USD;
        let price = app_state_mut.prices.read().as_ref().and_then(|p| p.get(default_fiat)).unwrap_or_else(|| FiatAmount::new_from_minor(0, default_fiat));
        (default_fiat, Rc::new(price))
    };
    let show_fiat_toggle = rate_rc.as_minor_units() != 0;

    #[derive(PartialEq, Clone, Copy)]
    enum WizardStep { AddRecipients, EnterFee, Review, Status }
    let mut wizard_step = use_signal(|| WizardStep::AddRecipients);
    let mut preferred_currency_mode = use_signal(|| CurrencyMode::Npt);
    let mut api_response = use_signal::<Option<Result<(TransactionKernelId, TransactionDetails), ServerFnError>>>(|| None);
    let mut recipients = use_signal(move || vec![Signal::new(EditableRecipient {
        amount: SourcedAmount::new(InputKind::Npt),
        ..Default::default()
    })]);
    let mut fee_input = use_signal(|| SourcedAmount::new(InputKind::Npt));
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
        use_memo(move || !recipients.read().is_empty() && recipients.read().iter().all(|r| r.read().is_valid(network, &rate)))
    };
    let is_fee_valid = {
        let rate = rate_rc.clone();
        use_memo(move || sourced_amount_to_npt(&fee_input.read(), &rate).is_ok())
    };

    let subtotals = {
        let rate = rate_rc.clone();
        use_memo(move || {
            recipients.read().iter().fold((NativeCurrencyAmount::zero(), FiatAmount::new_from_minor(0, fiat_currency)), |(npt_acc, fiat_acc), r| {
                let npt = sourced_amount_to_npt(&r.read().amount, &rate).unwrap_or_default();
                let fiat = npt_to_fiat(&npt, &rate);
                (npt_acc + npt, fiat_acc + fiat)
            })
        })
    };

    let mut reset_screen = move || {
        let initial_kind = if *preferred_currency_mode.read() == CurrencyMode::Npt { InputKind::Npt } else { InputKind::Fiat(fiat_currency) };
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
                let is_duplicate = recipients.read().iter().enumerate().any(|(i, r)| i != index && r.read().address_str == scanned_text);
                if is_duplicate && !suppress_duplicate_warning() {
                    pending_address.set(Some(scanned_text));
                    show_duplicate_warning_modal.set(true);
                } else {
                    if let Ok(mut recs) = recipients.try_write() {
                        if let Some(target_recipient) = recs.get_mut(index) {
                            target_recipient.with_mut(|r| {
                                r.address_str = scanned_text;
                                r.address_error = None;
                            });
                        }
                    }
                }
            } else {
                error_modal_message.set("Invalid Address from QR.".to_string());
                show_error_modal.set(true);
            }
        }
    };

    let mut local_fee_amount_str = use_signal(String::new);
    use_effect({
        let rate = rate_rc.clone();
        move || {
            let fi = fee_input.read();
            let display_string = match fi.source_kind {
                InputKind::Npt => {
                    if *preferred_currency_mode.read() == CurrencyMode::Npt {
                        fi.source_value.clone()
                    } else {
                        let npt = NativeCurrencyAmount::coins_from_str(&fi.source_value).unwrap_or_default();
                        npt_to_fiat(&npt, &rate).to_string()
                    }
                },
                InputKind::Fiat(source_currency) => {
                     let display_mode_is_fiat = *preferred_currency_mode.read() == CurrencyMode::Fiat;
                     if display_mode_is_fiat && source_currency == fiat_currency {
                        fi.source_value.clone()
                     } else {
                        let npt = sourced_amount_to_npt(&fi, &rate).unwrap_or_default();
                        if *preferred_currency_mode.read() == CurrencyMode::Npt {
                            npt.display_n_decimals(NPT_MAX_DECIMAL_DIGITS as usize)
                        } else {
                            npt_to_fiat(&npt, &rate).to_string()
                        }
                     }
                }
            };
            local_fee_amount_str.set(display_string);
        }
    });

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
                        let display_mode = *preferred_currency_mode.read();

                        let (new_source_kind, max_integers, max_decimals) = if display_mode == CurrencyMode::Npt {
                            (InputKind::Npt, NPT_MAX_INTEGER_DIGITS, NPT_MAX_DECIMAL_DIGITS)
                        } else {
                            (InputKind::Fiat(fiat_currency), FIAT_MAX_INTEGER_DIGITS, fiat_currency.decimals())
                        };

                        if !kinds_match(r.amount.source_kind, new_source_kind) {
                            r.amount.source_kind = new_source_kind;
                        }

                        let sanitized = sanitize_and_format(new_value, max_integers, max_decimals);
                        r.amount.source_value = sanitized;

                        match sourced_amount_to_npt(&r.amount, &rate) {
                            Ok(amt) if amt.is_zero() && !r.amount.source_value.is_empty() => r.amount_error = Some("Amount must be > 0.".to_string()),
                            Ok(_) => r.amount_error = None,
                            Err(e) if !r.amount.source_value.is_empty() => r.amount_error = Some(e),
                            _ => r.amount_error = None,
                        }
                    });
                }
            }
        }
    };


    let on_recipient_keypad_press = {
        let mut updater = update_recipient_value.clone();
        move |(index, key): (usize, String)| {
            if let Some(recipient) = recipients.read().get(index).map(|s| s.read()) {
                let display_mode = *preferred_currency_mode.read();
                let new_source_kind = if display_mode == CurrencyMode::Npt {
                    InputKind::Npt
                } else {
                    InputKind::Fiat(fiat_currency)
                };

                let value_to_modify = if !kinds_match(recipient.amount.source_kind, new_source_kind) {
                    String::new()
                } else {
                    recipient.amount.source_value.clone()
                };

                let new_val = if key == "⌫" {
                    let mut chars = value_to_modify.chars();
                    chars.next_back();
                    chars.as_str().to_string()
                } else {
                    value_to_modify + &key
                };
                updater((index, new_val));
            }
        }
    };

    let on_recipient_currency_toggle = {
        let rate = rate_rc.clone();
        move |index: usize| {
            // First, toggle the global display mode.
            let new_mode = if *preferred_currency_mode.read() == CurrencyMode::Npt {
                CurrencyMode::Fiat
            } else {
                CurrencyMode::Npt
            };
            preferred_currency_mode.set(new_mode);

            // Then, convert the specific recipient's value and set it as their new source of truth.
            if let Ok(mut recs) = recipients.try_write() {
                if let Some(recipient) = recs.get_mut(index) {
                    recipient.with_mut(|r| {
                        let current_npt = sourced_amount_to_npt(&r.amount, &rate).unwrap_or_default();

                        if new_mode == CurrencyMode::Fiat {
                            let new_fiat_val = npt_to_fiat(&current_npt, &rate);
                            r.amount.source_kind = InputKind::Fiat(fiat_currency);
                            r.amount.source_value = new_fiat_val.to_string();
                        } else {
                            r.amount.source_kind = InputKind::Npt;
                            r.amount.source_value = current_npt.display_n_decimals(NPT_MAX_DECIMAL_DIGITS as usize);
                        }
                    });
                }
            }
        }
    };

    rsx! {
        if let Some(popup_ui) = popup_slot() {
            {popup_ui}
        }

        NoTitleModal {
            is_open: is_address_actions_modal_open,
            div {
                style: "display: flex; flex-direction: column; gap: 1rem;",
                h3 { "Set Address" },
                p {
                    {
                        if let Some(index) = action_target_index() {
                            format!("Choose an action for recipient number {}.", index + 1)
                        } else {
                            "Choose an action.".to_string()
                        }
                    }
                },
                Button { on_click: move |_| { if action_target_index().is_some() { spawn(async move { if let Some(ct) = crate::compat::clipboard_get().await { handle_scanned_data(ct); } }); } is_address_actions_modal_open.set(false); }, "Paste Address" },
                Button { on_click: move |_| { is_address_actions_modal_open.set(false); is_qr_scanner_modal_open.set(true); }, "Scan QR Code" },
                Button { on_click: move |_| { is_address_actions_modal_open.set(false); is_qr_upload_modal_open.set(true); }, "Upload QR Image" },
                Button { button_type: ButtonType::Secondary, outline: true, on_click: move |_| is_address_actions_modal_open.set(false), "Cancel" }
            }
        },
        NoTitleModal { is_open: is_qr_scanner_modal_open, QrScanner { on_scan: move |d| { handle_scanned_data(d); is_qr_scanner_modal_open.set(false); }, on_close: move |_| is_qr_scanner_modal_open.set(false) } },
        NoTitleModal { is_open: is_qr_upload_modal_open, QrUploader { on_scan: move |d| { handle_scanned_data(d); is_qr_upload_modal_open.set(false); }, on_close: move |_| is_qr_upload_modal_open.set(false) } },

        Modal {
            is_open: show_error_modal,
            title: "Error".to_string(),
            p { "{error_modal_message}" }
            footer {
                Button { on_click: move |_| show_error_modal.set(false), "Close" }
            }
        },
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
                        oninput: move |e| suppress_duplicate_warning.set(e.value() == "true")
                    }
                    "Don't ask me again"
                }
            }
            footer {
                Button { button_type: ButtonType::Secondary, outline: true, on_click: move |_| show_duplicate_warning_modal.set(false), "Cancel" },
                Button {
                    on_click: move |_| {
                        if let (Some(addr), Some(index)) = (pending_address.take(), action_target_index()) {
                             if let Ok(mut recs) = recipients.try_write() {
                                 if let Some(target) = recs.get_mut(index) {
                                    target.with_mut(|r| { r.address_str = addr; r.address_error = None; });
                                }
                             }
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
                    div {
                        style: "display: flex; flex-direction: column; height: 75vh;",
                        h3 { style: "margin: 0 0 0.5rem 0; padding: 0 0.5rem;", "Add Recipients" },
                        div {
                            style: "flex-grow: 1; overflow-y: auto; padding: 0 0.5rem;",
                            Card {
                                for (i, recipient) in recipients.iter().enumerate() {
                                    EditableRecipientRow {
                                        key: "{recipient.read().id}",
                                        index: i,
                                        recipient: *recipient,
                                        preferred_currency_mode: preferred_currency_mode,
                                        on_delete: move |idx| { if recipients.read().len() > 1 { if let Ok(mut recs) = recipients.try_write() { recs.remove(idx); } if active_row_index() == Some(idx) { active_row_index.set(None); } } },
                                        on_open_address_actions: move |idx| { if active_row_index() == Some(idx) { action_target_index.set(Some(idx)); is_address_actions_modal_open.set(true); } },
                                        popup_setter: popup_slot,
                                        can_delete: recipients.read().len() > 1,
                                        is_active: active_row_index() == Some(i),
                                        on_set_active: move |idx| active_row_index.set(Some(idx)),
                                        on_done_editing: move |_| active_row_index.set(None),
                                        is_any_other_row_active: is_any_row_active() && active_row_index() != Some(i),
                                        on_amount_input: update_recipient_value.clone(),
                                        on_keypad_press: on_recipient_keypad_press.clone(),
                                        on_currency_toggle: on_recipient_currency_toggle.clone(),
                                    }
                                }
                            }
                        },
                        div {
                            style: "flex-shrink: 0; padding: 0.25rem 1rem; background: var(--pico-background-color); border-top: 1px solid var(--pico-muted-border-color); display: flex; justify-content: space-between; align-items: center;",
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                on_click: move |_| {
                                    if let Ok(mut recs) = recipients.try_write() {
                                        let initial_kind = if *preferred_currency_mode.read() == CurrencyMode::Npt { InputKind::Npt } else { InputKind::Fiat(fiat_currency) };
                                        recs.insert(0, Signal::new(EditableRecipient { amount: SourcedAmount::new(initial_kind), ..Default::default() }));
                                    }
                                    active_row_index.set(Some(0));
                                },
                                disabled: is_any_row_active(),
                                "Add Another Recipient"
                            },
                            div {
                                style: "text-align: right; line-height: 1.2;",
                                div { "{subtotals().0} NPT" }
                                if show_fiat_toggle {
                                    small { style: "color: var(--pico-muted-color);", "{subtotals().1} {subtotals().1.currency().code()}" }
                                }
                            },
                            Button {
                                on_click: move |_| {
                                    if are_recipients_valid() {
                                        // Reset the fee input when navigating to the fee step.
                                        let initial_kind = if *preferred_currency_mode.read() == CurrencyMode::Npt {
                                            InputKind::Npt
                                        } else {
                                            InputKind::Fiat(fiat_currency)
                                        };
                                        fee_input.set(SourcedAmount::new(initial_kind));
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
                            let npt = sourced_amount_to_npt(&fee_input.read(), &rate).unwrap_or_default();
                            let fiat = npt_to_fiat(&npt, &rate);
                            (npt, fiat)
                        };
                        let total_spend_npt = subtotals().0 + fee_npt;
                        let total_spend_fiat = subtotals().1 + fee_fiat;

                        rsx! {
                             Card {
                                h3 { "Set Fee" },
                                p {
                                    "Subtotal: {subtotals().0} NPT"
                                    if show_fiat_toggle {
                                        span { style: "color: var(--pico-muted-color);", " ({subtotals().1} {subtotals().1.currency().code()})" }
                                    }
                                },
                                hr {},
                                label {
                                    if *preferred_currency_mode.read() == CurrencyMode::Npt { "Fee (NPT)" } else { "Fee ({fiat_currency.code()})" }
                                },
                                 div {
                                    style: "display: flex; align-items: center; gap: 0.5rem;",
                                    CurrencyAmountInput {
                                        value: local_fee_amount_str,
                                        on_input: {
                                            let rate = rate_rc.clone();
                                            move |sanitized_value: String| {
                                                if let Ok(mut fi) = fee_input.try_write() {
                                                    let display_mode = *preferred_currency_mode.read();
                                                    let new_source_kind = if display_mode == CurrencyMode::Npt {
                                                        InputKind::Npt
                                                    } else {
                                                        InputKind::Fiat(fiat_currency)
                                                    };
                                                    if !kinds_match(fi.source_kind, new_source_kind) {
                                                        fi.source_kind = new_source_kind;
                                                    }
                                                    fi.source_value = sanitized_value;

                                                    match sourced_amount_to_npt(&fi, &rate) {
                                                        Ok(_) => fee_error.set(None),
                                                        Err(e) if !fi.source_value.is_empty() => fee_error.set(Some(e)),
                                                        _ => fee_error.set(None),
                                                    }
                                                }
                                            }
                                        },
                                        on_keypad_press: {
                                            let rate = rate_rc.clone();
                                            move |key: String| {
                                                if let Ok(mut fi) = fee_input.try_write() {
                                                    let display_mode = *preferred_currency_mode.read();
                                                    let new_source_kind = if display_mode == CurrencyMode::Npt {
                                                        InputKind::Npt
                                                    } else {
                                                        InputKind::Fiat(fiat_currency)
                                                    };
                                                    let value_to_modify = if !kinds_match(fi.source_kind, new_source_kind) {
                                                        String::new()
                                                    } else {
                                                        fi.source_value.clone()
                                                    };
                                                    let new_val = if key == "⌫" {
                                                        let mut chars = value_to_modify.chars();
                                                        chars.next_back();
                                                        chars.as_str().to_string()
                                                    } else {
                                                        value_to_modify + &key
                                                    };
                                                    let sanitized = sanitize_and_format(new_val, fee_max_integers, fee_max_decimals);
                                                    fi.source_value = sanitized;
                                                    fi.source_kind = new_source_kind;

                                                    match sourced_amount_to_npt(&fi, &rate) {
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
                                        placeholder: "0.0".to_string()
                                    },

                                    if show_fiat_toggle {
                                        button {
                                            style: "width: 5rem; margin-bottom: 0;",
                                            onclick: {
                                                let rate = rate_rc.clone();
                                                move |_| {
                                                    // First, toggle the global display mode.
                                                    let new_mode = if *preferred_currency_mode.read() == CurrencyMode::Npt {
                                                        CurrencyMode::Fiat
                                                    } else {
                                                        CurrencyMode::Npt
                                                    };
                                                    preferred_currency_mode.set(new_mode);

                                                    // Then, convert the fee_input's value and set it as the new source of truth.
                                                    if let Ok(mut fi) = fee_input.try_write() {
                                                        let current_npt = sourced_amount_to_npt(&fi, &rate).unwrap_or_default();

                                                        if new_mode == CurrencyMode::Fiat {
                                                            // We are now displaying Fiat.
                                                            let new_fiat_val = npt_to_fiat(&current_npt, &rate);
                                                            fi.source_kind = InputKind::Fiat(fiat_currency);
                                                            fi.source_value = new_fiat_val.to_string();
                                                        } else {
                                                            // We are now displaying NPT.
                                                            fi.source_kind = InputKind::Npt;
                                                            fi.source_value = current_npt.display_n_decimals(NPT_MAX_DECIMAL_DIGITS as usize);
                                                        }
                                                    }
                                                }
                                            },
                                            {if *preferred_currency_mode.read() == CurrencyMode::Npt { fiat_currency.code().to_string() } else { "NPT".to_string() }}
                                        }
                                    }
                                },
                                if let Some(err) = fee_error() { small { style: "color: var(--pico-color-red-500);", "{err}" } },
                                div {
                                    style: "margin-top: 1rem; text-align: right;",
                                     h4 { "Total Spend" },
                                     Amount { amount: total_spend_npt, fiat_equivalent: Some(total_spend_fiat) }
                                     if show_fiat_toggle { div { style: "color: var(--pico-muted-color); font-size: 0.9em;", "{total_spend_fiat} {total_spend_fiat.currency().code()}" } }
                                },
                                footer {
                                     Button { button_type: ButtonType::Secondary, outline: true, on_click: move |_| wizard_step.set(WizardStep::AddRecipients), "Back" },
                                     Button { on_click: move |_| wizard_step.set(WizardStep::Review), disabled: !is_fee_valid(), "Next: Review" }
                                }
                            }
                        }
                    }
                },
                 WizardStep::Review => rsx! {
                    {
                        let rate = rate_rc.clone();
                        let fee_npt = sourced_amount_to_npt(&fee_input.read(), &rate).unwrap_or_default();
                        let total_spend_npt = subtotals().0 + fee_npt;

                        let fiat_fee_display = npt_to_fiat(&fee_npt, &rate);
                        let fiat_total_display = subtotals().1 + fiat_fee_display;

                        rsx! {
                            Card {
                                h3 { "Review Transaction" },
                                p { "Please review the details below. This action cannot be undone." },
                                h5 { style: "margin-top: 1rem;", "Recipients:" },
                                table {
                                    role: "grid",
                                    tbody {
                                        for recipient_signal in recipients.read().iter() {
                                            {
                                                let recipient = recipient_signal.read();
                                                let final_npt_amount = sourced_amount_to_npt(&recipient.amount, &rate).unwrap();
                                                let fiat_equiv = Some(npt_to_fiat(&final_npt_amount, &rate));
                                                let addr = Rc::new(ReceivingAddress::from_bech32m(&recipient.address_str, network).unwrap());
                                                rsx! {
                                                    tr {
                                                        td { Address { address: addr.clone() } }
                                                        td { style: "text-align: right;", Amount { amount: final_npt_amount, fiat_equivalent: fiat_equiv } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                div {
                                    style: "text-align: right; margin-top: 1rem;",
                                    strong { "Fee: " },
                                    Amount { amount: fee_npt, fiat_equivalent: Some(fiat_fee_display) }
                                    if show_fiat_toggle {
                                        span { style: "color: var(--pico-muted-color); font-weight: normal; margin-left: 0.5rem;", "({fiat_fee_display} {fiat_fee_display.currency().code()})" }
                                    }
                                },
                                 div {
                                    style: "text-align: right; margin-top: 0.5rem; font-weight: bold; border-top: 1px solid var(--pico-secondary-border); padding-top: 0.5rem; display: grid; grid-template-columns: 1fr auto;",
                                    span { style: "justify-self: end; padding-right: 1rem;", "Total Spend: " },
                                    div {
                                        Amount { amount: total_spend_npt, fiat_equivalent: Some(fiat_total_display) }
                                        if show_fiat_toggle {
                                            span { style: "color: var(--pico-muted-color); font-weight: normal; margin-left: 0.5rem;", "({fiat_total_display} {fiat_total_display.currency().code()} est.)" }
                                        }
                                    }
                                },
                                footer {
                                    Button { button_type: ButtonType::Secondary, outline: true, on_click: move |_| wizard_step.set(WizardStep::EnterFee), "Back" },
                                    Button {
                                        on_click: {
                                            let rate = rate_rc.clone();
                                            move |_| {
                                            let network = network;
                                            let recipients = recipients.clone();
                                            let fee_input = fee_input.clone();
                                            let mut api_response = api_response.clone();
                                            let mut wizard_step = wizard_step.clone();
                                            let rate = rate.clone();

                                            spawn(async move {
                                                let outputs: Vec<OutputFormat> = recipients.read().iter().map(|rs| {
                                                    let r = rs.read();
                                                    let addr = ReceivingAddress::from_bech32m(&r.address_str, network).unwrap();
                                                    let amount = sourced_amount_to_npt(&r.amount, &rate).unwrap();
                                                    OutputFormat::AddressAndAmount(addr, amount)
                                                }).collect();
                                                let fee = sourced_amount_to_npt(&fee_input.read(), &rate).unwrap_or_default();
                                                let result = api::send(outputs, ChangePolicy::default(), fee).await;
                                                api_response.set(Some(result));
                                                wizard_step.set(WizardStep::Status);
                                            });
                                        }},
                                        "Confirm & Send"
                                    }
                                }
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
                                    p { style: "color: var(--pico-color-green-500);", "Transaction sent successfully!" },
                                    div {
                                        style: "display: flex; justify-content: space-between; align-items: center; margin-top: 1.5rem; margin-bottom: 1.5rem; padding: 0.75rem; border: 1px solid var(--pico-secondary-border); border-radius: var(--pico-border-radius);",
                                        strong { "Transaction ID" },
                                        div { style: "display: flex; align-items: center; gap: 0.5rem;", code { "{kernel_id}" }, CopyButton { text_to_copy: kernel_id.to_string() } }
                                    },
                                    div {
                                        style: "display: flex; gap: 1rem; margin-top: 1.5rem; flex-wrap: wrap;",
                                        Button { button_type: ButtonType::Primary, outline: true, on_click: move |_| { active_screen.set(Screen::MempoolTx(kernel_id)); }, "View in Mempool" },
                                        Button { on_click: move |_| reset_screen(), "Send Another Transaction" }
                                    }
                                },
                                Err(err) => rsx! {
                                    h4 { style: "color: var(--pico-color-red-500);", "Error Sending Transaction" },
                                    p { "{err}" },
                                    div {
                                        style: "display: flex; gap: 1rem; margin-top: 1.5rem; flex-wrap: wrap;",
                                        Button { button_type: ButtonType::Secondary, outline: true, on_click: move |_| wizard_step.set(WizardStep::Review), "Back" },
                                        Button { on_click: move |_| reset_screen(), "Send Another Transaction" }
                                    }
                                }
                            }
                        }
                    } else {
                        Card {
                            h3 { "Sending Transaction..." },
                            p { "Please wait." },
                            progress {}
                        }
                    }
                }
            }
        }
    }
}