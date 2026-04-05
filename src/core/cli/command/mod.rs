mod config;
mod help;
mod mode;
mod shared;

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
    Config(crate::core::preferences::ConfigCommand),
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
        "config" => config::parse_config_command(&args[1..]),
        "help" => help::parse_help_command(&args[1..]),
        _ => mode::parse_mode_command(&args),
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
    fn parses_modes() {
        assert_eq!(
            parse(&["daemon"]).expect("daemon should parse"),
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
