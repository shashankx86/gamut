mod command;
mod output;

pub use command::{parse_command, CliCommand, CliMode, HelpTopic};
pub use output::{print_help, print_version};

#[cfg(test)]
mod tests {
    use super::output::{config_help_text, help_text, shortcut_help_text, version_text};

    #[test]
    fn root_help_mentions_config_command() {
        let help = help_text();

        assert!(help.contains("config"));
        assert!(help.contains("Usage:"));
        assert!(help.contains("Commands:"));
        assert!(help.contains("toggle"));
    }

    #[test]
    fn config_help_mentions_shortcut_modes() {
        let help = config_help_text();

        assert!(help.contains("shortcut"));
        assert!(help.contains("set <key> <value>"));
        assert!(help.contains("reset <key>|all"));
    }

    #[test]
    fn shortcut_help_mentions_interactive_mode() {
        let help = shortcut_help_text();

        assert!(help.contains("interactive"));
        assert!(help.contains("Modifier+Key"));
        assert!(help.contains("move-down"));
    }

    #[test]
    fn version_text_matches_package_metadata() {
        assert_eq!(
            version_text(),
            format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        );
    }
}
