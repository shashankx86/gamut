mod command;
mod output;

pub use command::{CliCommand, CliMode, parse_command};
pub use output::{print_help, print_version};

#[cfg(test)]
mod tests {
    use super::output::{help_text, version_text};
    use super::{CliCommand, CliMode, parse_command};
    use std::ffi::OsString;

    #[test]
    fn defaults_to_toggle() {
        assert_eq!(
            parse_command(Vec::<OsString>::new()),
            CliCommand::Run(CliMode::Toggle),
        );
    }

    #[test]
    fn parses_known_flags_in_any_position() {
        assert_eq!(
            parse_command([OsString::from("--what"), OsString::from("--daemon")]),
            CliCommand::Run(CliMode::Daemon),
        );
        assert_eq!(
            parse_command([OsString::from("value"), OsString::from("--quit")]),
            CliCommand::Run(CliMode::Quit),
        );
        assert_eq!(
            parse_command([OsString::from("value"), OsString::from("--toggle")]),
            CliCommand::Run(CliMode::Toggle),
        );
        assert_eq!(
            parse_command([OsString::from("value"), OsString::from("--help")]),
            CliCommand::Help,
        );
        assert_eq!(
            parse_command([OsString::from("value"), OsString::from("-h")]),
            CliCommand::Help,
        );
        assert_eq!(
            parse_command([OsString::from("value"), OsString::from("--version")]),
            CliCommand::Version,
        );
        assert_eq!(
            parse_command([OsString::from("value"), OsString::from("-v")]),
            CliCommand::Version,
        );
    }

    #[test]
    fn last_known_mode_flag_wins_when_multiple_are_provided() {
        assert_eq!(
            parse_command([
                OsString::from("--daemon"),
                OsString::from("--quit"),
                OsString::from("--toggle"),
            ]),
            CliCommand::Run(CliMode::Toggle),
        );
    }

    #[test]
    fn help_takes_precedence_over_other_flags() {
        assert_eq!(
            parse_command([
                OsString::from("--daemon"),
                OsString::from("--version"),
                OsString::from("--help"),
                OsString::from("--quit"),
            ]),
            CliCommand::Help,
        );
    }

    #[test]
    fn version_takes_precedence_over_mode_flags() {
        assert_eq!(
            parse_command([
                OsString::from("--daemon"),
                OsString::from("--version"),
                OsString::from("--quit"),
            ]),
            CliCommand::Version,
        );
    }

    #[test]
    fn help_text_lists_version_option() {
        let help = help_text();

        assert!(help.contains("-v, --version"));
        assert!(help.contains("gamut [OPTIONS]"));
    }

    #[test]
    fn version_text_matches_package_metadata() {
        assert_eq!(
            version_text(),
            format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        );
    }
}
