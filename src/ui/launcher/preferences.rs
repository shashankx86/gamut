use crate::core::preferences::{AppPreferences, ShortcutBinding, ShortcutPreferences};
use std::str::FromStr;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ui) enum ThemeColorField {
    Background,
    Text,
    Accent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ui) enum ShortcutAction {
    LaunchSelected,
    ExpandOrMoveDown,
    MoveUp,
    CloseLauncher,
}

impl ShortcutAction {
    pub(in crate::ui) const ALL: [Self; 4] = [
        Self::LaunchSelected,
        Self::ExpandOrMoveDown,
        Self::MoveUp,
        Self::CloseLauncher,
    ];

    pub(in crate::ui) fn label(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Launch selected result",
            Self::ExpandOrMoveDown => "Expand or move down",
            Self::MoveUp => "Move up",
            Self::CloseLauncher => "Close launcher",
        }
    }

    pub(in crate::ui) fn helper_text(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Runs the highlighted application or command.",
            Self::ExpandOrMoveDown => "Expands the empty launcher or moves selection down.",
            Self::MoveUp => "Moves the highlighted selection upward.",
            Self::CloseLauncher => "Hides the launcher window.",
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::ui) struct PreferencesEditor {
    custom_background: String,
    custom_text: String,
    custom_accent: String,
    launch_selected: String,
    expand_or_move_down: String,
    move_up: String,
    close_launcher: String,
    theme_error: Option<String>,
    shortcut_error: Option<String>,
    save_error: Option<String>,
}

impl PreferencesEditor {
    pub(in crate::ui) fn from_preferences(preferences: &AppPreferences) -> Self {
        Self {
            custom_background: preferences.appearance.custom_theme.background.clone(),
            custom_text: preferences.appearance.custom_theme.text.clone(),
            custom_accent: preferences.appearance.custom_theme.accent.clone(),
            launch_selected: preferences.shortcuts.launch_selected.to_string(),
            expand_or_move_down: preferences.shortcuts.expand_or_move_down.to_string(),
            move_up: preferences.shortcuts.move_up.to_string(),
            close_launcher: preferences.shortcuts.close_launcher.to_string(),
            theme_error: None,
            shortcut_error: None,
            save_error: None,
        }
    }

    pub(in crate::ui) fn sync_from_preferences(&mut self, preferences: &AppPreferences) {
        *self = Self::from_preferences(preferences);
    }

    pub(in crate::ui) fn theme_value(&self, field: ThemeColorField) -> &str {
        match field {
            ThemeColorField::Background => &self.custom_background,
            ThemeColorField::Text => &self.custom_text,
            ThemeColorField::Accent => &self.custom_accent,
        }
    }

    pub(in crate::ui) fn set_theme_value(&mut self, field: ThemeColorField, value: String) {
        match field {
            ThemeColorField::Background => self.custom_background = value,
            ThemeColorField::Text => self.custom_text = value,
            ThemeColorField::Accent => self.custom_accent = value,
        }
    }

    pub(in crate::ui) fn shortcut_value(&self, action: ShortcutAction) -> &str {
        match action {
            ShortcutAction::LaunchSelected => &self.launch_selected,
            ShortcutAction::ExpandOrMoveDown => &self.expand_or_move_down,
            ShortcutAction::MoveUp => &self.move_up,
            ShortcutAction::CloseLauncher => &self.close_launcher,
        }
    }

    pub(in crate::ui) fn set_shortcut_value(&mut self, action: ShortcutAction, value: String) {
        match action {
            ShortcutAction::LaunchSelected => self.launch_selected = value,
            ShortcutAction::ExpandOrMoveDown => self.expand_or_move_down = value,
            ShortcutAction::MoveUp => self.move_up = value,
            ShortcutAction::CloseLauncher => self.close_launcher = value,
        }
    }

    pub(in crate::ui) fn theme_error(&self) -> Option<&str> {
        self.theme_error.as_deref()
    }

    pub(in crate::ui) fn set_theme_error(&mut self, error: Option<String>) {
        self.theme_error = error;
    }

    pub(in crate::ui) fn shortcut_error(&self) -> Option<&str> {
        self.shortcut_error.as_deref()
    }

    pub(in crate::ui) fn set_shortcut_error(&mut self, error: Option<String>) {
        self.shortcut_error = error;
    }

    pub(in crate::ui) fn save_error(&self) -> Option<&str> {
        self.save_error.as_deref()
    }

    pub(in crate::ui) fn set_save_error(&mut self, error: Option<String>) {
        self.save_error = error;
    }
}

pub(in crate::ui) fn normalize_hex_color(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('#');

    match trimmed.len() {
        6 | 8 if trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) => {
            Some(format!("#{}", trimmed.to_ascii_uppercase()))
        }
        _ => None,
    }
}

pub(in crate::ui) fn shortcut_preferences_from_editor(
    editor: &PreferencesEditor,
) -> Result<ShortcutPreferences, String> {
    let shortcuts = ShortcutPreferences {
        launch_selected: parse_shortcut(editor.shortcut_value(ShortcutAction::LaunchSelected))?,
        expand_or_move_down: parse_shortcut(
            editor.shortcut_value(ShortcutAction::ExpandOrMoveDown),
        )?,
        move_up: parse_shortcut(editor.shortcut_value(ShortcutAction::MoveUp))?,
        close_launcher: parse_shortcut(editor.shortcut_value(ShortcutAction::CloseLauncher))?,
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
        PreferencesEditor, ShortcutAction, normalize_hex_color, shortcut_preferences_from_editor,
    };
    use crate::core::preferences::AppPreferences;

    #[test]
    fn hex_colors_are_normalized_with_hash() {
        assert_eq!(normalize_hex_color("a1b2c3"), Some("#A1B2C3".to_string()));
        assert_eq!(
            normalize_hex_color("#abcdef12"),
            Some("#ABCDEF12".to_string())
        );
        assert_eq!(normalize_hex_color("invalid"), None);
    }

    #[test]
    fn duplicate_shortcuts_are_rejected() {
        let preferences = AppPreferences::default();
        let mut editor = PreferencesEditor::from_preferences(&preferences);
        editor.set_shortcut_value(ShortcutAction::MoveUp, "ArrowDown".to_string());

        let error = shortcut_preferences_from_editor(&editor)
            .expect_err("duplicate bindings should be rejected");

        assert!(error.contains("Move up"));
    }
}
