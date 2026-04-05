use super::shared::{config_help_topic, is_shortcut_alias, shortcut_help_topic};
use super::{CliCommand, CliError, HelpTopic};

pub(super) fn parse_help_command(args: &[String]) -> Result<CliCommand, CliError> {
    match args {
        [] => Ok(CliCommand::Help(HelpTopic::Root)),
        [config] if config == "config" => Ok(CliCommand::Help(HelpTopic::Config)),
        [config, shortcut] if config == "config" && is_shortcut_alias(shortcut) => {
            Ok(CliCommand::Help(HelpTopic::ConfigShortcut))
        }
        [config, subcommand] if config == "config" => {
            Ok(CliCommand::Help(config_help_topic(subcommand)?))
        }
        [config, shortcut, subcommand] if config == "config" && is_shortcut_alias(shortcut) => {
            Ok(CliCommand::Help(shortcut_help_topic(subcommand)?))
        }
        _ => Err(CliError::new("unknown help topic", HelpTopic::Root)),
    }
}
