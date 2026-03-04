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
    match args
        .into_iter()
        .next()
        .and_then(|arg| arg.into_string().ok())
    {
        Some(flag) if flag == "--daemon" => CliMode::Daemon,
        Some(flag) if flag == "--quit" => CliMode::Quit,
        Some(flag) if flag == "--toggle" => CliMode::Toggle,
        _ => CliMode::Toggle,
    }
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
    fn parses_known_flags() {
        assert_eq!(parse_mode([OsString::from("--daemon")]), CliMode::Daemon);
        assert_eq!(parse_mode([OsString::from("--quit")]), CliMode::Quit);
        assert_eq!(parse_mode([OsString::from("--toggle")]), CliMode::Toggle);
    }

    #[test]
    fn unknown_flag_falls_back_to_toggle() {
        assert_eq!(parse_mode([OsString::from("--what")]), CliMode::Toggle);
    }
}
