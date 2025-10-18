//! Defines the fiat currencies supported by the application.

use serde::Deserialize;
use serde::Serialize;

/// Represents a fiat currency, containing its code, symbol, and formatting rules.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum FiatCurrency {
    USD,
    EUR,
    JPY,
}

impl FiatCurrency {
    /// Returns the number of decimal digits used by the currency.
    ///
    /// For example, USD uses 2 decimal places (cents), while JPY uses 0.
    pub fn decimals(&self) -> u8 {
        match self {
            Self::USD => 2,
            Self::EUR => 2,
            Self::JPY => 0,
        }
    }

    /// Returns the graphical symbol for the currency (e.g., '$').
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::USD => "$",
            Self::EUR => "€",
            Self::JPY => "¥",
        }
    }

    /// Returns the ISO 4217 string code for the currency (e.g., "USD").
    pub fn code(&self) -> &'static str {
        match self {
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::JPY => "JPY",
        }
    }
}