use crate::core::cli::{parse_command, print_help, print_version, CliCommand, CliMode};
use crate::core::display_target::active_output_name;
use crate::core::ipc::{self, IpcCommand};
use crate::core::logging;
use crate::core::tray;
use crate::ui;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use log::{debug, info, warn};

type DynError = Box<dyn Error>;
const DAEMON_START_RETRIES: usize = 40;
const DAEMON_START_DELAY: Duration = Duration::from_millis(50);

pub fn run() -> Result<(), DynError> {
    run_with_args(env::args_os().skip(1))
}

fn run_with_args<I>(args: I) -> Result<(), DynError>
where
    I: IntoIterator<Item = OsString>,
{
    match parse_command(args) {
        CliCommand::Help => {
            print_help();
            Ok(())
        }
        CliCommand::Version => {
            print_version();
            Ok(())
        }
        CliCommand::Run(mode) => {
            logging::init();
            info!("handling {} command", mode.name());
            run_mode(mode)
        }
    }
}

fn run_mode(mode: CliMode) -> Result<(), DynError> {
    match mode {
        CliMode::Daemon => run_daemon(),
        CliMode::Toggle => ensure_daemon_and_send(IpcCommand::Toggle {
            target_output: active_output_name(),
        }),
        CliMode::Preferences => ui::run_preferences(),
        CliMode::Quit => send_quit(),
    }
}

fn run_daemon() -> Result<(), DynError> {
    info!("starting daemon services");

    if ipc::is_daemon_running() {
        warn!("daemon start requested while another daemon is already running");
        return Err("gamut daemon is already running".into());
    }

    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let _tray_service = tray::start(command_tx)?;
    info!("tray service started");
    ui::run_daemon(command_rx)
}

fn send_quit() -> Result<(), DynError> {
    info!("sending quit command to daemon");
    ipc::send_command(IpcCommand::Quit)?;
    Ok(())
}

fn ensure_daemon_and_send(command: IpcCommand) -> Result<(), DynError> {
    debug!("sending IPC command: {:?}", command);

    if ipc::send_command(command.clone()).is_ok() {
        return Ok(());
    }

    info!("daemon unavailable, starting a new background process");
    let mut daemon_child = spawn_daemon_process()?;
    wait_for_daemon(command, &mut daemon_child)
}

fn spawn_daemon_process() -> Result<Child, DynError> {
    let current_exe = env::current_exe()?;
    debug!("spawning daemon process from {}", current_exe.display());

    let child = Command::new(current_exe)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child)
}

fn wait_for_daemon(command: IpcCommand, daemon_child: &mut Child) -> Result<(), DynError> {
    let mut last_error: Option<String> = None;

    for attempt in 1..=DAEMON_START_RETRIES {
        thread::sleep(DAEMON_START_DELAY);

        match ipc::send_command(command.clone()) {
            Ok(()) => return Ok(()),
            Err(error) => {
                debug!("daemon not ready after attempt {attempt}/{DAEMON_START_RETRIES}: {error}");
                last_error = Some(error.to_string());
            }
        }

        if let Some(status) = daemon_child.try_wait()? {
            return Err(
                format!("gamut daemon exited before becoming ready (status: {status})").into(),
            );
        }
    }

    let detail = last_error.unwrap_or_else(|| "no socket response".to_string());
    warn!("could not contact daemon after {DAEMON_START_RETRIES} attempts: {detail}");
    Err(
        format!("could not contact gamut daemon after {DAEMON_START_RETRIES} attempts: {detail}")
            .into(),
    )
}
