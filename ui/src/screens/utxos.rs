//=============================================================================
// File: src/screens/utxos.rs
//=============================================================================
use std::ops::Deref;
use std::rc::Rc;

use dioxus::prelude::*;
use neptune_types::block_height::BlockHeight;
use neptune_types::block_selector::BlockSelector;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::timestamp::Timestamp;
use neptune_types::ui_utxo::UiUtxo;
use neptune_types::ui_utxo::UtxoStatusEvent;

use crate::components::action_link::ActionLink;
use crate::components::amount::Amount;
use crate::components::amount::AmountType;
use crate::components::empty_state::EmptyState;
use crate::components::pico::Card;
use crate::hooks::use_rpc_checker::use_rpc_checker;
use crate::Screen;

const UTXOS_EMPTY_SVG: &str = include_str!("../../assets/svg/utxos-empty.svg");

#[derive(Clone, Copy, PartialEq)]
enum SortableColumn {
    Received,
    Index,
    Amount,
    Releases,
    Spent,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, PartialEq)]
enum DisplayMode {
    Date,
    DateTime,
    BlockHeight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UiUtxoReadOnly(Rc<UiUtxo>);

impl Deref for UiUtxoReadOnly {
    type Target = UiUtxo;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn get_event_sort_key(event: &UtxoStatusEvent) -> u64 {
    match event {
        UtxoStatusEvent::Confirmed { timestamp, .. } => timestamp.to_millis(),
        UtxoStatusEvent::Pending => u64::MAX,
        UtxoStatusEvent::Expected => u64::MAX - 1,
        UtxoStatusEvent::Abandoned => 0,
        UtxoStatusEvent::None => 0,
    }
}

#[component]
fn BlockHeightDisplay(height: BlockHeight) -> Element {
    let active_screen = use_context::<Signal<Screen>>();

    rsx! {
        ActionLink {
            state: active_screen,
            to: Screen::Block(BlockSelector::Height(height)),
            "{height}"
        }
    }
}

#[component]
fn UtxoEventDisplay(event: UtxoStatusEvent, mode: Signal<DisplayMode>) -> Element {
    let tooltip_text = match event {
        UtxoStatusEvent::Confirmed {
            block_height,
            timestamp,
        } => {
            format!("{} (Block {})", timestamp.standard_format(), block_height)
        }
        UtxoStatusEvent::Pending => "Exists in mempool.  Unconfirmed in a  block.".to_string(),
        UtxoStatusEvent::Expected => {
            "We expect to receive this UTXO but it has not yet been confirmed in a block."
                .to_string()
        }
        UtxoStatusEvent::Abandoned => "Never confirmed in a block".to_string(),
        UtxoStatusEvent::None => "Not yet spent".to_string(),
    };

    match event {
        UtxoStatusEvent::Confirmed {
            block_height,
            timestamp,
        } => {
            rsx! {
                span {
                    title: "{tooltip_text}",
                    style: "cursor: help; border-bottom: 1px dotted var(--pico-muted-border-color);",
                    match *mode.read() {
                        DisplayMode::Date => rsx! { {timestamp.format("%Y-%m-%d")} },
                        DisplayMode::DateTime => rsx! { {timestamp.format("%Y-%m-%d %H:%M")} },
                        DisplayMode::BlockHeight => rsx! { BlockHeightDisplay { height: block_height } },
                    }
                }
            }
        }
        _ => {
            let text = event.to_string();

            rsx! {
                span {
                    title: "{tooltip_text}",
                    style: "cursor: help; border-bottom: 1px dotted var(--pico-muted-border-color);",
                    {text}
                }
            }
        }
    }
}

#[component]
fn SortableHeader(
    title: &'static str,
    column: SortableColumn,
    sort_column: Signal<SortableColumn>,
    sort_direction: Signal<SortDirection>,
    style: Option<&'static str>,
) -> Element {
    let (arrow_char, is_active) = if *sort_column.read() == column {
        (
            match *sort_direction.read() {
                SortDirection::Ascending => "▲",
                SortDirection::Descending => "▼",
            },
            true,
        )
    } else {
        ("\u{00A0}", false)
    };

    rsx! {
        th {
            style: format!("{}{}", "position: sticky; top: 0; background: var(--pico-card-background-color); z-index: 20; cursor: pointer; white-space: nowrap; ", style.unwrap_or("")),
            onclick: move |_| {
                if is_active {
                    sort_direction.with_mut(|dir| {
                        *dir = match dir {
                            SortDirection::Ascending => SortDirection::Descending,
                            SortDirection::Descending => SortDirection::Ascending,
                        };
                    });
                } else {
                    sort_column.set(column);
                    sort_direction.set(SortDirection::Ascending);
                }
            },
            "{title}"
            span {
                style: "display: inline-block; width: 1.2em; text-align: right;",
                "{arrow_char}"
            }
        }
    }
}

#[component]
fn UtxoRow(utxo: UiUtxoReadOnly, display_mode: Signal<DisplayMode>) -> Element {
    let mut is_hovered = use_signal(|| false);

    let index_display = match utxo.aocl_leaf_index {
        Some(idx) => idx.to_string(),
        None => "-".to_string(),
    };

    let (released_display, released_tooltip) = match utxo.release_date {
        Some(ts) => {
            let text = match *display_mode.read() {
                DisplayMode::Date => ts.format("%Y-%m-%d"),
                _ => ts.format("%Y-%m-%d %H:%M"),
            };
            (text, format!("Can be spent after {}", ts.standard_format()))
        }
        None => ("-".to_string(), "Not Applicable".to_string()),
    };

    rsx! {
        tr {
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),

            td {
                UtxoEventDisplay {
                    event: utxo.received,
                    mode: display_mode
                }
            }
            td {
                "{index_display}"
            }
            td {
                style: "text-align: right; white-space: nowrap; min-width: 21ch;",
                Amount {
                    amount: utxo.amount,
                }
            }
            td {
                title: "{released_tooltip}",
                style: "cursor: help; border-bottom: 1px dotted var(--pico-muted-border-color);",
                "{released_display}"
            }
            td {
                UtxoEventDisplay {
                    event: utxo.spent,
                    mode: display_mode
                }
            }
        }
    }
}

#[component]
pub fn UtxosScreen() -> Element {
    let mut rpc = use_rpc_checker();
    let mut utxos_resource = use_resource(move || async move { api::list_utxos().await });

    // State for display mode
    let mut display_mode = use_signal(|| DisplayMode::Date);

    // State for sorting
    let sort_column = use_signal(|| SortableColumn::Received);
    let sort_direction = use_signal(|| SortDirection::Descending);

    let status_sig = rpc.status();
    use_effect(move || {
        if status_sig.read().is_connected() {
            utxos_resource.restart();
        }
    });

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let rpc_status = rpc.status();
        let mut data_resource = utxos_resource;
        async move {
            loop {
                crate::compat::sleep(std::time::Duration::from_secs(10)).await;
                if (*rpc_status.read()).is_connected() {
                    data_resource.restart();
                }
            }
        }
    });

    rsx! {
        match &*utxos_resource.read() {
            None => rsx! {
                Card { h3 { "UTXOs" }, p { "Loading..." }, progress {} }
            },
            Some(result) if !rpc.check_result_ref(&result) => rsx! {
                Card { h3 { "UTXOs" } }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load UTXOs: {e}" }
                    button { onclick: move |_| utxos_resource.restart(), "Retry" }
                }
            },
            Some(Ok(utxo_list)) if utxo_list.is_empty() => rsx! {
                Card {
                    h3 { "UTXOs" }
                    EmptyState {
                        title: "No UTXOs Found".to_string(),
                        description: Some("Your wallet currently holds no Unspent Transaction Outputs.".to_string()),
                        icon: rsx! {
                            span {
                                dangerous_inner_html: UTXOS_EMPTY_SVG,
                                style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                            }
                        }
                    }
                }
            },
            Some(Ok(utxo_list)) => {
                let mut sorted_utxos = utxo_list.clone();
                sorted_utxos.sort_by(|a, b| {
                    let ordering = match sort_column() {
                        SortableColumn::Received => {
                            get_event_sort_key(&a.received).cmp(&get_event_sort_key(&b.received))
                        },
                        SortableColumn::Index => a.aocl_leaf_index.cmp(&b.aocl_leaf_index),
                        SortableColumn::Amount => a.amount.cmp(&b.amount),
                        SortableColumn::Releases => a.release_date.cmp(&b.release_date),
                        SortableColumn::Spent => {
                            get_event_sort_key(&a.spent).cmp(&get_event_sort_key(&b.spent))
                        },
                    };
                    match sort_direction() {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });

                rsx! {
                    Card {
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; width: 100%;",

                            h3 {
                                style: "margin-bottom: 0;",
                                "UTXOs "
                                small {
                                    style: "font-weight: normal; font-size: 0.8rem; color: var(--pico-muted-color); vertical-align: middle;",
                                    "({utxo_list.len()})"
                                }
                            }

                            select {
                                style: "width: auto; margin-bottom: 0; padding: 4px 8px; font-size: 0.9rem;",
                                onchange: move |evt| {
                                    match evt.value().as_str() {
                                        "date" => display_mode.set(DisplayMode::Date),
                                        "datetime" => display_mode.set(DisplayMode::DateTime),
                                        "height" => display_mode.set(DisplayMode::BlockHeight),
                                        _ => {}
                                    }
                                },
                                option { value: "date", selected: *display_mode.read() == DisplayMode::Date, "Date" }
                                option { value: "datetime", selected: *display_mode.read() == DisplayMode::DateTime, "Date & Time" }
                                option { value: "height", selected: *display_mode.read() == DisplayMode::BlockHeight, "Height" }
                            }
                        }

                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {
                                thead {
                                    tr {
                                        SortableHeader { title: "Received", column: SortableColumn::Received, sort_column, sort_direction }
                                        SortableHeader { title: "Index", column: SortableColumn::Index, sort_column, sort_direction }
                                        SortableHeader { title: "Amount", column: SortableColumn::Amount, sort_column, sort_direction, style: "text-align: right; padding-right: 0" }
                                        SortableHeader { title: "Releases", column: SortableColumn::Releases, sort_column, sort_direction }
                                        SortableHeader { title: "Spent", column: SortableColumn::Spent, sort_column, sort_direction }
                                    }
                                }
                                tbody {
                                    for utxo in sorted_utxos {
                                        UtxoRow {
                                            utxo: UiUtxoReadOnly(Rc::new(utxo)),
                                            display_mode: display_mode
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
