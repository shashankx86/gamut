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
    ThemeSchemeId, VK_BACKSPACE, VK_DELETE, VK_DOWN, VK_END, VK_ENTER, VK_ESCAPE, VK_F1, VK_HOME,
    VK_INSERT, VK_LEFT, VK_NUMPAD_0, VK_PAGE_DOWN, VK_PAGE_UP, VK_RIGHT, VK_TAB, VK_UP,
    virtual_keycode_from_ascii_char, virtual_keycode_from_token,
};
pub(crate) use normalize::{normalize_identifier, normalize_key_token};
pub(crate) use store::{config_path, load_preferences, save_preferences};
