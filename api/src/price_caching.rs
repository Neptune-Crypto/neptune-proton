//! Handles the caching logic for external price provider data.
#![allow(dead_code)]

use crate::price_map::PriceMap;
use crate::price_providers::{coin_gecko::CoinGecko, PriceProvider};
use dioxus::prelude::ServerFnError;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{OnceCell, RwLock};

#[derive(Clone, Debug)]
struct CachedPrices {
    price_map: PriceMap,
    last_fetched: Instant,
}

/// Retrieves fiat prices, using a lazy, time-based cache.
///
/// This function acts as a gatekeeper to the underlying price provider. It only
/// calls the provider when the cache is empty or older than the defined `CACHE_DURATION`.
pub async fn get_cached_fiat_prices() -> Result<PriceMap, ServerFnError> {
    static CACHE: OnceCell<Arc<RwLock<Option<CachedPrices>>>> = OnceCell::const_new();
    const CACHE_DURATION: Duration = Duration::from_secs(60);

    let cache_lock = CACHE
        .get_or_init(|| async { Arc::new(RwLock::new(None)) })
        .await;

    // Check if a valid, non-stale cache entry exists first with a read lock.
    let read_lock = cache_lock.read().await;
    if let Some(cache) = &*read_lock {
        if cache.last_fetched.elapsed() < CACHE_DURATION {
            return Ok(cache.price_map.clone());
        }
    }
    drop(read_lock); // Release read lock before attempting to acquire a write lock.

    // If the cache was empty or stale, acquire a write lock to update it.
    let mut write_lock = cache_lock.write().await;

    // A crucial double-check: another task might have updated the cache while we were waiting for the write lock.
    if let Some(cache) = &*write_lock {
        if cache.last_fetched.elapsed() < CACHE_DURATION {
            return Ok(cache.price_map.clone());
        }
    }

    // We have the lock and the cache is confirmed to be stale. Fetch new data.
    let provider = CoinGecko;
    let new_price_map = provider.get_prices().await?;

    *write_lock = Some(CachedPrices {
        price_map: new_price_map.clone(),
        last_fetched: Instant::now(),
    });

    Ok(new_price_map)
}
