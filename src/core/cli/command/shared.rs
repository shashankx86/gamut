use super::{CliCommand, CliError, HelpTopic};
use crate::core::preferences::{ConfigKey, ShortcutAction};

pub(super) fn require_exact_args<F>(
    args: &[String],
    expected: usize,
    build: F,
) -> Result<CliCommand, CliError>
where
    F: FnOnce() -> CliCommand,
{
    if args.len() == expected {
        Ok(build())
    } else {
        Err(CliError::new(
            "unexpected extra arguments",
            HelpTopic::Config,
        ))
    }
}

pub(super) fn parse_config_key_for(value: &str, topic: HelpTopic) -> Result<ConfigKey, CliError> {
    value
        .parse::<ConfigKey>()
        .map_err(|error| CliError::new(error, topic))
}

pub(super) fn parse_shortcut_action_for(
    value: &str,
    topic: HelpTopic,
) -> Result<ShortcutAction, CliError> {
    value
        .parse::<ShortcutAction>()
        .map_err(|error| CliError::new(error, topic))
}

pub(super) fn contains_help_flag(args: &[String]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h" | "help"))
}

pub(super) fn is_shortcut_alias(value: &str) -> bool {
    matches!(value, "shortcut" | "shortcuts")
}

pub(super) fn config_help_topic(value: &str) -> Result<HelpTopic, CliError> {
    match value {
        "show" => Ok(HelpTopic::ConfigShow),
        "path" => Ok(HelpTopic::ConfigPath),
        "keys" => Ok(HelpTopic::ConfigKeys),
        "get" => Ok(HelpTopic::ConfigGet),
        "set" => Ok(HelpTopic::ConfigSet),
        "reset" => Ok(HelpTopic::ConfigReset),
        value if is_shortcut_alias(value) => Ok(HelpTopic::ConfigShortcut),
        _ => Err(CliError::new(
            "unknown config help topic",
            HelpTopic::Config,
        )),
    }
}

pub(super) fn shortcut_help_topic(value: &str) -> Result<HelpTopic, CliError> {
    match value {
        "list" => Ok(HelpTopic::ConfigShortcutList),
        "set" => Ok(HelpTopic::ConfigShortcutSet),
        "interactive" => Ok(HelpTopic::ConfigShortcutInteractive),
        _ => Err(CliError::new(
            "unknown shortcut help topic",
            HelpTopic::ConfigShortcut,
        )),
    }
}

pub(super) fn help_topic_for_config_args(args: &[String]) -> HelpTopic {
    match args {
        [command, ..] if is_shortcut_alias(command) => match args.get(1).map(String::as_str) {
            Some("list") => HelpTopic::ConfigShortcutList,
            Some("set") => HelpTopic::ConfigShortcutSet,
            Some("interactive") => HelpTopic::ConfigShortcutInteractive,
            _ => HelpTopic::ConfigShortcut,
        },
        [command, ..] => config_help_topic(command).unwrap_or(HelpTopic::Config),
        [] => HelpTopic::Config,
    }
}
