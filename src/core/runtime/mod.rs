use crate::core::cli::{CliMode, parse_mode};
use crate::core::ipc::{self, IpcCommand};
use crate::core::tray;
use crate::ui;
use std::env;
use std::error::Error;
use std::process::{Child, Command, Stdio};
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

    let _tray_service = tray::start()?;
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

    let mut daemon_child = spawn_daemon_process()?;
    wait_for_daemon(command, &mut daemon_child)
}

fn spawn_daemon_process() -> Result<Child, DynError> {
    let child = Command::new(env::current_exe()?)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child)
}

fn wait_for_daemon(command: IpcCommand, daemon_child: &mut Child) -> Result<(), DynError> {
    let mut last_error: Option<String> = None;

    for _ in 0..DAEMON_START_RETRIES {
        thread::sleep(DAEMON_START_DELAY);

        match ipc::send_command(command) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error.to_string()),
        }

        if let Some(status) = daemon_child.try_wait()? {
            return Err(
                format!("gamut daemon exited before becoming ready (status: {status})").into(),
            );
        }
    }

    let detail = last_error.unwrap_or_else(|| "no socket response".to_string());
    Err(
        format!("could not contact gamut daemon after {DAEMON_START_RETRIES} attempts: {detail}")
            .into(),
    )
}
