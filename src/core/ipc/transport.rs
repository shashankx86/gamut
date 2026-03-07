use super::command::IpcCommand;
use super::path::socket_path;
use log::{debug, info, warn};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

const READ_COMMAND_TIMEOUT: Duration = Duration::from_millis(250);

pub fn send_command(command: IpcCommand) -> io::Result<()> {
    send_command_to_path(&socket_path(), command)
}

pub fn is_daemon_running() -> bool {
    send_command(IpcCommand::Ping).is_ok()
}

pub fn start_listener() -> io::Result<Receiver<IpcCommand>> {
    start_listener_at(&socket_path())
}

fn send_command_to_path(path: &Path, command: IpcCommand) -> io::Result<()> {
    debug!(
        "connecting to IPC socket {} for {:?}",
        path.display(),
        command
    );
    let mut stream = UnixStream::connect(path)?;
    write_command_to(&mut stream, command)?;
    Ok(())
}

fn start_listener_at(path: &Path) -> io::Result<Receiver<IpcCommand>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    remove_stale_socket(path)?;

    let listener = UnixListener::bind(path)?;
    info!("listening for IPC commands on {}", path.display());
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for incoming in listener.incoming() {
            let Ok(stream) = incoming else {
                if let Err(error) = incoming {
                    warn!("failed to accept IPC connection: {error}");
                }
                continue;
            };

            let tx = tx.clone();
            thread::spawn(move || {
                if let Some(command) = read_command(stream) {
                    let _ = tx.send(command);
                }
            });
        }
    });

    Ok(rx)
}

fn remove_stale_socket(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("refusing to replace non-socket path: {}", path.display()),
        ));
    }

    match UnixStream::connect(path) {
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            "daemon already running",
        )),
        Err(_) => {
            debug!("removing stale IPC socket at {}", path.display());
            fs::remove_file(path)
        }
    }
}

fn read_command(stream: UnixStream) -> Option<IpcCommand> {
    let _ = stream.set_read_timeout(Some(READ_COMMAND_TIMEOUT));

    let mut reader = BufReader::new(stream);
    read_command_from(&mut reader)
}

fn write_command_to<W: Write>(writer: &mut W, command: IpcCommand) -> io::Result<()> {
    writer.write_all(command.as_wire().as_bytes())?;
    writer.flush()
}

fn read_command_from<R: BufRead>(reader: &mut R) -> Option<IpcCommand> {
    let mut line = String::new();

    match reader.read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => IpcCommand::from_wire(&line),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
            ) =>
        {
            None
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{read_command_from, start_listener_at, write_command_to, IpcCommand};
    use std::fs;
    use std::io::{self, BufReader, Cursor};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_socket_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let dir = PathBuf::from("/tmp").join(format!("gamut-ipc-test-{nanos}"));
        fs::create_dir_all(&dir).expect("failed to create test directory");
        dir.join("launcher.sock")
    }

    fn cleanup_socket_path(path: &PathBuf) {
        let _ = fs::remove_file(path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn write_command_serializes_wire_protocol() {
        let mut buffer = Vec::new();

        write_command_to(&mut buffer, IpcCommand::Toggle).expect("toggle should serialize");

        assert_eq!(buffer, b"toggle\n");
    }

    #[test]
    fn rejects_non_socket_paths_without_deleting_them() {
        let socket_path = unique_test_socket_path();
        fs::write(&socket_path, "not a socket").expect("failed to create blocking file");

        let error =
            start_listener_at(&socket_path).expect_err("listener should reject non-socket path");
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(socket_path.is_file());

        cleanup_socket_path(&socket_path);
    }

    #[test]
    fn read_command_parses_trimmed_wire_protocol() {
        let cursor = Cursor::new(b"  quit \n".to_vec());
        let mut reader = BufReader::new(cursor);

        assert_eq!(read_command_from(&mut reader), Some(IpcCommand::Quit));
    }

    #[test]
    fn read_command_ignores_unknown_payloads() {
        let cursor = Cursor::new(b"noop\n".to_vec());
        let mut reader = BufReader::new(cursor);

        assert_eq!(read_command_from(&mut reader), None);
    }
}
