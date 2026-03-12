use super::{DAEMON_START_DELAY, DAEMON_START_RETRIES, DynError};
use crate::core::ipc::{self, IpcCommand};
use crate::core::tray;
use crate::ui;
use log::{debug, info, warn};
use std::env;
use std::process::{Child, Command, Stdio};
use std::thread;

pub(super) fn run_daemon() -> Result<(), DynError> {
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

pub(super) fn send_quit() -> Result<(), DynError> {
    info!("sending quit command to daemon");
    ipc::send_command(IpcCommand::Quit)?;
    Ok(())
}

pub(super) fn ensure_daemon_and_send(command: IpcCommand) -> Result<(), DynError> {
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

#[cfg(test)]
mod tests {
    use crate::core::ipc::IpcCommand;

    #[test]
    fn cloned_ipc_command_preserves_target_output() {
        let command = IpcCommand::Toggle {
            target_output: Some("DP-1".to_string()),
        };

        assert_eq!(command.clone(), command);
    }
}
