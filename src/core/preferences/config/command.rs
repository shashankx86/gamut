use super::ConfigKey;
use crate::core::preferences::ShortcutAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommand {
    Show,
    Path,
    Keys,
    Get { key: ConfigKey },
    Set { key: ConfigKey, value: String },
    Reset { target: ConfigResetTarget },
    Shortcut(ShortcutConfigCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutConfigCommand {
    List,
    Set {
        action: ShortcutAction,
        binding: String,
    },
    Interactive {
        action: Option<ShortcutAction>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigResetTarget {
    All,
    Key(ConfigKey),
}
