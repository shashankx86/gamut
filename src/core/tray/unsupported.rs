use super::TrayService;
use crate::core::app_command::AppCommand;
use std::sync::mpsc::Sender;

pub(super) fn start(
    _command_tx: Sender<AppCommand>,
) -> Result<TrayService, Box<dyn std::error::Error>> {
    Ok(TrayService::detached())
}
