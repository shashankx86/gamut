mod config;
mod model;
mod normalize;
mod store;

pub(crate) use config::{
    ConfigCommand, ConfigKey, ConfigResetTarget, ShortcutConfigCommand, execute as run_config,
};
pub(crate) use model::normalize_hex_color;
pub(crate) use model::{
    AppPreferences, AppearancePreferences, LauncherPlacement, LauncherSize, RadiusPreference,
    ShortcutAction, ShortcutBinding, ShortcutPreferences, ThemeColors, ThemePreference,
    ThemeSchemeId,
};
pub(crate) use normalize::{normalize_identifier, normalize_key_token};
pub(crate) use store::{config_path, load_preferences, save_preferences};
