//=============================================================================
// File: src/screens/addresses.rs
//=============================================================================
use crate::components::pico::Card;
use dioxus::prelude::*;
use neptune_types::network::Network;
use neptune_types::address::AddressableKeyType;

#[allow(non_snake_case)]
#[component]
pub fn AddressesScreen() -> Element {

    let mut known_keys = use_resource(move || async move {
        // Your async API call goes here.
        api::known_keys().await
    });

    rsx! {

        match &*known_keys.read() {
            // The resource is still loading or has not been run yet.
            None => {
                rsx! {
                    Card {
                        h3 { "My Addresses" }
                        p { "Loading..." }
                        progress {} // An indeterminate progress bar
                    }
                }
            }

            // The async task returned an error.
            Some(Err(e)) => {
                rsx! {
                    Card {
                        h3 { "Error" }
                        p { "Failed to load addresses: {e}" }
                        // You could add a "Retry" button here
                        button {
                            onclick: move |_| known_keys.restart(),
                            "Retry"
                        }
                    }
                }
            }

            // The async task finished successfully.
            Some(Ok(keys)) => {
                let addresses = keys
                .iter()
                .rev()
                .filter_map(|key| key.to_address() )
                .map(|addr| {
                    (
                        AddressableKeyType::from(&addr).to_string(),
                        addr
                            .to_display_bech32m_abbreviated(Network::Main)
                            .unwrap()
                    )
                });

                rsx! {
                    Card {
                        h3 { "My Addresses" }
                        table {
                            // Use the standard `.iter().map()` pattern to iterate and transform the data.
                            {addresses.map(|(ktype, addr_abbrev)| {
                                rsx! {
                                    tr {
                                        td { "{ktype}" }
                                        td { "{addr_abbrev}" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
