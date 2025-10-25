use super::display_preference::DisplayPreference;
use serde::Serialize;
use serde::Deserialize;

/// Represents all user prefs. Intended for saving to a file. editing in settings dialog, etc.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct UserPrefs {
    display_preference: DisplayPreference,
}

impl UserPrefs {
    pub fn display_preference(&self) -> &DisplayPreference {
        &self.display_preference
    }
}
