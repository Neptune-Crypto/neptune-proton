use crate::app_state::AppState;
use crate::app_state_mut::{AppStateMut, DisplayCurrency};
use crate::components::amount::Amount;
use crate::components::block::Block;
use crate::components::pico::Card;
use api::fiat_currency::FiatCurrency;
use dioxus::prelude::*;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use num_traits::CheckedSub;
use num_traits::Zero;
use std::rc::Rc;

/// A responsive container for a section of the dashboard.
#[component]
fn InfoCard(children: Element) -> Element {
    rsx! {
        article {
            style: "border: 1px solid var(--pico-card-border-color); border-radius: var(--pico-border-radius); padding: 1rem; background-color: var(--pico-card-background-color);",
            // The title is now passed in as part of children to allow for more complex headers
            {children}
        }
    }
}

/// A small, reusable component for displaying a key-value pair.
#[component]
fn InfoItem(label: String, children: Element) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0; border-bottom: 1px solid var(--pico-secondary-border);",
            dt { style: "font-weight: 500;", "{label}" }
            dd { style: "margin: 0; text-align: right;", {children} }
        }
    }
}

/// A specialized component for displaying the two-part balance rows.
#[component]
fn BalanceRow(available: NativeCurrencyAmount, total: NativeCurrencyAmount) -> Element {
    let time_locked = total.checked_sub(&available).unwrap_or_default();
    rsx! {
        InfoItem {
            label: "Available".to_string(),
            Amount { amount: available }
        }
        if time_locked > NativeCurrencyAmount::zero() {
            InfoItem {
                label: "Time-locked".to_string(),
                Amount { amount: time_locked }
            }
            InfoItem {
                label: "Total".to_string(),
                Amount { amount: total }
            }
        }
    }
}

#[component]
pub fn BalanceScreen() -> Element {
    let app_state = use_context::<AppState>();
    let mut app_state_mut = use_context::<AppStateMut>();
    let mut dashboard_data =
        use_resource(move || async move { api::dashboard_overview_data().await });

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let mut data_resource = dashboard_data;
        async move {
            loop {
                // Correct for WASM targets: use a timer from the `gloo` ecosystem.
                crate::compat::sleep(std::time::Duration::from_millis(5000)).await;
                data_resource.restart();
            }
        }
    });

    rsx! {
        match &*dashboard_data.read() {
            None => rsx! {
                Card {
                    h3 { "Wallet Overview" }
                    p { "Loading..." }
                    progress {}
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load dashboard data: {e}" }
                    button { onclick: move |_| dashboard_data.restart(), "Retry" }
                }
            },
            Some(Ok(data)) => {
                let status_color = if data.syncing { "var(--pico-color-green-500)" } else { "var(--pico-color-amber-500)" };
                let sync_text = if data.syncing { "Syncing..." } else { "Synced" };
                let block_digest = Rc::new(data.tip_digest);
                let height = Rc::new(data.tip_header.height);

                let show_unconfirmed = data.unconfirmed_available_balance != data.confirmed_available_balance
                    || data.unconfirmed_total_balance != data.confirmed_total_balance;

                let balance_grid_style = if show_unconfirmed {
                    "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem 2rem;"
                } else {
                    "display: block;"
                };

                let mining_status_str = std::fmt::format(format_args!("{}", data.mining_status.unwrap_or_default()));
                let proving_capability_str = std::fmt::format(format_args!("{}", data.proving_capability));

                // Logic for the global display toggle button
                let toggle_text = match *app_state_mut.display_currency.read() {
                    DisplayCurrency::Npt => "USD", // Show what it will change to
                    DisplayCurrency::Fiat(_) => "NPT",
                };
                let toggle_onclick = move |_| {
                    let new_currency = match *app_state_mut.display_currency.read() {
                        DisplayCurrency::Npt => DisplayCurrency::Fiat(FiatCurrency::USD),
                        DisplayCurrency::Fiat(_) => DisplayCurrency::Npt,
                    };
                    app_state_mut.display_currency.set(new_currency);
                };

                rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem;",

                        InfoCard {
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--pico-secondary-border); margin-bottom: 1rem; padding-bottom: 0.5rem;",
                                h5 { style: "margin: 0;", "Confirmed Balance" }
                                button {
                                    class: "outline secondary",
                                    style: "padding: 0.2rem 0.5rem; font-size: 0.8em; margin-left: 1rem; min-width: 55px;",
                                    onclick: toggle_onclick,
                                    "{toggle_text}"
                                }
                            }
                            dl {
                                style: "margin: 0;",
                                div {
                                    style: "{balance_grid_style}",
                                    BalanceRow {
                                        available: data.confirmed_available_balance,
                                        total: data.confirmed_total_balance,
                                    }
                                }
                            }
                        }

                        if show_unconfirmed {
                            InfoCard {
                                h5 {
                                    style: "margin-top: 0; margin-bottom: 1rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;",
                                    "Unconfirmed Balance"
                                }
                                dl {
                                    style: "margin: 0;",
                                    div {
                                        style: "{balance_grid_style}",
                                        BalanceRow {
                                            available: data.unconfirmed_available_balance,
                                            total: data.unconfirmed_total_balance,
                                        }
                                    }
                                }
                            }
                        }

                        InfoCard {
                            h5 { style: "margin-top: 0; margin-bottom: 1rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;", "Blockchain" }
                            dl {
                                style: "margin: 0;",
                                InfoItem {
                                    label: "Network".to_string(),
                                    span { "{app_state.network}" }
                                }
                                InfoItem {
                                    label: "Status".to_string(),
                                    span { style: "color: {status_color};", "{sync_text}" }
                                }
                                InfoItem {
                                    label: "Tip".to_string(),
                                    Block { block_digest, height }
                                }
                            }
                        }

                        InfoCard {
                            h5 { style: "margin-top: 0; margin-bottom: 1rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;", "Mempool" }
                            dl {
                                style: "margin: 0;",
                                InfoItem {
                                    label: "Transactions".to_string(),
                                    span { "{data.mempool_total_tx_count}" }
                                }
                                InfoItem {
                                    label: "Size (bytes)".to_string(),
                                    span { "{data.mempool_size}" }
                                }
                            }
                        }

                        InfoCard {
                            h5 { style: "margin-top: 0; margin-bottom: 1rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;", "Network Peers" }
                            dl {
                                style: "margin: 0;",
                                InfoItem {
                                    label: "Connected Peers".to_string(),
                                    span { "{data.peer_count.unwrap_or_default()}" }
                                }
                                InfoItem {
                                    label: "Max Peers".to_string(),
                                    span { "{data.max_num_peers}" }
                                }
                            }
                        }

                        InfoCard {
                            h5 { style: "margin-top: 0; margin-bottom: 1rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;", "Node Info" }
                            dl {
                                style: "margin: 0;",
                                InfoItem {
                                    label: "Mining Status".to_string(),
                                    span { "{mining_status_str}" }
                                }
                                InfoItem {
                                    label: "Proving Capability".to_string(),
                                    code { "{proving_capability_str}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
