use crate::core::cli::{CliMode, parse_mode};
use crate::core::ipc::{self, IpcCommand};
use crate::ui;
use std::env;
use std::error::Error;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

type DynError = Box<dyn Error>;
const DAEMON_START_RETRIES: usize = 40;
const DAEMON_START_DELAY: Duration = Duration::from_millis(50);

pub fn run() -> Result<(), DynError> {
    match parse_mode(env::args_os().skip(1)) {
        CliMode::Daemon => run_daemon(),
        CliMode::Toggle => ensure_daemon_and_send(IpcCommand::Toggle),
        CliMode::Quit => send_quit(),
    }
}

fn run_daemon() -> Result<(), DynError> {
    if ipc::is_daemon_running() {
        return Err("gamut daemon is already running".into());
    }

    ui::run_daemon()
}

fn send_quit() -> Result<(), DynError> {
    ipc::send_command(IpcCommand::Quit)?;
    Ok(())
}

fn ensure_daemon_and_send(command: IpcCommand) -> Result<(), DynError> {
    if ipc::send_command(command).is_ok() {
        return Ok(());
    }

    spawn_daemon_process()?;
    wait_for_daemon(command)
}

fn spawn_daemon_process() -> Result<(), DynError> {
    Command::new(env::current_exe()?)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

fn wait_for_daemon(command: IpcCommand) -> Result<(), DynError> {
    for _ in 0..DAEMON_START_RETRIES {
        thread::sleep(DAEMON_START_DELAY);
        if ipc::send_command(command).is_ok() {
            return Ok(());
        }
    }

    Err("could not contact gamut daemon".into())
}
