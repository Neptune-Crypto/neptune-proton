//=============================================================================
// File: src/screens/balance.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;

#[component]
pub fn BalanceScreen() -> Element {
    // 1. `use_resource` takes an async block that will be run in the background.
    //    It immediately returns a `Resource` signal.
    let mut balance = use_resource(move || async move {
        // Your async API call goes here.
        api::wallet_balance().await
    });

    rsx! {
        // 2. The `rsx!` macro reads the current state of the `balance` signal.
        match &*balance.read() {
            // The resource is still loading or has not been run yet.
            None => {
                rsx! {
                    Card {
                        h2 { "Current Balance" }
                        p { "Loading balance..." }
                        progress {} // An indeterminate progress bar
                    }
                }
            }
            // The async task finished successfully.
            Some(Ok(balance_amount)) => {
                rsx! {
                    Card {
                        h3 { "Current Balance" }
                        p { "{balance_amount} NPT" }
                        p { "$281,400.65 USD" } // Placeholder for USD conversion
                    }
                    Card {
                        h3 { "Asset Allocation" }
                        progress { value: "75", max: "100" }
                    }
                }
            }
            // The async task returned an error.
            Some(Err(e)) => {
                rsx! {
                    Card {
                        h3 { "Error" }
                        p { "Failed to load balance: {e}" }
                        // You could add a "Retry" button here
                        button {
                            onclick: move |_| balance.restart(),
                            "Retry"
                        }
                    }
                }
            }
        }
    }
}
