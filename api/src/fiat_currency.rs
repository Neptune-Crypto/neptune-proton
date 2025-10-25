//! Defines the fiat currencies supported by the application.

use serde::Deserialize;
use serde::Serialize;

/// Represents a fiat currency, containing its code, symbol, and formatting rules.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Default, strum::EnumIs, strum::EnumIter, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
#[allow(clippy::upper_case_acronyms)]
pub enum FiatCurrency {
    AED, // United Arab Emirates Dirham
    ARS, // Argentine Peso
    AUD, // Australian Dollar
    BHD, // Bahraini Dinar
    BMD, // Bermudian Dollar
    BRL, // Brazilian Real
    CAD, // Canadian Dollar
    CHF, // Swiss Franc
    CLP, // Chilean Peso
    CNY, // Chinese Yuan
    CZK, // Czech Koruna
    DKK, // Danish Krone
    EUR, // Euro
    GBP, // Great British Pound
    GEL, // Georgian Lari
    HKD, // Hong Kong Dollar
    HUF, // Hungarian Forint
    IDR, // Indonesian Rupiah
    ILS, // Israeli New Shekel
    INR, // Indian Rupee
    JPY, // Japanese Yen
    KRW, // South Korean Won
    KWD, // Kuwaiti Dinar
    LKR, // Sri Lankan Rupee
    MXN, // Mexican Peso
    MYR, // Malaysian Ringgit
    NGN, // Nigerian Naira
    NOK, // Norwegian Krone
    NZD, // New Zealand Dollar
    PHP, // Philippine Peso
    PKR, // Pakistani Rupee
    PLN, // Polish Złoty
    RON, // Romanian Leu
    SAR, // Saudi Riyal
    SEK, // Swedish Krona
    SGD, // Singapore Dollar
    THB, // Thai Baht
    TRY, // Turkish Lira
    TWD, // New Taiwan Dollar
    UAH, // Ukrainian Hryvnia
    #[default]
    USD, // United States Dollar
    VND, // Vietnamese Đồng
    ZAR, // South African Rand
}

impl FiatCurrency {
    /// Returns the number of decimal digits used by the currency.
    ///
    /// For example, USD uses 2 decimal places (cents), while JPY uses 0.
    /// KWD and BHD use 3 decimal places.
    pub fn decimals(&self) -> u8 {
        match self {
            Self::JPY | Self::KRW | Self::CLP | Self::VND => 0,
            Self::KWD | Self::BHD => 3,
            _ => 2, // Most currencies use 2 decimal places
        }
    }

    /// Returns the graphical symbol for the currency (e.g., '$').
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::AED => "د.إ",
            Self::ARS => "$",
            Self::AUD => "$",
            Self::BHD => ".د.ب",
            Self::BMD => "$",
            Self::BRL => "R$",
            Self::CAD => "$",
            Self::CHF => "CHF",
            Self::CLP => "$",
            Self::CNY => "¥",
            Self::CZK => "Kč",
            Self::DKK => "kr",
            Self::EUR => "€",
            Self::GBP => "£",
            Self::GEL => "₾",
            Self::HKD => "$",
            Self::HUF => "Ft",
            Self::IDR => "Rp",
            Self::ILS => "₪",
            Self::INR => "₹",
            Self::JPY => "¥",
            Self::KRW => "₩",
            Self::KWD => "د.ك",
            Self::LKR => "Rs",
            Self::MXN => "$",
            Self::MYR => "RM",
            Self::NGN => "₦",
            Self::NOK => "kr",
            Self::NZD => "$",
            Self::PHP => "₱",
            Self::PKR => "₨",
            Self::PLN => "zł",
            Self::RON => "lei",
            Self::SAR => "﷼",
            Self::SEK => "kr",
            Self::SGD => "$",
            Self::THB => "฿",
            Self::TRY => "₺",
            Self::TWD => "NT$",
            Self::UAH => "₴",
            Self::USD => "$",
            Self::VND => "₫", // Note: Switched from Rp to the correct đồng symbol
            Self::ZAR => "R",
        }
    }

    /// Returns the ISO 4217 string code for the currency (e.g., "USD").
    /// This is handled automatically by the `strum::IntoStaticStr` derive macro.
    pub fn code(&self) -> &'static str {
        self.into()
    }

    /// Returns the full name of the currency.
    pub fn name(&self) -> &'static str {
        match self {
            Self::AED => "United Arab Emirates Dirham",
            Self::ARS => "Argentine Peso",
            Self::AUD => "Australian Dollar",
            Self::BHD => "Bahraini Dinar",
            Self::BMD => "Bermudian Dollar",
            Self::BRL => "Brazilian Real",
            Self::CAD => "Canadian Dollar",
            Self::CHF => "Swiss Franc",
            Self::CLP => "Chilean Peso",
            Self::CNY => "Chinese Yuan",
            Self::CZK => "Czech Koruna",
            Self::DKK => "Danish Krone",
            Self::EUR => "Euro",
            Self::GBP => "Great British Pound",
            Self::GEL => "Georgian Lari",
            Self::HKD => "Hong Kong Dollar",
            Self::HUF => "Hungarian Forint",
            Self::IDR => "Indonesian Rupiah",
            Self::ILS => "Israeli New Shekel",
            Self::INR => "Indian Rupee",
            Self::JPY => "Japanese Yen",
            Self::KRW => "South Korean Won",
            Self::KWD => "Kuwaiti Dinar",
            Self::LKR => "Sri Lankan Rupee",
            Self::MXN => "Mexican Peso",
            Self::MYR => "Malaysian Ringgit",
            Self::NGN => "Nigerian Naira",
            Self::NOK => "Norwegian Krone",
            Self::NZD => "New Zealand Dollar",
            Self::PHP => "Philippine Peso",
            Self::PKR => "Pakistani Rupee",
            Self::PLN => "Polish Złoty",
            Self::RON => "Romanian Leu",
            Self::SAR => "Saudi Riyal",
            Self::SEK => "Swedish Krona",
            Self::SGD => "Singapore Dollar",
            Self::THB => "Thai Baht",
            Self::TRY => "Turkish Lira",
            Self::TWD => "New Taiwan Dollar",
            Self::UAH => "Ukrainian Hryvnia",
            Self::USD => "United States Dollar",
            Self::VND => "Vietnamese Đồng",
            Self::ZAR => "South African Rand",
        }
    }

    pub fn format_amount(&self, amt: &str) -> String {
        format!("{} {}", amt, self.code())
    }
}