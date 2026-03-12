use super::TrayService;
use crate::core::app_command::AppCommand;
use crate::core::preferences::AppPreferences;
use std::sync::mpsc::Sender;

pub(super) fn start(
    _command_tx: Sender<AppCommand>,
    _preferences: AppPreferences,
) -> Result<TrayService, Box<dyn std::error::Error>> {
    Ok(TrayService::detached())
}
