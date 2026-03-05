use super::command::IpcCommand;
use super::path::socket_path;
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
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(command.as_wire().as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn start_listener_at(path: &Path) -> io::Result<Receiver<IpcCommand>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    remove_stale_socket(path)?;

    let listener = UnixListener::bind(path)?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for incoming in listener.incoming() {
            let Ok(stream) = incoming else {
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
        Err(_) => fs::remove_file(path),
    }
}

fn read_command(stream: UnixStream) -> Option<IpcCommand> {
    let _ = stream.set_read_timeout(Some(READ_COMMAND_TIMEOUT));

    let mut line = String::new();
    let mut reader = BufReader::new(stream);
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
    use super::{IpcCommand, send_command_to_path, start_listener_at};
    use std::env;
    use std::fs;
    use std::io;
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::process;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn unique_test_socket_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!("gamut-ipc-test-{}-{nanos}", process::id()));
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
    fn receives_commands_over_unix_socket() {
        let socket_path = unique_test_socket_path();
        let receiver = start_listener_at(&socket_path).expect("listener should start");

        send_command_to_path(&socket_path, IpcCommand::Toggle)
            .expect("toggle command should be delivered");

        let received = receiver
            .recv_timeout(Duration::from_millis(700))
            .expect("listener should receive command");
        assert_eq!(received, IpcCommand::Toggle);

        cleanup_socket_path(&socket_path);
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
    fn slow_clients_do_not_block_command_processing() {
        let socket_path = unique_test_socket_path();
        let receiver = start_listener_at(&socket_path).expect("listener should start");

        let _slow_client = UnixStream::connect(&socket_path).expect("slow client should connect");

        send_command_to_path(&socket_path, IpcCommand::Quit)
            .expect("second client command should still be delivered");

        let received = receiver
            .recv_timeout(Duration::from_millis(700))
            .expect("listener should process command even when another client is idle");
        assert_eq!(received, IpcCommand::Quit);

        cleanup_socket_path(&socket_path);
    }
}
