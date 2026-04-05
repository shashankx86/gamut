use super::shared::{
    contains_help_flag, help_topic_for_config_args, is_shortcut_alias, parse_config_key_for,
    parse_shortcut_action_for, require_exact_args,
};
use super::{CliCommand, CliError, HelpTopic};
use crate::core::preferences::{ConfigCommand, ConfigResetTarget, ShortcutConfigCommand};

pub(super) fn parse_config_command(args: &[String]) -> Result<CliCommand, CliError> {
    if args.is_empty() {
        return Ok(CliCommand::Help(HelpTopic::Config));
    }

    if contains_help_flag(args) {
        return Ok(CliCommand::Help(help_topic_for_config_args(args)));
    }

    match args[0].as_str() {
        "show" => require_exact_args(args, 1, || CliCommand::Config(ConfigCommand::Show)),
        "path" => require_exact_args(args, 1, || CliCommand::Config(ConfigCommand::Path)),
        "keys" => require_exact_args(args, 1, || CliCommand::Config(ConfigCommand::Keys)),
        "get" => parse_config_get(args),
        "set" => parse_config_set(args),
        "reset" => parse_config_reset(args),
        value if is_shortcut_alias(value) => parse_shortcut_command(&args[1..]),
        value => Err(CliError::new(
            format!("unknown config subcommand `{value}`"),
            HelpTopic::Config,
        )),
    }
}

fn parse_config_get(args: &[String]) -> Result<CliCommand, CliError> {
    let Some(key) = args.get(1) else {
        return Err(CliError::new(
            "missing config key for `config get`",
            HelpTopic::ConfigGet,
        ));
    };

    if args.len() != 2 {
        return Err(CliError::new(
            "`config get` expects exactly one key",
            HelpTopic::ConfigGet,
        ));
    }

    Ok(CliCommand::Config(ConfigCommand::Get {
        key: parse_config_key_for(key, HelpTopic::ConfigGet)?,
    }))
}

fn parse_config_set(args: &[String]) -> Result<CliCommand, CliError> {
    let Some(key) = args.get(1) else {
        return Err(CliError::new(
            "missing config key for `config set`",
            HelpTopic::ConfigSet,
        ));
    };
    let Some(value) = args.get(2) else {
        return Err(CliError::new(
            "missing value for `config set`",
            HelpTopic::ConfigSet,
        ));
    };

    if args.len() != 3 {
        return Err(CliError::new(
            "`config set` expects exactly one key and one value",
            HelpTopic::ConfigSet,
        ));
    }

    Ok(CliCommand::Config(ConfigCommand::Set {
        key: parse_config_key_for(key, HelpTopic::ConfigSet)?,
        value: value.clone(),
    }))
}

fn parse_config_reset(args: &[String]) -> Result<CliCommand, CliError> {
    let target = match args.get(1) {
        Some(value) if value == "all" => ConfigResetTarget::All,
        Some(value) => ConfigResetTarget::Key(parse_config_key_for(value, HelpTopic::ConfigReset)?),
        None => ConfigResetTarget::All,
    };

    if args.len() > 2 {
        return Err(CliError::new(
            "`config reset` accepts `all` or a single key",
            HelpTopic::ConfigReset,
        ));
    }

    Ok(CliCommand::Config(ConfigCommand::Reset { target }))
}

fn parse_shortcut_command(args: &[String]) -> Result<CliCommand, CliError> {
    if args.is_empty() {
        return Ok(CliCommand::Help(HelpTopic::ConfigShortcut));
    }

    if contains_help_flag(args) {
        return Ok(CliCommand::Help(HelpTopic::ConfigShortcut));
    }

    match args[0].as_str() {
        "list" => require_exact_args(args, 1, || {
            CliCommand::Config(ConfigCommand::Shortcut(ShortcutConfigCommand::List))
        }),
        "set" => {
            let Some(action) = args.get(1) else {
                return Err(CliError::new(
                    "missing shortcut action for `config shortcut set`",
                    HelpTopic::ConfigShortcutSet,
                ));
            };
            let Some(binding) = args.get(2) else {
                return Err(CliError::new(
                    "missing shortcut binding for `config shortcut set`",
                    HelpTopic::ConfigShortcutSet,
                ));
            };

            if args.len() != 3 {
                return Err(CliError::new(
                    "`config shortcut set` expects one action and one binding",
                    HelpTopic::ConfigShortcutSet,
                ));
            }

            Ok(CliCommand::Config(ConfigCommand::Shortcut(
                ShortcutConfigCommand::Set {
                    action: parse_shortcut_action_for(action, HelpTopic::ConfigShortcutSet)?,
                    binding: binding.clone(),
                },
            )))
        }
        "interactive" => {
            let action = match args.get(1) {
                Some(value) => Some(parse_shortcut_action_for(
                    value,
                    HelpTopic::ConfigShortcutInteractive,
                )?),
                None => None,
            };

            if args.len() > 2 {
                return Err(CliError::new(
                    "`config shortcut interactive` accepts at most one action",
                    HelpTopic::ConfigShortcutInteractive,
                ));
            }

            Ok(CliCommand::Config(ConfigCommand::Shortcut(
                ShortcutConfigCommand::Interactive { action },
            )))
        }
        value => Err(CliError::new(
            format!("unknown shortcut subcommand `{value}`"),
            HelpTopic::ConfigShortcut,
        )),
    }
}
