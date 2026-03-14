use crate::core::preferences::{
    normalize_hex_color, AppPreferences, ShortcutBinding, ShortcutPreferences, ThemeSchemeId,
};
use crate::core::theme::LIGHT_SEED;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColorField {
    Background,
    Text,
    Accent,
}

impl ThemeColorField {
    pub const ALL: [Self; 3] = [Self::Background, Self::Text, Self::Accent];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Background => "Background color",
            Self::Text => "Text color",
            Self::Accent => "Accent color",
        }
    }

    pub const fn placeholder(self) -> &'static str {
        match self {
            Self::Background => LIGHT_SEED.background,
            Self::Text => LIGHT_SEED.text,
            Self::Accent => LIGHT_SEED.accent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeEditorState {
    light_background: String,
    light_text: String,
    light_accent: String,
    dark_background: String,
    dark_text: String,
    dark_accent: String,
    theme_error: Option<String>,
}

impl ThemeEditorState {
    pub fn from_preferences(preferences: &AppPreferences) -> Self {
        Self {
            light_background: preferences.appearance.schemes.light.background.clone(),
            light_text: preferences.appearance.schemes.light.text.clone(),
            light_accent: preferences.appearance.schemes.light.accent.clone(),
            dark_background: preferences.appearance.schemes.dark.background.clone(),
            dark_text: preferences.appearance.schemes.dark.text.clone(),
            dark_accent: preferences.appearance.schemes.dark.accent.clone(),
            theme_error: None,
        }
    }

    pub fn theme_value(&self, scheme: ThemeSchemeId, field: ThemeColorField) -> &str {
        match (scheme, field) {
            (ThemeSchemeId::Light, ThemeColorField::Background) => &self.light_background,
            (ThemeSchemeId::Light, ThemeColorField::Text) => &self.light_text,
            (ThemeSchemeId::Light, ThemeColorField::Accent) => &self.light_accent,
            (ThemeSchemeId::Dark, ThemeColorField::Background) => &self.dark_background,
            (ThemeSchemeId::Dark, ThemeColorField::Text) => &self.dark_text,
            (ThemeSchemeId::Dark, ThemeColorField::Accent) => &self.dark_accent,
        }
    }

    pub fn set_theme_value(
        &mut self,
        scheme: ThemeSchemeId,
        field: ThemeColorField,
        value: String,
    ) {
        match (scheme, field) {
            (ThemeSchemeId::Light, ThemeColorField::Background) => self.light_background = value,
            (ThemeSchemeId::Light, ThemeColorField::Text) => self.light_text = value,
            (ThemeSchemeId::Light, ThemeColorField::Accent) => self.light_accent = value,
            (ThemeSchemeId::Dark, ThemeColorField::Background) => self.dark_background = value,
            (ThemeSchemeId::Dark, ThemeColorField::Text) => self.dark_text = value,
            (ThemeSchemeId::Dark, ThemeColorField::Accent) => self.dark_accent = value,
        }
    }

    pub fn theme_error(&self) -> Option<&str> {
        self.theme_error.as_deref()
    }

    pub fn set_theme_error(&mut self, error: Option<String>) {
        self.theme_error = error;
    }

    pub fn apply_preferences(&mut self, preferences: &AppPreferences) {
        self.light_background = preferences.appearance.schemes.light.background.clone();
        self.light_text = preferences.appearance.schemes.light.text.clone();
        self.light_accent = preferences.appearance.schemes.light.accent.clone();
        self.dark_background = preferences.appearance.schemes.dark.background.clone();
        self.dark_text = preferences.appearance.schemes.dark.text.clone();
        self.dark_accent = preferences.appearance.schemes.dark.accent.clone();
    }
}

pub fn update_theme_scheme_color(
    preferences: &mut AppPreferences,
    editor: &mut ThemeEditorState,
    scheme: ThemeSchemeId,
    field: ThemeColorField,
    value: String,
) -> Result<(), String> {
    editor.set_theme_value(scheme, field, value.clone());

    let Some(normalized) = normalize_hex_color(&value) else {
        return Err(
            "Theme colors must use 6 or 8 digit hexadecimal values like #151516 or #151516FF."
                .to_string(),
        );
    };

    let colors = preferences.appearance.scheme_mut(scheme);
    match field {
        ThemeColorField::Background => colors.background = normalized,
        ThemeColorField::Text => colors.text = normalized,
        ThemeColorField::Accent => colors.accent = normalized,
    }

    editor.apply_preferences(preferences);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    LaunchSelected,
    ExpandOrMoveDown,
    MoveUp,
    CloseLauncher,
}

impl ShortcutAction {
    pub const ALL: [Self; 4] = [
        Self::LaunchSelected,
        Self::ExpandOrMoveDown,
        Self::MoveUp,
        Self::CloseLauncher,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Launch selected result",
            Self::ExpandOrMoveDown => "Expand or move down",
            Self::MoveUp => "Move up",
            Self::CloseLauncher => "Close launcher",
        }
    }

    pub fn helper_text(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Runs the highlighted application or command.",
            Self::ExpandOrMoveDown => "Expands the empty launcher or moves selection down.",
            Self::MoveUp => "Moves the highlighted selection upward.",
            Self::CloseLauncher => "Hides the launcher window.",
        }
    }
}

