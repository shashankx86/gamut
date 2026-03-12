mod constants;
mod launcher;
mod layout;
mod preferences;
mod styles;
mod surface;
mod theme;
mod view;

use iced::Theme;
use iced::window;
use iced_layershell::daemon;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use launcher::Launcher;
use lucide_icons::LUCIDE_FONT_BYTES;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use crate::core::app_command::AppCommand;
use styles::launcher_base_style;

type DynError = Box<dyn Error>;

pub fn run_daemon(command_rx: Receiver<AppCommand>) -> Result<(), DynError> {
    let command_rx = Arc::new(Mutex::new(Some(command_rx)));

    daemon(
        {
            let command_rx = Arc::clone(&command_rx);
            move || {
                let receiver = command_rx
                    .lock()
                    .expect("app command receiver poisoned")
                    .take()
                    .expect("launcher boot called more than once");
                Launcher::new(receiver)
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

pub fn run_preferences() -> Result<(), DynError> {
    preferences::run()
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
