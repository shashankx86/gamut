use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;

const SOCKET_NAME: &str = "gamut-launcher.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommand {
    Toggle,
    Quit,
    Ping,
}

impl IpcCommand {
    fn as_wire(self) -> &'static str {
        match self {
            Self::Toggle => "toggle\n",
            Self::Quit => "quit\n",
            Self::Ping => "ping\n",
        }
    }

    fn from_wire(line: &str) -> Option<Self> {
        match line.trim() {
            "toggle" => Some(Self::Toggle),
            "quit" => Some(Self::Quit),
            "ping" => Some(Self::Ping),
            _ => None,
        }
    }
}

pub fn send_command(command: IpcCommand) -> io::Result<()> {
    send_command_to_path(&socket_path(), command)
}

pub fn is_daemon_running() -> bool {
    send_command(IpcCommand::Ping).is_ok()
}

pub fn start_listener() -> io::Result<Receiver<IpcCommand>> {
    start_listener_at(&socket_path())
}

pub fn socket_path() -> PathBuf {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    let uid = current_uid();
    let run_user = PathBuf::from(format!("/run/user/{uid}"));

    socket_path_for(runtime_dir.as_deref(), uid, run_user.is_dir())
}

fn socket_path_for(runtime_dir: Option<&Path>, uid: u32, run_user_exists: bool) -> PathBuf {
    if let Some(dir) = runtime_dir {
        return dir.join(SOCKET_NAME);
    }

    if run_user_exists {
        return PathBuf::from(format!("/run/user/{uid}/{SOCKET_NAME}"));
    }

    PathBuf::from(format!("/tmp/{SOCKET_NAME}.{uid}"))
}

fn send_command_to_path(path: &Path, command: IpcCommand) -> io::Result<()> {
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(command.as_wire().as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn start_listener_at(path: &Path) -> io::Result<Receiver<IpcCommand>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        match UnixStream::connect(path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "daemon already running",
                ));
            }
            Err(_) => {
                let _ = fs::remove_file(path);
            }
        }
    }

    let listener = UnixListener::bind(path)?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for incoming in listener.incoming() {
            let Ok(stream) = incoming else {
                continue;
            };

            if let Some(command) = read_command(stream)
                && tx.send(command).is_err()
            {
                break;
            }
        }
    });

    Ok(rx)
}

fn read_command(stream: UnixStream) -> Option<IpcCommand> {
    let mut line = String::new();
    let mut reader = BufReader::new(stream);
    if reader.read_line(&mut line).ok()? == 0 {
        return None;
    }

    IpcCommand::from_wire(&line)
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use super::{IpcCommand, SOCKET_NAME, socket_path_for};
    use std::path::PathBuf;

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

    #[test]
    fn prefers_xdg_runtime_socket_path() {
        let path = socket_path_for(Some(PathBuf::from("/tmp/runtime").as_path()), 1000, true);
        assert_eq!(path, PathBuf::from(format!("/tmp/runtime/{SOCKET_NAME}")));
    }

    #[test]
    fn falls_back_to_run_user_socket_path() {
        let path = socket_path_for(None, 1001, true);
        assert_eq!(path, PathBuf::from(format!("/run/user/1001/{SOCKET_NAME}")));
    }

    #[test]
    fn falls_back_to_tmp_socket_path() {
        let path = socket_path_for(None, 1002, false);
        assert_eq!(
            path,
            PathBuf::from(format!("/tmp/{SOCKET_NAME}.1002"))
        );
    }
}
