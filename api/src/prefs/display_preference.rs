use crate::fiat_currency::FiatCurrency;
use crate::price_providers::PriceProviderKind;
use std::env;
use std::str::FromStr;
use serde::Serialize;
use serde::Deserialize;

/// Represents the user's complete currency display preference.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, strum::EnumIs)]
pub enum DisplayPreference {
    /// Pure NPT mode. The app should not fetch or display any fiat info.
    NptOnly,

    /// Fiat integration is enabled.
    FiatEnabled {
        /// The specific fiat currency selected by the user.
        fiat: FiatCurrency,

        /// Determines the default display: `true` for fiat, `false` for NPT.
        display_as_fiat: bool,

        /// The enum variant for the selected price data provider.
        provider: PriceProviderKind,
    },
}

impl DisplayPreference {

    /// Creates a DisplayPreference instance from environment variables,
    /// with a conservative in-code default.
    ///
    /// # Environment Variables (case-insensitive for "true" or "false"):
    /// - `NPT_ONLY`:
    ///   If "true", forces NPT-only mode. If "false", use Fiat mode.
    ///   defaults to false
    /// - `FIAT_CURRENCY`: "USD", "EUR", or "JPY".
    /// - `DISPLAY_AS_FIAT`: "true" to make fiat the default display.
    /// - `PRICE_PROVIDER`: "coingecko" or "coinpaprika".
    pub fn from_env() -> Self {
        /// **Easy toggle:** Set to `true` to make NPT-only the default mode.
        /// This is the lowest priority setting.
        const NPT_ONLY: bool = false;

        let is_npt_mode = match env::var("NPT_ONLY") {
            Ok(val) => val.eq_ignore_ascii_case("true") || val == "1",
            Err(_) => NPT_ONLY, // Fallback to the in-code constant
        };

        if is_npt_mode {
            Self::NptOnly
        } else {
            // Fiat mode is active, now parse the specific settings.
            let fiat = env::var("FIAT_CURRENCY")
                .ok()
                .and_then(|s| FiatCurrency::from_str(&s).ok())
                .unwrap_or_default();

            let display_as_fiat = env::var("DISPLAY_AS_FIAT")
                .map(|val| val.eq_ignore_ascii_case("true") || val == "1")
                .unwrap_or(true);

            let provider = env::var("PRICE_PROVIDER")
                .ok()
                .and_then(|s| PriceProviderKind::from_str(&s).ok())
                .unwrap_or_default();

            Self::FiatEnabled {
                fiat,
                display_as_fiat,
                provider,
            }
        }
    }
}


impl Default for DisplayPreference {
    fn default() -> Self {
        Self::from_env()
    }
}
