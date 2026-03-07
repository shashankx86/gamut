mod model;
mod store;

pub(crate) use model::{
    AppPreferences, AppearancePreferences, CustomThemeColors, LauncherPlacement, LauncherSize,
    RadiusPreference, ShortcutBinding, ShortcutPreferences, ThemePreference,
};
pub(crate) use store::{load_preferences, save_preferences};
