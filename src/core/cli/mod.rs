use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliMode {
    Toggle,
    Daemon,
    Quit,
}

pub fn parse_mode<I>(args: I) -> CliMode
where
    I: IntoIterator<Item = OsString>,
{
    args.into_iter()
        .filter_map(|arg| arg.into_string().ok())
        .fold(CliMode::Toggle, |mode, flag| match flag.as_str() {
            "--daemon" => CliMode::Daemon,
            "--quit" => CliMode::Quit,
            "--toggle" => CliMode::Toggle,
            _ => mode,
        })
}

#[cfg(test)]
mod tests {
    use super::{CliMode, parse_mode};
    use std::ffi::OsString;

    #[test]
    fn defaults_to_toggle() {
        assert_eq!(parse_mode(Vec::<OsString>::new()), CliMode::Toggle);
    }

    #[test]
    fn parses_known_flags_in_any_position() {
        assert_eq!(
            parse_mode([OsString::from("--what"), OsString::from("--daemon")]),
            CliMode::Daemon,
        );
        assert_eq!(
            parse_mode([OsString::from("value"), OsString::from("--quit")]),
            CliMode::Quit,
        );
        assert_eq!(
            parse_mode([OsString::from("value"), OsString::from("--toggle")]),
            CliMode::Toggle,
        );
    }

    #[test]
    fn last_known_flag_wins_when_multiple_are_provided() {
        assert_eq!(
            parse_mode([
                OsString::from("--daemon"),
                OsString::from("--quit"),
                OsString::from("--toggle"),
            ]),
            CliMode::Toggle,
        );
    }
}
