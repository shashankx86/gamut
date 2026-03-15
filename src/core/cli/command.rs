use crate::core::preferences::{
    ConfigCommand, ConfigKey, ConfigResetTarget, ShortcutAction, ShortcutConfigCommand,
};
use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliMode {
    Toggle,
    Daemon,
    Quit,
}

impl CliMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Toggle => "toggle",
            Self::Daemon => "daemon",
            Self::Quit => "quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    Root,
    Config,
    ConfigGet,
    ConfigSet,
    ConfigReset,
    ConfigPath,
    ConfigKeys,
    ConfigShow,
    ConfigShortcut,
    ConfigShortcutList,
    ConfigShortcutSet,
    ConfigShortcutInteractive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Run(CliMode),
    Config(ConfigCommand),
    Help(HelpTopic),
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    message: String,
    help_topic: HelpTopic,
}

impl CliError {
    fn new(message: impl Into<String>, help_topic: HelpTopic) -> Self {
        Self {
            message: message.into(),
            help_topic,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn help_topic(&self) -> HelpTopic {
        self.help_topic
    }
}

pub fn parse_command<I>(args: I) -> Result<CliCommand, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let args: Vec<String> = args
        .into_iter()
        .filter_map(|arg| arg.into_string().ok())
        .collect();

    if args.is_empty() {
        return Ok(CliCommand::Run(CliMode::Toggle));
    }

    match args[0].as_str() {
        "config" => parse_config_command(&args[1..]),
        "help" => parse_help_command(&args[1..]),
        _ => parse_legacy_mode_command(&args),
    }
}

fn parse_help_command(args: &[String]) -> Result<CliCommand, CliError> {
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

fn parse_legacy_mode_command(args: &[String]) -> Result<CliCommand, CliError> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return Ok(CliCommand::Help(HelpTopic::Root));
    }

    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--version" | "-v"))
    {
        return Ok(CliCommand::Version);
    }

    let mut mode = CliMode::Toggle;

    for arg in args {
        match arg.as_str() {
            "--daemon" | "daemon" => mode = CliMode::Daemon,
            "--quit" | "quit" => mode = CliMode::Quit,
            "--toggle" | "toggle" => mode = CliMode::Toggle,
            value if value.starts_with('-') => {
                return Err(CliError::new(
                    format!("unknown option `{value}`"),
                    HelpTopic::Root,
                ));
            }
            value => {
                return Err(CliError::new(
                    format!("unknown command `{value}`"),
                    HelpTopic::Root,
                ));
            }
        }
    }

    Ok(CliCommand::Run(mode))
}

fn parse_config_command(args: &[String]) -> Result<CliCommand, CliError> {
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

fn require_exact_args<F>(args: &[String], expected: usize, build: F) -> Result<CliCommand, CliError>
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

fn parse_config_key_for(value: &str, topic: HelpTopic) -> Result<ConfigKey, CliError> {
    value
        .parse::<ConfigKey>()
        .map_err(|error| CliError::new(error, topic))
}

fn parse_shortcut_action_for(value: &str, topic: HelpTopic) -> Result<ShortcutAction, CliError> {
    value
        .parse::<ShortcutAction>()
        .map_err(|error| CliError::new(error, topic))
}

fn contains_help_flag(args: &[String]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h" | "help"))
}

fn help_topic_for_config_args(args: &[String]) -> HelpTopic {
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

fn is_shortcut_alias(value: &str) -> bool {
    matches!(value, "shortcut" | "shortcuts")
}

fn config_help_topic(value: &str) -> Result<HelpTopic, CliError> {
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

fn shortcut_help_topic(value: &str) -> Result<HelpTopic, CliError> {
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

#[cfg(test)]
mod tests {
    use super::{CliCommand, CliError, CliMode, HelpTopic, parse_command};
    use crate::core::preferences::{
        ConfigCommand, ConfigKey, ConfigResetTarget, ShortcutAction, ShortcutConfigCommand,
    };
    use std::ffi::OsString;

    fn parse(args: &[&str]) -> Result<CliCommand, CliError> {
        parse_command(args.iter().map(OsString::from))
    }

    #[test]
    fn defaults_to_toggle() {
        assert_eq!(
            parse(&[]).expect("default command should parse"),
            CliCommand::Run(CliMode::Toggle)
        );
    }

    #[test]
    fn parses_legacy_modes() {
        assert_eq!(
            parse(&["--daemon"]).expect("daemon should parse"),
            CliCommand::Run(CliMode::Daemon)
        );
        assert_eq!(
            parse(&["quit"]).expect("quit should parse"),
            CliCommand::Run(CliMode::Quit)
        );
    }

    #[test]
    fn parses_config_set_command() {
        assert_eq!(
            parse(&["config", "set", "appearance.theme", "dark"]).expect("config set should parse"),
            CliCommand::Config(ConfigCommand::Set {
                key: ConfigKey::AppearanceTheme,
                value: "dark".to_string(),
            }),
        );
    }

    #[test]
    fn parses_shortcut_commands() {
        assert_eq!(
            parse(&["config", "shortcut", "set", "move-up", "Ctrl+K"])
                .expect("shortcut set should parse"),
            CliCommand::Config(ConfigCommand::Shortcut(ShortcutConfigCommand::Set {
                action: ShortcutAction::MoveUp,
                binding: "Ctrl+K".to_string(),
            })),
        );
        assert_eq!(
            parse(&["config", "shortcut", "interactive", "close-launcher"])
                .expect("shortcut interactive should parse"),
            CliCommand::Config(ConfigCommand::Shortcut(
                ShortcutConfigCommand::Interactive {
                    action: Some(ShortcutAction::CloseLauncher),
                }
            )),
        );
    }

    #[test]
    fn routes_help_to_nested_topics() {
        assert_eq!(
            parse(&["config", "--help"]).expect("config help should parse"),
            CliCommand::Help(HelpTopic::Config),
        );
        assert_eq!(
            parse(&["config", "shortcut", "--help"]).expect("shortcut help should parse"),
            CliCommand::Help(HelpTopic::ConfigShortcut),
        );
        assert_eq!(
            parse(&["help", "config", "reset"]).expect("reset help should parse"),
            CliCommand::Help(HelpTopic::ConfigReset),
        );
        assert_eq!(
            parse(&["help", "config", "shortcut", "interactive"])
                .expect("interactive help should parse"),
            CliCommand::Help(HelpTopic::ConfigShortcutInteractive),
        );
    }

    #[test]
    fn config_defaults_to_help() {
        assert_eq!(
            parse(&["config"]).expect("config should parse"),
            CliCommand::Help(HelpTopic::Config),
        );
        assert_eq!(
            parse(&["config", "shortcut"]).expect("shortcut should parse"),
            CliCommand::Help(HelpTopic::ConfigShortcut),
        );
    }

    #[test]
    fn parses_reset_all_command() {
        assert_eq!(
            parse(&["config", "reset", "all"]).expect("reset all should parse"),
            CliCommand::Config(ConfigCommand::Reset {
                target: ConfigResetTarget::All,
            }),
        );
    }

    #[test]
    fn rejects_unknown_preferences_flag() {
        let error = parse(&["--preferences"]).expect_err("preferences flag should be rejected");
        assert!(error.message().contains("unknown option"));
        assert_eq!(error.help_topic(), HelpTopic::Root);
    }
}
