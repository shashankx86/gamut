use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommand {
    Toggle,
    Quit,
    Ping,
}

pub fn send_command(_command: IpcCommand) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::NotConnected,
        "ipc transport not implemented yet",
    ))
}

pub fn is_daemon_running() -> bool {
    send_command(IpcCommand::Ping).is_ok()
}
