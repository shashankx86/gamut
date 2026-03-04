use crate::app;
use crate::cli::{CliMode, parse_mode};
use crate::ipc::{self, IpcCommand};
use std::env;
use std::error::Error;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

type DynError = Box<dyn Error + Send + Sync>;

pub fn run() -> Result<(), DynError> {
    match parse_mode(env::args_os().skip(1)) {
        CliMode::Daemon => {
            if ipc::is_daemon_running() {
                return Err("gamut daemon is already running".into());
            }

            app::run_daemon()
        }
        CliMode::Toggle => ensure_daemon_and_send(IpcCommand::Toggle),
        CliMode::Quit => {
            ipc::send_command(IpcCommand::Quit)?;
            Ok(())
        }
    }
}

fn ensure_daemon_and_send(command: IpcCommand) -> Result<(), DynError> {
    if ipc::send_command(command).is_ok() {
        return Ok(());
    }

    let executable = env::current_exe()?;

    Command::new(executable)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    for _ in 0..40 {
        thread::sleep(Duration::from_millis(50));

        if ipc::send_command(command).is_ok() {
            return Ok(());
        }
    }

    Err("could not contact gamut daemon".into())
}
