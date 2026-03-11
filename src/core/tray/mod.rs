mod icon;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use crate::core::app_command::AppCommand;

type DynError = Box<dyn Error>;

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

pub(crate) fn start(command_tx: Sender<AppCommand>) -> Result<TrayService, DynError> {
    #[cfg(target_os = "linux")]
    {
        linux::start(command_tx)
    }

    #[cfg(not(target_os = "linux"))]
    {
        unsupported::start(command_tx)
    }
}
