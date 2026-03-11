use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliMode {
    Toggle,
    Daemon,
    Preferences,
    Quit,
}

impl CliMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Toggle => "toggle",
            Self::Daemon => "daemon",
            Self::Preferences => "preferences",
            Self::Quit => "quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommand {
    Run(CliMode),
    Help,
    Version,
}

pub fn parse_command<I>(args: I) -> CliCommand
where
    I: IntoIterator<Item = OsString>,
{
    let flags: Vec<String> = args
        .into_iter()
        .filter_map(|arg| arg.into_string().ok())
        .collect();

    if flags
        .iter()
        .any(|flag| matches!(flag.as_str(), "--help" | "-h"))
    {
        return CliCommand::Help;
    }

    if flags
        .iter()
        .any(|flag| matches!(flag.as_str(), "--version" | "-v"))
    {
        return CliCommand::Version;
    }

    let mode = flags
        .into_iter()
        .fold(CliMode::Toggle, |mode, flag| match flag.as_str() {
            "--daemon" => CliMode::Daemon,
            "--preferences" => CliMode::Preferences,
            "--quit" => CliMode::Quit,
            "--toggle" => CliMode::Toggle,
            _ => mode,
        });

    CliCommand::Run(mode)
}
