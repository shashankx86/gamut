#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommand {
    Toggle,
    Quit,
    Ping,
}

impl IpcCommand {
    pub(super) fn as_wire(self) -> &'static str {
        match self {
            Self::Toggle => "toggle\n",
            Self::Quit => "quit\n",
            Self::Ping => "ping\n",
        }
    }

    pub(super) fn from_wire(line: &str) -> Option<Self> {
        match line.trim() {
            "toggle" => Some(Self::Toggle),
            "quit" => Some(Self::Quit),
            "ping" => Some(Self::Ping),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IpcCommand;

    #[test]
    fn parses_wire_commands() {
        assert_eq!(IpcCommand::from_wire("toggle"), Some(IpcCommand::Toggle));
        assert_eq!(IpcCommand::from_wire("quit"), Some(IpcCommand::Quit));
        assert_eq!(IpcCommand::from_wire("ping"), Some(IpcCommand::Ping));
        assert_eq!(IpcCommand::from_wire("noop"), None);
    }

    #[test]
    fn writes_wire_commands() {
        assert_eq!(IpcCommand::Toggle.as_wire(), "toggle\n");
        assert_eq!(IpcCommand::Quit.as_wire(), "quit\n");
        assert_eq!(IpcCommand::Ping.as_wire(), "ping\n");
    }
}
