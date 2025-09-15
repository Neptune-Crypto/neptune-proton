use crate::components::block::Block;
use crate::components::block::BlockProps;
use crate::components::pico::Card;
use crate::AppState;
use dioxus::prelude::*;
use neptune_types::{
    dashboard_overview_data_from_client::DashBoardOverviewDataFromClient,
    native_currency_amount::NativeCurrencyAmount,
};
use num_traits::CheckedSub;
use num_traits::Zero;
use std::rc::Rc;

/// A responsive container for a section of the dashboard.
#[component]
fn InfoCard(title: String, children: Element) -> Element {
    rsx! {
        article {
            style: "border: 1px solid var(--pico-card-border-color); border-radius: var(--pico-border-radius); padding: 1rem; background-color: var(--pico-card-background-color);",
            h5 {
                style: "margin-top: 0; margin-bottom: 1rem; border-bottom: 1px solid var(--pico-secondary-border); padding-bottom: 0.5rem;",
                "{title}"
            }
            dl { style: "margin: 0;", {children} }
        }
    }
}

/// A small, reusable component for displaying a key-value pair.
/// It correctly receives its value as `children`.
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
        InfoItem{
            label: "Available".to_string(),
            span { "{available}" }
        }
        if time_locked > NativeCurrencyAmount::zero() {
            InfoItem{
                label: "Time-locked".to_string(),
                span { "{time_locked}" }
            }
            InfoItem{
                label: "Total".to_string(),
                span { "{total}" }
            }
        }
    }
}

#[component]
pub fn BalanceScreen() -> Element {
    let network = use_context::<AppState>().network;
    let mut dashboard_data =
        use_resource(move || async move { api::dashboard_overview_data().await });

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let mut data_resource = dashboard_data.clone();
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

                rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem;",

                        InfoCard {
                            title: "Confirmed Balance".to_string(),
                            div {
                                style: "{balance_grid_style}",
                                BalanceRow {
                                    available: data.confirmed_available_balance,
                                    total: data.confirmed_total_balance,
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
                                    }
                                }
                            }
                        }

                        InfoCard {
                            title: "Blockchain".to_string(),
                            InfoItem {
                                label: "Network".to_string(),
                                span { "{network}" }
                            }
                            InfoItem {
                                label: "Status".to_string(),
                                span { style: "color: {status_color};", "{sync_text}" }
                            }
                            InfoItem {
                                label: "Tip".to_string(),
                                Block{ block_digest, height }
                            }
                            // InfoItem {
                            //     label: "Proof of Work".to_string(),
                            //     code { "{data.total_pow}" }
                            // }
                        }

                        InfoCard {
                            title: "Mempool".to_string(),
                            InfoItem {
                                label: "Transactions".to_string(),
                                span { "{data.mempool_total_tx_count}" }
                            }
                            InfoItem {
                                label: "Size (bytes)".to_string(),
                                span { "{data.mempool_size}" }
                            }
                        }

                        InfoCard {
                            title: "Network Peers".to_string(),
                            InfoItem {
                                label: "Connected Peers".to_string(),
                                span { "{data.peer_count.unwrap_or_default()}" }
                            }
                            InfoItem {
                                label: "Max Peers".to_string(),
                                span { "{data.max_num_peers}" }
                            }
                        }

                        InfoCard {
                            title: "Node Info".to_string(),
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
