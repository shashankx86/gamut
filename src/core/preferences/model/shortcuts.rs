use serde::de::{Deserializer, Error as DeError};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

pub const VK_SHIFT: u32 = 16;
pub const VK_CONTROL: u32 = 17;
pub const VK_ALT: u32 = 18;
pub const VK_SUPER: u32 = 91;
pub const VK_BACKSPACE: u32 = 8;
pub const VK_TAB: u32 = 9;
pub const VK_ENTER: u32 = 13;
pub const VK_ESCAPE: u32 = 27;
pub const VK_SPACE: u32 = 32;
pub const VK_LEFT: u32 = 37;
pub const VK_UP: u32 = 38;
pub const VK_RIGHT: u32 = 39;
pub const VK_DOWN: u32 = 40;
pub const VK_PAGE_UP: u32 = 33;
pub const VK_PAGE_DOWN: u32 = 34;
pub const VK_END: u32 = 35;
pub const VK_HOME: u32 = 36;
pub const VK_INSERT: u32 = 45;
pub const VK_DELETE: u32 = 46;
pub const VK_NUMPAD_0: u32 = 96;
pub const VK_F1: u32 = 112;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutPreferences {
    pub launch_selected: ShortcutBinding,
    pub expand: ShortcutBinding,
    pub move_down: ShortcutBinding,
    pub move_up: ShortcutBinding,
    pub close_launcher: ShortcutBinding,
}

