mod model;
mod store;

pub(crate) use model::{
    AppPreferences, AppearancePreferences, CustomThemeColors, LauncherPlacement, LauncherSize,
    LayoutPreferences, RadiusPreference, ShortcutBinding, ShortcutPreferences, ThemePreference,
};
pub(crate) use store::{load_preferences, preferences_path, save_preferences};
