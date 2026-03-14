use super::{TrayController, TrayService};
use crate::core::app_command::AppCommand;
use crate::core::preferences::AppPreferences;
use std::sync::mpsc::Sender;

pub(super) fn start(
    _command_tx: Sender<AppCommand>,
    _preferences: AppPreferences,
) -> Result<(TrayService, TrayController), Box<dyn std::error::Error>> {
    let (sender, _receiver) = std::sync::mpsc::channel();
    Ok((TrayService::detached(), TrayController { sender }))
}
