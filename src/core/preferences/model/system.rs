use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemPreferences {
    pub start_at_login: bool,
}

impl Default for SystemPreferences {
    fn default() -> Self {
        Self {
            start_at_login: false,
        }
    }
}
