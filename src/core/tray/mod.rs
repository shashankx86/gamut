mod icon;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use crate::core::app_command::AppCommand;
use crate::core::preferences::AppPreferences;

type DynError = Box<dyn Error>;

#[derive(Clone)]
pub(crate) struct TrayController {
    pub(crate) sender: std::sync::mpsc::Sender<AppPreferences>,
}

impl TrayController {
    pub(crate) fn update_preferences(&self, preferences: AppPreferences) {
        let _ = self.sender.send(preferences);
    }

    #[cfg(test)]
    pub(crate) fn detached() -> Self {
        let (sender, _receiver) = std::sync::mpsc::channel();
        Self { sender }
    }
}

pub(crate) struct TrayService {
    _thread: Option<JoinHandle<()>>,
}

impl TrayService {
    #[cfg(not(target_os = "linux"))]
    fn detached() -> Self {
        Self { _thread: None }
    }

    #[cfg(target_os = "linux")]
    fn from_thread(thread: JoinHandle<()>) -> Self {
        Self {
            _thread: Some(thread),
        }
    }
}

pub(crate) fn start(
    command_tx: Sender<AppCommand>,
    preferences: AppPreferences,
) -> Result<(TrayService, TrayController), DynError> {
    #[cfg(target_os = "linux")]
    {
        linux::start(command_tx, preferences)
    }

    #[cfg(not(target_os = "linux"))]
    {
        unsupported::start(command_tx, preferences)
    }
}
