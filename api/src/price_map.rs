//! Provides a specialized map for storing NPT prices against various fiat currencies.

use crate::fiat_amount::FiatAmount;
use crate::fiat_currency::FiatCurrency;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

/// A map holding the price of one NPT token in various fiat currencies.
///
/// This struct wraps a `HashMap` to provide a type-safe API for price management,
/// storing only the raw minor-unit amounts for efficiency. It can be iterated
/// over to yield `FiatAmount` instances.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceMap(HashMap<FiatCurrency, i64>);

impl PriceMap {
    /// Creates a new, empty `PriceMap`.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts or updates the price for a given currency.
    ///
    /// This takes a `FiatAmount` and internally stores only its minor-unit value.
    ///
    /// If the map previously contained a price for the given currency, the old
    /// value is returned as a `FiatAmount`.
    pub fn insert(&mut self, price: FiatAmount) -> Option<FiatAmount> {
        let currency = price.currency();
        self.0
            .insert(currency, price.as_minor_units())
            .map(|old_amount| FiatAmount::new_from_minor(old_amount, currency))
    }

    /// **[NEW]** Removes a price from the map for a specific currency, returning the price if it existed.
    ///
    /// Returns `None` if the price for the requested currency was not in the map.
    pub fn remove(&mut self, currency: FiatCurrency) -> Option<FiatAmount> {
        self.0
            .remove(&currency)
            .map(|amount| FiatAmount::new_from_minor(amount, currency))
    }

    /// Retrieves the price for a specific currency as a complete `FiatAmount`.
    ///
    /// Returns `None` if the price for the requested currency is not available.
    pub fn get(&self, currency: FiatCurrency) -> Option<FiatAmount> {
        self.0
            .get(&currency)
            .map(|&amount| FiatAmount::new_from_minor(amount, currency))
    }

    /// Returns an iterator over the prices in the map.
    ///
    /// The iterator yields `FiatAmount` instances.
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }
}

/// An iterator over the `FiatAmount` items in a `PriceMap`.
///
/// This struct is created by the `iter` method on `PriceMap`.
pub struct Iter<'a>(std::collections::hash_map::Iter<'a, FiatCurrency, i64>);

/// Implements the `Iterator` trait for our custom `Iter` struct.
impl<'a> Iterator for Iter<'a> {
    type Item = FiatAmount;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|(currency, &amount)| FiatAmount::new_from_minor(amount, *currency))
    }
}

/// Allows `PriceMap` to be used directly in `for` loops.
impl<'a> IntoIterator for &'a PriceMap {
    type Item = FiatAmount;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