pub fn shortcut_preferences_from_values(
    values: &[(ShortcutAction, String)],
) -> Result<ShortcutPreferences, String> {
    let value_for = |action| {
        values
            .iter()
            .find(|(candidate, _)| *candidate == action)
            .map(|(_, value)| value.as_str())
            .unwrap_or("")
    };

    let shortcuts = ShortcutPreferences {
        launch_selected: parse_shortcut(value_for(ShortcutAction::LaunchSelected))?,
        expand_or_move_down: parse_shortcut(value_for(ShortcutAction::ExpandOrMoveDown))?,
        move_up: parse_shortcut(value_for(ShortcutAction::MoveUp))?,
        close_launcher: parse_shortcut(value_for(ShortcutAction::CloseLauncher))?,
    };

    let bindings = [
        (ShortcutAction::LaunchSelected, &shortcuts.launch_selected),
        (
            ShortcutAction::ExpandOrMoveDown,
            &shortcuts.expand_or_move_down,
        ),
        (ShortcutAction::MoveUp, &shortcuts.move_up),
        (ShortcutAction::CloseLauncher, &shortcuts.close_launcher),
    ];

    for (index, (left_action, left_binding)) in bindings.iter().enumerate() {
        for (right_action, right_binding) in bindings.iter().skip(index + 1) {
            if same_shortcut(left_binding, right_binding) {
                return Err(format!(
                    "{} conflicts with {}.",
                    (*left_action).label(),
                    (*right_action).label()
                ));
            }
        }
    }

    Ok(shortcuts)
}

fn parse_shortcut(value: &str) -> Result<ShortcutBinding, String> {
    ShortcutBinding::from_str(value).map_err(|error| format!("Invalid shortcut `{value}`: {error}"))
}

fn same_shortcut(left: &ShortcutBinding, right: &ShortcutBinding) -> bool {
    left.ctrl == right.ctrl
        && left.alt == right.alt
        && left.shift == right.shift
        && left.super_key == right.super_key
        && left.normalized_key() == right.normalized_key()
}

#[cfg(test)]
mod tests {
    use super::{
        shortcut_preferences_from_values, update_theme_scheme_color, ShortcutAction,
        ThemeColorField, ThemeEditorState,
    };
    use crate::core::preferences::{AppPreferences, ThemeSchemeId};

    #[test]
    fn duplicate_shortcuts_are_rejected() {
        let error = shortcut_preferences_from_values(&[
            (ShortcutAction::LaunchSelected, "Enter".to_string()),
            (ShortcutAction::ExpandOrMoveDown, "ArrowDown".to_string()),
            (ShortcutAction::MoveUp, "ArrowDown".to_string()),
            (ShortcutAction::CloseLauncher, "Escape".to_string()),
        ])
        .expect_err("duplicate bindings should be rejected");

        assert!(error.contains("Move up"));
    }

    #[test]
    fn theme_updates_target_selected_scheme() {
        let mut preferences = AppPreferences::default();
        let mut editor = ThemeEditorState::from_preferences(&preferences);

        update_theme_scheme_color(
            &mut preferences,
            &mut editor,
            ThemeSchemeId::Light,
            ThemeColorField::Accent,
            "#123abc".to_string(),
        )
        .expect("light scheme update should succeed");

        assert_eq!(preferences.appearance.schemes.light.accent, "#123ABC");
        assert_eq!(preferences.appearance.schemes.dark.accent, "#5E8BFF");
    }
}
