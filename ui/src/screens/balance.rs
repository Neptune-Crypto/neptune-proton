// ui/src/screens/balance.rs
use std::rc::Rc;

use api::fiat_amount::FiatAmount;
use api::fiat_currency::FiatCurrency;
use api::prefs::display_preference::DisplayPreference;
use dioxus::prelude::*;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use num_traits::CheckedSub;
use num_traits::Zero;
use strum::IntoEnumIterator;

use crate::components::amount::Amount;
use crate::components::block::Block;
use crate::components::currency_chooser::CurrencyChooser;
use crate::components::currency_chooser::CurrencyInfo;
use crate::components::pico::Card;
use crate::currency::npt_to_fiat;
use crate::AppState;
use crate::AppStateMut;
use crate::hooks::use_rpc_checker::use_rpc_checker;

/// A responsive container for a section of the dashboard.
#[component]
fn InfoCard(title: String, children: Element) -> Element {
    rsx! {
        article {
            style: "margin-bottom: 0; border: 1px solid var(--pico-card-border-color); border-radius: var(--pico-border-radius); padding: 0.5rem; background-color: var(--pico-card-background-color);",
            h5 {
                style: "margin-top: 0; margin-bottom: 0.5rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;",
                "{title}"
            }
            dl {
                style: "margin: 0;",
                {children}
            }
        }
    }
}

/// A small, reusable component for displaying a key-value pair.
#[component]
fn InfoItem(label: String, children: Element) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; padding: 0.3rem 0; border-bottom: 1px solid var(--pico-secondary-border);",
            dt {
                style: "font-weight: 500;",
                "{label}"
            }
            dd {
                style: "margin: 0; text-align: right;",
                {children}
            }
        }
    }
}

/// A specialized component for displaying the two-part balance rows.
#[component]
fn BalanceRow(
    available: NativeCurrencyAmount,
    total: NativeCurrencyAmount,
    #[props(optional)] available_fiat: Option<FiatAmount>,
    #[props(optional)] total_fiat: Option<FiatAmount>,
) -> Element {
    let time_locked = total.checked_sub(&available).unwrap_or_default();
    let time_locked_fiat = match (available_fiat, total_fiat) {
        (Some(avail), Some(tot)) if avail.currency() == tot.currency() => {
            Some(FiatAmount::new_from_minor(
                tot.as_minor_units() - avail.as_minor_units(),
                tot.currency(),
            ))
        }
        _ => None,
    };

    rsx! {
        InfoItem {
            label: "Available".to_string(),
            Amount {
                amount: available,
                fiat_equivalent: available_fiat,
            }
        }
        if time_locked > NativeCurrencyAmount::zero() {
            InfoItem {
                label: "Time-locked".to_string(),
                Amount {
                    amount: time_locked,
                    fiat_equivalent: time_locked_fiat,
                }
            }
            InfoItem {
                label: "Total".to_string(),
                Amount {
                    amount: total,
                    fiat_equivalent: total_fiat,
                }
            }
        }
    }
}

