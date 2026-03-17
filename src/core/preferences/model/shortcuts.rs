use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShortcutPreferences {
    pub launch_selected: ShortcutBinding,
    pub expand: ShortcutBinding,
    pub move_down: ShortcutBinding,
    pub move_up: ShortcutBinding,
    pub close_launcher: ShortcutBinding,
}

impl<'de> Deserialize<'de> for ShortcutPreferences {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(default)]
        struct ShortcutPreferencesWire {
            launch_selected: ShortcutBinding,
            expand: Option<ShortcutBinding>,
            move_down: Option<ShortcutBinding>,
            #[serde(alias = "expand_or_move_down")]
            legacy_expand_or_move_down: Option<ShortcutBinding>,
            move_up: ShortcutBinding,
            close_launcher: ShortcutBinding,
        }

        impl Default for ShortcutPreferencesWire {
            fn default() -> Self {
                let defaults = ShortcutPreferences::default();

                Self {
                    launch_selected: defaults.launch_selected,
                    expand: Some(defaults.expand),
                    move_down: Some(defaults.move_down),
                    legacy_expand_or_move_down: None,
                    move_up: defaults.move_up,
                    close_launcher: defaults.close_launcher,
                }
            }
        }

        let wire = ShortcutPreferencesWire::deserialize(deserializer)?;
        let legacy = wire.legacy_expand_or_move_down.clone();
        let defaults = ShortcutPreferences::default();

        Ok(Self {
            launch_selected: wire.launch_selected,
            expand: wire
                .expand
                .or_else(|| legacy.clone())
                .unwrap_or(defaults.expand),
            move_down: wire.move_down.or(legacy).unwrap_or(defaults.move_down),
            move_up: wire.move_up,
            close_launcher: wire.close_launcher,
        })
    }
}

impl Default for ShortcutPreferences {
    fn default() -> Self {
        Self {
            launch_selected: ShortcutBinding::named("Enter"),
            expand: ShortcutBinding::named("ArrowDown"),
            move_down: ShortcutBinding::named("ArrowDown"),
            move_up: ShortcutBinding::named("ArrowUp"),
            close_launcher: ShortcutBinding::named("Escape"),
        }
    }
}

impl ShortcutPreferences {
    pub fn binding(&self, action: ShortcutAction) -> &ShortcutBinding {
        match action {
            ShortcutAction::LaunchSelected => &self.launch_selected,
            ShortcutAction::Expand => &self.expand,
            ShortcutAction::MoveDown => &self.move_down,
            ShortcutAction::MoveUp => &self.move_up,
            ShortcutAction::CloseLauncher => &self.close_launcher,
        }
    }

    pub fn update_binding(
        &mut self,
        action: ShortcutAction,
        binding: ShortcutBinding,
    ) -> Result<(), String> {
        *self.binding_mut(action) = binding;
        Ok(())
    }

    fn binding_mut(&mut self, action: ShortcutAction) -> &mut ShortcutBinding {
        match action {
            ShortcutAction::LaunchSelected => &mut self.launch_selected,
            ShortcutAction::Expand => &mut self.expand,
            ShortcutAction::MoveDown => &mut self.move_down,
            ShortcutAction::MoveUp => &mut self.move_up,
            ShortcutAction::CloseLauncher => &mut self.close_launcher,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    LaunchSelected,
    Expand,
    MoveDown,
    MoveUp,
    CloseLauncher,
}

impl ShortcutAction {
    pub const ALL: [Self; 5] = [
        Self::LaunchSelected,
        Self::Expand,
        Self::MoveDown,
        Self::MoveUp,
        Self::CloseLauncher,
    ];

    pub const fn config_key(self) -> &'static str {
        match self {
            Self::LaunchSelected => "shortcuts.launch_selected",
            Self::Expand => "shortcuts.expand",
            Self::MoveDown => "shortcuts.move_down",
            Self::MoveUp => "shortcuts.move_up",
            Self::CloseLauncher => "shortcuts.close_launcher",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::LaunchSelected => "launch-selected",
            Self::Expand => "expand",
            Self::MoveDown => "move-down",
            Self::MoveUp => "move-up",
            Self::CloseLauncher => "close-launcher",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Launch selected result",
            Self::Expand => "Expand results",
            Self::MoveDown => "Move down",
            Self::MoveUp => "Move up",
            Self::CloseLauncher => "Close launcher",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Runs the highlighted application or command.",
            Self::Expand => "Expands the empty launcher to show results.",
            Self::MoveDown => "Moves the highlighted selection downward.",
            Self::MoveUp => "Moves the highlighted selection upward.",
            Self::CloseLauncher => "Hides the launcher window.",
        }
    }
}

impl fmt::Display for ShortcutAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for ShortcutAction {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match normalize_action_name(value).as_str() {
            "launchselected" | "shortcutslaunchselected" => Ok(Self::LaunchSelected),
            "expand" | "shortcutsexpand" => Ok(Self::Expand),
            "expandormovedown" => Ok(Self::MoveDown),
            "movedown" | "shortcutsmovedown" => Ok(Self::MoveDown),
            "moveup" | "shortcutsmoveup" => Ok(Self::MoveUp),
            "closelauncher" | "shortcutscloselauncher" => Ok(Self::CloseLauncher),
            _ => Err(
                "expected one of: launch-selected, expand, move-down, move-up, close-launcher"
                    .to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub key: String,
}

impl ShortcutBinding {
    pub fn named(key: &str) -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
            key: key.to_string(),
        }
    }