impl Default for ShortcutPreferences {
    fn default() -> Self {
        Self {
            launch_selected: ShortcutBinding::from_key_code(VK_ENTER),
            expand: ShortcutBinding::from_key_code(VK_DOWN),
            move_down: ShortcutBinding::from_key_code(VK_DOWN),
            move_up: ShortcutBinding::from_key_code(VK_UP),
            close_launcher: ShortcutBinding::from_key_code(VK_ESCAPE),
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
        match crate::core::preferences::normalize_identifier(value).as_str() {
            "launchselected" | "shortcutslaunchselected" => Ok(Self::LaunchSelected),
            "expand" | "shortcutsexpand" => Ok(Self::Expand),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub key_codes: Vec<u32>,
}

impl ShortcutBinding {
    pub fn from_key_code(key_code: u32) -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
            key_codes: vec![key_code],
        }
    }

    pub fn to_keycode_array(&self) -> Vec<u32> {
        let mut codes = Vec::new();

        if self.ctrl {
            codes.push(VK_CONTROL);
        }
        if self.alt {
            codes.push(VK_ALT);
        }
        if self.shift {
            codes.push(VK_SHIFT);
        }
        if self.super_key {
            codes.push(VK_SUPER);
        }

        for code in &self.key_codes {
            if !codes.contains(code) {
                codes.push(*code);
            }
        }

        codes
    }

    fn from_keycode_array(codes: Vec<u32>) -> Result<Self, String> {
        let mut binding = Self {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
            key_codes: Vec::new(),
        };

        for code in codes {
            match code {
                VK_CONTROL => binding.ctrl = true,
                VK_ALT => binding.alt = true,
                VK_SHIFT => binding.shift = true,
                VK_SUPER => binding.super_key = true,
                _ => {
                    if !binding.key_codes.contains(&code) {
                        binding.key_codes.push(code);
                    }
                }
            }
        }

        if binding.key_codes.is_empty() {
            return Err("shortcut must include at least one key code".to_string());
        }

        Ok(binding)
    }
}

pub(crate) fn virtual_keycode_from_ascii_char(ch: char) -> Option<u32> {
    if ch.is_ascii_alphabetic() {
        return Some(ch.to_ascii_uppercase() as u32);
    }

    if ch.is_ascii_digit() {
        return Some(ch as u32);
    }

    match ch {
        '/' => Some(191),
        '\\' => Some(220),
        '.' => Some(190),
        ',' => Some(188),
        ';' => Some(186),
        '\'' => Some(222),
        '[' => Some(219),
        ']' => Some(221),
        '-' => Some(189),
        '=' => Some(187),
        '`' => Some(192),
        _ => None,
    }
}

pub(crate) fn virtual_keycode_from_token(value: &str) -> Option<u32> {
    let normalized = crate::core::preferences::normalize_key_token(value);

    if normalized.is_empty() {
        return None;
    }

    match normalized.as_str() {
        "ctrl" | "control" | "controlleft" | "controlright" => Some(VK_CONTROL),
        "alt" | "altleft" | "altright" => Some(VK_ALT),
        "shift" | "shiftleft" | "shiftright" => Some(VK_SHIFT),
        "super" | "meta" | "cmd" | "command" | "win" | "metaleft" | "metaright" => Some(VK_SUPER),
        "enter" | "return" => Some(VK_ENTER),
        "escape" | "esc" => Some(VK_ESCAPE),
        "tab" => Some(VK_TAB),
        "backspace" => Some(VK_BACKSPACE),
        "space" | "spacebar" => Some(VK_SPACE),
        "left" | "arrowleft" => Some(VK_LEFT),
        "up" | "arrowup" => Some(VK_UP),
        "right" | "arrowright" => Some(VK_RIGHT),
        "down" | "arrowdown" => Some(VK_DOWN),
        "pageup" => Some(VK_PAGE_UP),
        "pagedown" => Some(VK_PAGE_DOWN),
        "home" => Some(VK_HOME),
        "end" => Some(VK_END),
        "insert" => Some(VK_INSERT),
        "delete" => Some(VK_DELETE),
        "slash" => Some(191),
        "backslash" => Some(220),
        "period" => Some(190),
        "comma" => Some(188),
        "semicolon" => Some(186),
        "quote" => Some(222),
        "bracketleft" => Some(219),
        "bracketright" => Some(221),
        "minus" => Some(189),
        "equal" => Some(187),
        "backquote" => Some(192),
        _ => {
            if let Some(value) = normalized.strip_prefix("key")
                && value.len() == 1
            {
                return value
                    .chars()
                    .next()
                    .and_then(virtual_keycode_from_ascii_char);
            }

            if let Some(value) = normalized.strip_prefix("digit")
                && value.len() == 1
            {
                return value
                    .chars()
                    .next()
                    .and_then(virtual_keycode_from_ascii_char);
            }

            if let Some(value) = normalized.strip_prefix("numpad")
                && value.len() == 1
                && let Some(digit) = value.chars().next().and_then(|ch| ch.to_digit(10))
            {
                return Some(VK_NUMPAD_0 + digit);
            }

            if let Some(value) = normalized.strip_prefix('f')
                && let Ok(number) = value.parse::<u32>()
                && (1..=12).contains(&number)
            {
                return Some(VK_F1 + number - 1);
            }

            if normalized.len() == 1 {
                return normalized
                    .chars()
                    .next()
                    .and_then(virtual_keycode_from_ascii_char);
            }

            None
        }
    }
}

impl Serialize for ShortcutBinding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_keycode_array().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ShortcutBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let codes = Vec::<u32>::deserialize(deserializer)?;
        Self::from_keycode_array(codes).map_err(D::Error::custom)
    }
}

impl fmt::Display for ShortcutBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self
            .to_keycode_array()
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "[{values}]")
    }
}

impl FromStr for ShortcutBinding {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        #[derive(Deserialize)]
        struct ShortcutBindingWrapper {
            binding: ShortcutBinding,
        }

        let wrapped = format!("binding = {value}");
        toml::from_str::<ShortcutBindingWrapper>(&wrapped)
            .map(|wrapper| wrapper.binding)
            .map_err(|error| format!("shortcut must be a keycode array like [17, 75]: {error}"))
    }
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
                ShortcutBinding::from_str("[40]").expect("binding should parse"),
            )
            .expect("duplicate bindings should be allowed");

        assert_eq!(shortcuts.move_up.to_string(), "[40]");
    }

    #[test]
    fn shortcut_binding_parses_modifier_and_multiple_key_codes() {
        let binding = ShortcutBinding::from_str("[17, 40, 74]").expect("binding should parse");

        assert!(binding.ctrl);
        assert_eq!(binding.key_codes, vec![40, 74]);
    }

    #[test]
    fn shortcut_binding_rejects_modifier_only_arrays() {
        let error = ShortcutBinding::from_str("[17]").expect_err("binding should be rejected");
        assert!(error.contains("at least one key code"));
    }
}
