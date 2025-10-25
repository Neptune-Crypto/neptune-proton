//! Defines traits and implementations for external price data providers.

use crate::fiat_amount::FiatAmount;
use crate::fiat_currency::FiatCurrency;
use crate::price_map::PriceMap;
use dioxus::prelude::ServerFnError;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use strum::IntoEnumIterator;

// The trait for provider metadata
pub trait PriceProviderMeta {
    fn name(&self) -> &'static str;
    fn website(&self) -> &'static str;
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, strum::EnumIs, strum::EnumIter, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum PriceProviderKind {
    #[default]
    CoinGecko,
    CoinPaprika,
}

// Implement the METADATA trait for the enum by dispatching to the real structs.
impl PriceProviderMeta for PriceProviderKind {
    fn name(&self) -> &'static str {
        match self {
            Self::CoinGecko => coin_gecko::CoinGecko.name(),
            Self::CoinPaprika => coin_paprika::CoinPaprika.name(),
        }
    }

    fn website(&self) -> &'static str {
        match self {
            Self::CoinGecko => coin_gecko::CoinGecko.website(),
            Self::CoinPaprika => coin_paprika::CoinPaprika.website(),
        }
    }
}

impl PriceProvider for PriceProviderKind {
    async fn get_prices(&self) -> Result<PriceMap, ServerFnError> {
        match self {
            Self::CoinGecko => coin_gecko::CoinGecko.get_prices().await,
            Self::CoinPaprika => coin_paprika::CoinPaprika.get_prices().await,
        }
    }
}

/// A trait for any service that can provide fiat prices for NPT.
pub(crate) trait PriceProvider: PriceProviderMeta {
    /// Fetches the latest price map.
    async fn get_prices(&self) -> Result<PriceMap, ServerFnError>;
}

/// Provides price data from the public CoinGecko API.
pub mod coin_gecko {
    use super::*;

    /// The structure of the JSON response from CoinGecko's simple price API.
    #[derive(Deserialize, Debug)]
    struct CoinGeckoResponse {
        #[serde(rename = "neptune-cash")]
        neptune_cash: HashMap<String, f64>,
    }

    /// An implementation of the `PriceProvider` trait for CoinGecko.
    pub struct CoinGecko;

    impl PriceProviderMeta for CoinGecko {
        fn name(&self) -> &'static str {
            "CoinGecko"
        }

        fn website(&self) -> &'static str {
            "coingecko.com"
        }
    }

    impl PriceProvider for CoinGecko {
        async fn get_prices(&self) -> Result<PriceMap, ServerFnError> {
            // 1. Build the comma-separated list of currency codes from the enum.
            let currency_codes = FiatCurrency::iter()
                .map(|c| c.code().to_lowercase())
                .collect::<Vec<_>>()
                .join(",");

            // 2. Construct the full URL dynamically.
            let url = format!(
                "https://api.coingecko.com/api/v3/simple/price?ids=neptune-cash&vs_currencies={}",
                currency_codes
            );

            let client = reqwest::Client::new();
            let resp = client.get(&url).send().await?.json::<CoinGeckoResponse>().await?;

            let mut price_map = PriceMap::new();

            // 3. Iterate over all supported currencies and populate the map from the response.
            for currency in FiatCurrency::iter() {
                let code_lower = currency.code().to_lowercase();
                if let Some(price) = resp.neptune_cash.get(&code_lower) {
                    price_map.insert(FiatAmount::new_from_float(*price, currency));
                }
            }

            Ok(price_map)
        }
    }
}

/// Provides price data from the CoinPaprika API.
pub mod coin_paprika {
    use super::*;
    use serde_json::Value;

    /// An implementation of the `PriceProvider` trait for CoinPaprika.
    #[allow(dead_code)]
    pub struct CoinPaprika;

    impl PriceProviderMeta for CoinPaprika {
        fn name(&self) -> &'static str {
            "CoinPaprika"
        }

        fn website(&self) -> &'static str {
            "coinpaprika.com"
        }
    }

    impl PriceProvider for CoinPaprika {
        async fn get_prices(&self) -> Result<PriceMap, ServerFnError> {
            // 1. Build the comma-separated list of currency codes from the enum.
            let currency_codes = FiatCurrency::iter().map(|c| c.code()).collect::<Vec<_>>().join(",");

            // 2. Construct the full URL dynamically.
            let url = format!(
                "https://api.coinpaprika.com/v1/tickers/npt-neptune-cash?quotes={}",
                currency_codes
            );

            let client = reqwest::Client::new();

            // Fetch the data and parse it into a generic serde_json::Value
            let resp: Value = client.get(&url).send().await?.json::<Value>().await?;

            let mut price_map = PriceMap::new();

            // Helper closure to extract the price for a given currency code.
            let get_price = |currency_code: &str| -> Option<f64> {
                resp.get("quotes")?.get(currency_code)?.get("price")?.as_f64()
            };

            // 3. Iterate over all supported currencies and populate the map from the response.
            for currency in FiatCurrency::iter() {
                if let Some(price) = get_price(currency.code()) {
                    price_map.insert(FiatAmount::new_from_float(price, currency));
                }
            }

            Ok(price_map)
        }
    }
}