use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortcutPreferences {
    pub launch_selected: ShortcutBinding,
    pub expand_or_move_down: ShortcutBinding,
    pub move_up: ShortcutBinding,
    pub close_launcher: ShortcutBinding,
}

impl Default for ShortcutPreferences {
    fn default() -> Self {
        Self {
            launch_selected: ShortcutBinding::named("Enter"),
            expand_or_move_down: ShortcutBinding::named("ArrowDown"),
            move_up: ShortcutBinding::named("ArrowUp"),
            close_launcher: ShortcutBinding::named("Escape"),
        }
    }
}

impl ShortcutPreferences {
    pub fn binding(&self, action: ShortcutAction) -> &ShortcutBinding {
        match action {
            ShortcutAction::LaunchSelected => &self.launch_selected,
            ShortcutAction::ExpandOrMoveDown => &self.expand_or_move_down,
            ShortcutAction::MoveUp => &self.move_up,
            ShortcutAction::CloseLauncher => &self.close_launcher,
        }
    }

    pub fn update_binding(
        &mut self,
        action: ShortcutAction,
        binding: ShortcutBinding,
    ) -> Result<(), String> {
        let mut next = self.clone();
        *next.binding_mut(action) = binding;
        next.validate_unique()?;
        *self = next;
        Ok(())
    }

    pub fn validate_unique(&self) -> Result<(), String> {
        for (index, left_action) in ShortcutAction::ALL.iter().copied().enumerate() {
            for right_action in ShortcutAction::ALL.iter().copied().skip(index + 1) {
                if self
                    .binding(left_action)
                    .same_as(self.binding(right_action))
                {
                    return Err(format!(
                        "{} conflicts with {}.",
                        left_action.label(),
                        right_action.label()
                    ));
                }
            }
        }

        Ok(())
    }

    fn binding_mut(&mut self, action: ShortcutAction) -> &mut ShortcutBinding {
        match action {
            ShortcutAction::LaunchSelected => &mut self.launch_selected,
            ShortcutAction::ExpandOrMoveDown => &mut self.expand_or_move_down,
            ShortcutAction::MoveUp => &mut self.move_up,
            ShortcutAction::CloseLauncher => &mut self.close_launcher,
        }
    }
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

    pub const fn config_key(self) -> &'static str {
        match self {
            Self::LaunchSelected => "shortcuts.launch_selected",
            Self::ExpandOrMoveDown => "shortcuts.expand_or_move_down",
            Self::MoveUp => "shortcuts.move_up",
            Self::CloseLauncher => "shortcuts.close_launcher",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::LaunchSelected => "launch-selected",
            Self::ExpandOrMoveDown => "expand-or-move-down",
            Self::MoveUp => "move-up",
            Self::CloseLauncher => "close-launcher",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Launch selected result",
            Self::ExpandOrMoveDown => "Expand or move down",
            Self::MoveUp => "Move up",
            Self::CloseLauncher => "Close launcher",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::LaunchSelected => "Runs the highlighted application or command.",
            Self::ExpandOrMoveDown => "Expands the empty launcher or moves selection down.",
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
            "expandormovedown" | "shortcutsexpandormovedown" => Ok(Self::ExpandOrMoveDown),
            "moveup" | "shortcutsmoveup" => Ok(Self::MoveUp),
            "closelauncher" | "shortcutscloselauncher" => Ok(Self::CloseLauncher),
            _ => Err(
                "expected one of: launch-selected, expand-or-move-down, move-up, close-launcher"
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

    pub fn same_as(&self, other: &Self) -> bool {
        self.ctrl == other.ctrl
            && self.alt == other.alt
            && self.shift == other.shift
            && self.super_key == other.super_key
            && self.normalized_key() == other.normalized_key()
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
    fn duplicate_shortcuts_are_rejected() {
        let mut shortcuts = ShortcutPreferences::default();

        let error = shortcuts
            .update_binding(
                ShortcutAction::MoveUp,
                ShortcutBinding::from_str("ArrowDown").expect("binding should parse"),
            )
            .expect_err("duplicate bindings should be rejected");

        assert!(error.contains("Move up"));
    }
}
