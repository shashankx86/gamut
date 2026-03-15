#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommand {
    Show { target_output: Option<String> },
    Toggle { target_output: Option<String> },
    ReloadPreferences,
    Quit,
    Ping,
}

impl IpcCommand {
    pub(super) fn as_wire(&self) -> String {
        match self {
            Self::Show { target_output } => serialize_command("show", target_output.as_deref()),
            Self::Toggle { target_output } => serialize_command("toggle", target_output.as_deref()),
            Self::ReloadPreferences => "reload-preferences\n".to_string(),
            Self::Quit => "quit\n".to_string(),
            Self::Ping => "ping\n".to_string(),
        }
    }

    pub(super) fn from_wire(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        let (command, target_output) =
            trimmed
                .split_once('\t')
                .map_or((trimmed, None), |(command, output)| {
                    let output = (!output.is_empty()).then(|| output.to_string());
                    (command, output)
                });

        match command {
            "show" => Some(Self::Show { target_output }),
            "toggle" => Some(Self::Toggle { target_output }),
            "reload-preferences" => Some(Self::ReloadPreferences),
            "quit" => Some(Self::Quit),
            "ping" => Some(Self::Ping),
            _ => None,
        }
    }
}

fn serialize_command(command: &str, target_output: Option<&str>) -> String {
    match target_output {
        Some(output) => format!("{command}\t{output}\n"),
        None => format!("{command}\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::IpcCommand;

    #[test]
    fn parses_wire_commands() {
        assert_eq!(
            IpcCommand::from_wire("show"),
            Some(IpcCommand::Show {
                target_output: None,
            })
        );
        assert_eq!(
            IpcCommand::from_wire("toggle"),
            Some(IpcCommand::Toggle {
                target_output: None,
            })
        );
        assert_eq!(
            IpcCommand::from_wire("toggle\tDP-1"),
            Some(IpcCommand::Toggle {
                target_output: Some("DP-1".to_string()),
            })
        );
        assert_eq!(
            IpcCommand::from_wire("reload-preferences"),
            Some(IpcCommand::ReloadPreferences),
        );
        assert_eq!(IpcCommand::from_wire("quit"), Some(IpcCommand::Quit));
        assert_eq!(IpcCommand::from_wire("ping"), Some(IpcCommand::Ping));
        assert_eq!(IpcCommand::from_wire("noop"), None);
    }

    #[test]
    fn writes_wire_commands() {
        assert_eq!(
            IpcCommand::Show {
                target_output: None,
            }
            .as_wire(),
            "show\n"
        );
        assert_eq!(
            IpcCommand::Toggle {
                target_output: None,
            }
            .as_wire(),
            "toggle\n"
        );
        assert_eq!(
            IpcCommand::Toggle {
                target_output: Some("DP-1".to_string()),
            }
            .as_wire(),
            "toggle\tDP-1\n"
        );
        assert_eq!(
            IpcCommand::ReloadPreferences.as_wire(),
            "reload-preferences\n",
        );
        assert_eq!(IpcCommand::Quit.as_wire(), "quit\n");
        assert_eq!(IpcCommand::Ping.as_wire(), "ping\n");
    }
}
