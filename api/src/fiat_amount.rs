//! Provides a safe, self-contained type for representing fiat currency amounts.

use std::fmt;
use std::ops::Add;
use std::ops::AddAssign;

use num_traits::CheckedAdd;
use thiserror::Error;

use crate::fiat_currency::FiatCurrency;

/// An error that can occur when parsing a string into a `FiatAmount`.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseFiatAmountError {
    /// The string is not in a valid numeric format (e.g., "abc", "1.2.3").
    #[error("invalid fiat amount format")]
    InvalidFormat,
    /// The string has more decimal places than the currency supports (e.g., "$1.234").
    #[error("too many decimal places for the currency")]
    TooManyDecimals,
}

/// Represents a monetary value in a specific fiat currency.
///
/// Internally, the amount is stored as a signed 64-bit integer in the currency's
/// smallest unit (e.g., cents for USD) to prevent floating-point inaccuracies.
/// The default `Display` implementation formats this as a plain numeric string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiatAmount {
    amount: i64,
    currency: FiatCurrency,
}

impl FiatAmount {
    // --- Getters ---

    /// Returns the currency type of the amount.
    pub fn currency(&self) -> FiatCurrency {
        self.currency
    }

    /// Returns the raw amount in the currency's smallest unit (e.g., cents).
    pub fn as_minor_units(&self) -> i64 {
        self.amount
    }

    // --- Constructors ---

    /// Creates a new `FiatAmount` from a floating-point value, typically from an API.
    ///
    /// The float is safely converted to an integer representation by rounding to the
    /// nearest minor unit based on the currency's specified number of decimal places.
    ///
    /// # Examples
    /// ```
    /// let amount = FiatAmount::new_from_float(123.456, FiatCurrency::USD);
    /// assert_eq!(amount.as_minor_units(), 12346); // Rounded to 12346 cents
    /// ```
    pub fn new_from_float(value: f64, currency: FiatCurrency) -> Self {
        let decimals = currency.decimals();
        let multiplier = 10_f64.powi(decimals as i32);
        let amount = (value * multiplier).round() as i64;

        Self { amount, currency }
    }

    /// Creates a new `FiatAmount` directly from its smallest unit.
    ///
    /// # Example
    /// ```
    /// // 12345 cents represents $123.45
    /// let amount = FiatAmount::new_from_minor(12345, FiatCurrency::USD);
    /// assert_eq!(amount.to_string(), "123.45");
    /// ```
    pub fn new_from_minor(amount: i64, currency: FiatCurrency) -> Self {
        Self { amount, currency }
    }

    /// Creates a new `FiatAmount` by parsing a string representation.
    ///
    /// This is a fallible operation that returns an error if the string is not a
    /// valid number or has too many decimal places for the given currency.
    ///
    /// # Examples
    /// ```
    /// // Successful parsing
    /// let amount = FiatAmount::new_from_str("123.45", FiatCurrency::USD)?;
    /// assert_eq!(amount.as_minor_units(), 12345);
    ///
    /// // Error on too many decimals
    /// let err = FiatAmount::new_from_str("1.234", FiatCurrency::USD).unwrap_err();
    /// assert_eq!(err, ParseFiatAmountError::TooManyDecimals);
    /// ```
    pub fn new_from_str(s: &str, currency: FiatCurrency) -> Result<Self, ParseFiatAmountError> {
        let decimals = currency.decimals() as u32;

        let (is_negative, s) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };

        let mut parts = s.split('.');
        let major_str = parts.next().unwrap_or("");
        let minor_str = parts.next().unwrap_or("");

        if parts.next().is_some() || (major_str.is_empty() && minor_str.is_empty()) {
            return Err(ParseFiatAmountError::InvalidFormat);
        }

        if minor_str.len() > decimals as usize {
            return Err(ParseFiatAmountError::TooManyDecimals);
        }

        let major_units = if major_str.is_empty() {
            0
        } else {
            major_str
                .parse::<i64>()
                .map_err(|_| ParseFiatAmountError::InvalidFormat)?
        };

        let minor_units = if minor_str.is_empty() {
            0
        } else {
            minor_str
                .parse::<i64>()
                .map_err(|_| ParseFiatAmountError::InvalidFormat)?
        };

        let scaling_factor = 10_i64.pow(decimals - minor_str.len() as u32);
        let scaled_minor_units = minor_units
            .checked_mul(scaling_factor)
            .ok_or(ParseFiatAmountError::InvalidFormat)?;

        let multiplier = 10_i64.pow(decimals);
        let mut total_minor_units = major_units
            .checked_mul(multiplier)
            .ok_or(ParseFiatAmountError::InvalidFormat)?
            .checked_add(scaled_minor_units)
            .ok_or(ParseFiatAmountError::InvalidFormat)?;

        if is_negative {
            total_minor_units = -total_minor_units;
        }

        Ok(Self::new_from_minor(total_minor_units, currency))
    }

    // --- Display Methods ---

    /// Formats the amount with its currency symbol (e.g., "$25.34").
    pub fn to_string_with_symbol(&self) -> String {
        format!("{}{}", self.currency.symbol(), self)
    }

    /// Formats the amount with its currency code (e.g., "25.34 USD").
    pub fn to_string_with_code(&self) -> String {
        format!("{} {}", self, self.currency.code())
    }
}

/// Implements the default `Display` trait to format the amount as a numeric string (e.g., "25.34").
///
/// This allows `.to_string()` to be called on a `FiatAmount` for its base numeric representation.
impl fmt::Display for FiatAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimals = self.currency.decimals() as usize;

        if decimals == 0 {
            return write!(f, "{}", self.amount);
        }

        let divisor = 10_i64.pow(decimals as u32);
        let major_units = self.amount / divisor;
        let minor_units = self.amount.abs() % divisor;

        write!(
            f,
            "{}.{:0width$}",
            major_units,
            minor_units,
            width = decimals
        )
    }
}

/// Implements the addition operator. Panics if currencies do not match.
impl Add for FiatAmount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            panic!(
                "Cannot add amounts of different currencies: {:?} and {:?}",
                self.currency, rhs.currency
            );
        }
        Self {
            amount: self.amount + rhs.amount,
            currency: self.currency,
        }
    }
}

/// Implements the addition assignment operator. Panics if currencies do not match.
impl AddAssign for FiatAmount {
    fn add_assign(&mut self, rhs: Self) {
        if self.currency != rhs.currency {
            panic!(
                "Cannot add amounts of different currencies: {:?} and {:?}",
                self.currency, rhs.currency
            );
        }
        self.amount += rhs.amount;
    }
}

/// Implements checked addition. Returns `None` if currencies mismatch or if addition overflows.
impl CheckedAdd for FiatAmount {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        if self.currency != v.currency {
            return None; // Mismatched currencies
        }
        self.amount.checked_add(v.amount).map(|new_amount| Self {
            amount: new_amount,
            currency: self.currency,
        })
    }
}