    pub fn normalized_key(&self) -> String {
        normalize_key_name(&self.key)
    }
}

impl fmt::Display for ShortcutBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.super_key {
            parts.push("Super".to_string());
        }

        parts.push(pretty_key_name(&self.key));
        write!(f, "{}", parts.join("+"))
    }
}

impl FromStr for ShortcutBinding {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut binding = Self::named("");
        let mut key: Option<String> = None;

        for part in value
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => binding.ctrl = true,
                "alt" => binding.alt = true,
                "shift" => binding.shift = true,
                "super" | "meta" | "cmd" | "command" | "win" => binding.super_key = true,
                _ => {
                    if key.is_some() {
                        return Err("shortcut can only contain one key".to_string());
                    }

                    let normalized = normalize_key_name(part);
                    if normalized.is_empty() {
                        return Err("shortcut key cannot be empty".to_string());
                    }

                    key = Some(normalized);
                }
            }
        }

        binding.key = key.ok_or_else(|| "shortcut must include a key".to_string())?;
        Ok(binding)
    }
}

fn pretty_key_name(key: &str) -> String {
    match normalize_key_name(key).as_str() {
        "arrowup" => "ArrowUp".to_string(),
        "arrowdown" => "ArrowDown".to_string(),
        "arrowleft" => "ArrowLeft".to_string(),
        "arrowright" => "ArrowRight".to_string(),
        "escape" => "Escape".to_string(),
        "enter" => "Enter".to_string(),
        "space" => "Space".to_string(),
        normalized if normalized.len() == 1 => normalized.to_ascii_uppercase(),
        normalized => {
            let mut chars = normalized.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn normalize_key_name(key: &str) -> String {
    key.trim().to_ascii_lowercase().replace([' ', '_', '-'], "")
}

fn normalize_action_name(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '_', '-', '.'], "")
}

#[cfg(test)]
mod tests {
    use super::{ShortcutAction, ShortcutBinding, ShortcutPreferences};
    use std::str::FromStr;

    #[test]
    fn shortcut_actions_accept_cli_aliases() {
        assert_eq!(
            ShortcutAction::from_str("launch-selected").expect("action should parse"),
            ShortcutAction::LaunchSelected,
        );
        assert_eq!(
            ShortcutAction::from_str("shortcuts.close_launcher").expect("action should parse"),
            ShortcutAction::CloseLauncher,
        );
    }

    #[test]
    fn duplicate_shortcuts_are_allowed() {
        let mut shortcuts = ShortcutPreferences::default();

        shortcuts
            .update_binding(
                ShortcutAction::MoveUp,
                ShortcutBinding::from_str("ArrowDown").expect("binding should parse"),
            )
            .expect("duplicate bindings should be allowed");

        assert_eq!(shortcuts.move_up.to_string(), "ArrowDown");
    }

    #[test]
    fn legacy_expand_or_move_down_migrates_to_expand_and_move_down() {
        let shortcuts: ShortcutPreferences = toml::from_str(
            r#"
[launch_selected]
ctrl = false
alt = false
shift = false
super_key = false
key = "Enter"

[expand_or_move_down]
ctrl = false
alt = false
shift = false
super_key = false
key = "ArrowDown"

[move_up]
ctrl = false
alt = false
shift = false
super_key = false
key = "ArrowUp"

[close_launcher]
ctrl = false
alt = false
shift = false
super_key = false
key = "Escape"
"#,
        )
        .expect("legacy shortcut config should parse");

        assert_eq!(shortcuts.expand.to_string(), "ArrowDown");
        assert_eq!(shortcuts.move_down.to_string(), "ArrowDown");
    }
}
