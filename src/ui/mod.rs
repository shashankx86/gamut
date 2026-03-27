mod constants;
mod format;
mod launcher;
mod layout;
mod styles;
mod surface;
pub(crate) mod theme;
mod view;

use iced::Theme;
use iced::window;
use iced_layershell::daemon;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use launcher::Launcher;
use lucide_icons::LUCIDE_FONT_BYTES;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use crate::core::app_command::AppCommand;
use crate::core::error::DynError;
use crate::core::tray::TrayController;
use styles::launcher_base_style;

pub fn run_daemon(
    command_rx: Receiver<AppCommand>,
    tray_controller: TrayController,
) -> Result<(), DynError> {
    let command_rx = Arc::new(Mutex::new(Some(command_rx)));
    let tray_controller = Arc::new(Mutex::new(Some(tray_controller)));

    daemon(
        {
            let command_rx = Arc::clone(&command_rx);
            let tray_controller = Arc::clone(&tray_controller);
            move || {
                let receiver = command_rx
                    .lock()
                    .expect("app command receiver poisoned")
                    .take()
                    .expect("launcher boot called more than once");
                let tray_controller = tray_controller
                    .lock()
                    .expect("tray controller poisoned")
                    .take()
                    .expect("launcher boot called more than once");
                Launcher::new(receiver, tray_controller)
            }
        },
        surface::namespace,
        Launcher::update,
        Launcher::view,
    )
    .title(launcher_title)
    .subscription(Launcher::subscription)
    .theme(launcher_theme)
    .style(launcher_base_style)
    .settings(Settings {
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        layer_settings: daemon_layer_settings(),
        ..Settings::default()
    })
    .run()
    .map_err(|error| Box::new(error) as DynError)
}
fn launcher_theme(_state: &Launcher, _id: window::Id) -> Theme {
    _state.window_theme_for(_id)
}

fn launcher_title(state: &Launcher, id: window::Id) -> Option<String> {
    state.window_title(id)
}

fn daemon_layer_settings() -> LayerShellSettings {
    LayerShellSettings {
        start_mode: StartMode::Background,
        ..LayerShellSettings::default()
    }
}
