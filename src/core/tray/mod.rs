mod icon;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

use std::error::Error;
use std::thread::JoinHandle;

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

pub(crate) fn start() -> Result<TrayService, DynError> {
    #[cfg(target_os = "linux")]
    {
        linux::start()
    }

    #[cfg(not(target_os = "linux"))]
    {
        unsupported::start()
    }
}