#[component]
pub fn BalanceScreen() -> Element {
    let mut rpc = use_rpc_checker(); // Initialize Hook
    let app_state = use_context::<AppState>();
    let app_state_mut = use_context::<AppStateMut>();
    let network = app_state.network;
    let mut dashboard_data =
        use_resource(move || async move {
            api::dashboard_overview_data().await
        });

    // Effect: Restarts the resource when connection is restored.
    let status_sig = rpc.status();
    use_effect(move || {
        if status_sig.read().is_connected() {
            dashboard_data.restart();
        }
    });

    // Coroutine: Polls every 5 seconds while connected.
    // This ensures we detect if the connection dies while sitting on this screen.
    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let rpc_status = rpc.status(); // Use signal handle
        let mut data_resource = dashboard_data;

        async move {
            loop {
                // Wait 5 seconds
                crate::compat::sleep(std::time::Duration::from_millis(5000)).await;

                // Only restart (poll) if we believe we are connected.
                // If disconnected, the global AppBody loop handles the "pinging".
                if (*rpc_status.read()).is_connected() {
                    data_resource.restart();
                }
            }
        }
    });

    rsx! {
        match &*dashboard_data.read() {
            None => rsx! {
                Card {

                    h3 {

                        "Wallet Overview"
                    }
                    p {

                        "Loading..."
                    }
                    progress {


                    }
                }
            },
            Some(result) if !rpc.check_result_ref(&result) => rsx! {
                // modal ConnectionLost is displayed by rpc.check_result_ref
                Card {
                    h3 {
                        "Wallet Overview"
                    }
                }
            },
            Some(Err(e)) => rsx! {
                Card {

                    h3 {

                        "Error"
                    }
                    p {

                        "Failed to load dashboard data: {e}"
                    }
                    button {
                        onclick: move |_| dashboard_data.restart(),
                        "Retry"
                    }
                }
            },
            Some(Ok(data)) => {
                let status_color = if data.syncing {
                    "var(--pico-color-green-500)"
                } else {
                    "var(--pico-color-amber-500)"
                };
                let sync_text = if data.syncing { "Syncing..." } else { "Synced" };
                let block_digest = Rc::new(data.tip_digest);
                let height = Rc::new(data.tip_header.height);
                let show_unconfirmed = data.unconfirmed_available_balance
                    != data.confirmed_available_balance
                    || data.unconfirmed_total_balance != data.confirmed_total_balance;
                let balance_grid_style = if show_unconfirmed {
                    "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem 2rem;"
                } else {
                    "display: block;"
                };
                let mining_status_str = std::fmt::format(
                    format_args!("{}", data.mining_status.unwrap_or_default()),
                );
                let proving_capability_str = std::fmt::format(
                    format_args!("{}", data.proving_capability),
                );
                let (rate, preferred_fiat_id_global, display_as_fiat, fiat_mode_active) = match *app_state_mut
                    .display_preference
                    .read()
                {
                    DisplayPreference::FiatEnabled { fiat, display_as_fiat, .. } => {
                        let price = app_state_mut
                            .prices
                            .read()
                            .as_ref()
                            .and_then(|p| p.get(fiat));
                        (price, fiat.code(), display_as_fiat, true)
                    }
                    DisplayPreference::NptOnly => (None, "", false, false),
                };
                let preferred_fiat_id = use_signal(|| preferred_fiat_id_global);
                let initial_display_id = if display_as_fiat {
                    preferred_fiat_id_global
                } else {
                    "NPT"
                };
                let displayed_id = use_signal(|| initial_display_id);
                use_effect({
                    let mut app_state_mut = app_state_mut;
                    move || {
                        let signal_preferred_fiat = *preferred_fiat_id.read();
                        let signal_display_is_fiat = *displayed_id.read() != "NPT";
                        app_state_mut
                            .display_preference
                            .with_mut(|pref| {
                                if let DisplayPreference::FiatEnabled {
                                    fiat,
                                    display_as_fiat,
                                    ..
                                } = pref {
                                    if fiat.code() != signal_preferred_fiat {
                                        if let Some(new_fiat) = FiatCurrency::iter()
                                            .find(|c| c.code() == signal_preferred_fiat)
                                        {
                                            *fiat = new_fiat;
                                        }
                                    }
                                    *display_as_fiat = signal_display_is_fiat;
                                }
                            });
                    }
                });
                let all_fiats: Vec<CurrencyInfo> = FiatCurrency::iter()
                    .map(|c| c.into())
                    .collect();
                let confirmed_available_fiat = rate
                    .as_ref()
                    .map(|r| npt_to_fiat(&data.confirmed_available_balance, r));
                let confirmed_total_fiat = rate
                    .as_ref()
                    .map(|r| npt_to_fiat(&data.confirmed_total_balance, r));
                let unconfirmed_available_fiat = rate
                    .as_ref()
                    .map(|r| npt_to_fiat(&data.unconfirmed_available_balance, r));
                let unconfirmed_total_fiat = rate
                    .as_ref()
                    .map(|r| npt_to_fiat(&data.unconfirmed_total_balance, r));
                rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem;",
                        article {
                            style: "margin-bottom: 0px; border: 1px solid var(--pico-card-border-color); border-radius: var(--pico-border-radius); padding: 0.5rem; background-color: var(--pico-card-background-color);",
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--pico-secondary-border); margin-top: 0; margin-bottom: 0.5rem;",
                                h5 {
                                    style: "margin-top: 0; margin-bottom: 0;",
                                    "Confirmed Balance"
                                }
                                {fiat_mode_active.then(|| rsx! {
                                    small {

                                        CurrencyChooser {
                                            displayed_id,
                                            preferred_fiat_id,
                                            all_fiats,
                                        }
                                    }
                                })}
                            }
                            dl {
                                style: "margin: 0;",
                                div {
                                    style: "{balance_grid_style}",
                                    BalanceRow {
                                        available: data.confirmed_available_balance,
                                        total: data.confirmed_total_balance,
                                        available_fiat: confirmed_available_fiat,
                                        total_fiat: confirmed_total_fiat,
                                    }
                                }
                            }
                        }
                        if show_unconfirmed {
                            InfoCard {
                                title: "Unconfirmed Balance".to_string(),
                                div {
                                    style: "{balance_grid_style}",
                                    BalanceRow {
                                        available: data.unconfirmed_available_balance,
                                        total: data.unconfirmed_total_balance,
                                        available_fiat: unconfirmed_available_fiat,
                                        total_fiat: unconfirmed_total_fiat,
                                    }
                                }
                            }
                        }
                        InfoCard {
                            title: "Blockchain".to_string(),
                            InfoItem {
                                label: "Network".to_string(),
                                span {

                                    "{network}"
                                }
                            }
                            InfoItem {
                                label: "Status".to_string(),
                                span {
                                    style: "color: {status_color};",
                                    "{sync_text}"
                                }
                            }
                            InfoItem {
                                label: "Tip".to_string(),
                                Block {
                                    block_digest,
                                    height,
                                }
                            }
                        }
                        InfoCard {
                            title: "Mempool".to_string(),
                            InfoItem {
                                label: "Transactions".to_string(),
                                span {

                                    "{data.mempool_total_tx_count}"
                                }
                            }
                            InfoItem {
                                label: "Size (bytes)".to_string(),
                                span {

                                    "{data.mempool_size}"
                                }
                            }
                        }
                        InfoCard {
                            title: "Network Peers".to_string(),
                            InfoItem {
                                label: "Connected Peers".to_string(),
                                span {

                                    "{data.peer_count.unwrap_or_default()}"
                                }
                            }
                            InfoItem {
                                label: "Max Peers".to_string(),
                                span {

                                    "{data.max_num_peers}"
                                }
                            }
                        }
                        InfoCard {
                            title: "Node Info".to_string(),
                            InfoItem {
                                label: "Mining Status".to_string(),
                                span {

                                    "{mining_status_str}"
                                }
                            }
                            InfoItem {
                                label: "Proving Capability".to_string(),
                                code {

                                    "{proving_capability_str}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
