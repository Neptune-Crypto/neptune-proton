//! Defines traits and implementations for external price data providers.

use crate::fiat_amount::FiatAmount;
use crate::fiat_currency::FiatCurrency;
use crate::price_map::PriceMap;
use dioxus::prelude::ServerFnError;
use serde::Deserialize;
use std::collections::HashMap;

/// A trait for any service that can provide fiat prices for NPT.
pub trait PriceProvider {
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

    impl PriceProvider for CoinGecko {
        async fn get_prices(&self) -> Result<PriceMap, ServerFnError> {
            const URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=neptune-cash&vs_currencies=usd,eur,jpy";

            let client = reqwest::Client::new();
            let resp = client
                .get(URL)
                .send()
                .await?
                .json::<CoinGeckoResponse>()
                .await?;

            let mut price_map = PriceMap::new();
            if let Some(price) = resp.neptune_cash.get("usd") {
                price_map.insert(FiatAmount::new_from_float(*price, FiatCurrency::USD));
            }
            if let Some(price) = resp.neptune_cash.get("eur") {
                price_map.insert(FiatAmount::new_from_float(*price, FiatCurrency::EUR));
            }
            if let Some(price) = resp.neptune_cash.get("jpy") {
                price_map.insert(FiatAmount::new_from_float(*price, FiatCurrency::JPY));
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

    impl PriceProvider for CoinPaprika {
        async fn get_prices(&self) -> Result<PriceMap, ServerFnError> {
            // CoinPaprika Tickers endpoint is used with a 'quotes' parameter for fiat conversion.
            const URL: &str = "https://api.coinpaprika.com/v1/tickers/npt-neptune-cash?quotes=EUR,USD,JPY";

            let client = reqwest::Client::new();

            // 1. Fetch the data and parse it into a generic serde_json::Value
            let resp: Value = client
                .get(URL)
                .send()
                .await?
                .json::<Value>()
                .await?;

            let mut price_map = PriceMap::new();

            // 2. Helper function to extract the price for a given currency
            let get_price = |currency_code: &str| -> Option<f64> {
                // The structure for price is:
                // resp["quotes"][currency_code]["price"]
                resp.get("quotes")?
                    .get(currency_code)?
                    .get("price")?
                    .as_f64()
            };

            // 3. Extract and insert prices into the PriceMap
            if let Some(price) = get_price("USD") {
                price_map.insert(FiatAmount::new_from_float(price, FiatCurrency::USD));
            }
            if let Some(price) = get_price("EUR") {
                price_map.insert(FiatAmount::new_from_float(price, FiatCurrency::EUR));
            }
            if let Some(price) = get_price("JPY") {
                price_map.insert(FiatAmount::new_from_float(price, FiatCurrency::JPY));
            }

            Ok(price_map)
        }
    }
}