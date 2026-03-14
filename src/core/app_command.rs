use crate::core::display::OutputTarget;
use crate::core::ipc::IpcCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AppCommand {
    ShowLauncher { target_output: Option<OutputTarget> },
    ToggleLauncher { target_output: Option<OutputTarget> },
    OpenPreferences,
    ReloadPreferences,
    Quit,
}

impl AppCommand {
    pub(crate) fn from_ipc(command: IpcCommand) -> Option<Self> {
        match command {
            IpcCommand::Show { target_output } => Some(Self::ShowLauncher {
                target_output: target_output.map(|name| OutputTarget { name }),
            }),
            IpcCommand::Toggle { target_output } => Some(Self::ToggleLauncher {
                target_output: target_output.map(|name| OutputTarget { name }),
            }),
            IpcCommand::OpenPreferences => Some(Self::OpenPreferences),
            IpcCommand::ReloadPreferences => Some(Self::ReloadPreferences),
            IpcCommand::Quit => Some(Self::Quit),
            IpcCommand::Ping => None,
        }
    }
}
