use super::{CliCommand, CliError, CliMode, HelpTopic};

pub(super) fn parse_mode_command(args: &[String]) -> Result<CliCommand, CliError> {
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
            "daemon" => mode = CliMode::Daemon,
            "quit" => mode = CliMode::Quit,
            "toggle" => mode = CliMode::Toggle,
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
