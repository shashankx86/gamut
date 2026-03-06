mod constants;
mod launcher;
mod layout;
mod styles;
mod surface;
mod view;

use iced::Theme;
use iced::window;
use iced_layershell::daemon;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use launcher::Launcher;
use std::error::Error;
use styles::launcher_base_style;

type DynError = Box<dyn Error>;

pub fn run_daemon() -> Result<(), DynError> {
    daemon(
        Launcher::new,
        surface::namespace,
        Launcher::update,
        Launcher::view,
    )
    .subscription(Launcher::subscription)
    .theme(launcher_theme)
    .style(launcher_base_style)
    .settings(Settings {
        layer_settings: daemon_layer_settings(),
        ..Settings::default()
    })
    .run()
    .map_err(|error| Box::new(error) as DynError)
}

fn launcher_theme(_state: &Launcher, _id: window::Id) -> Theme {
    Theme::Dark
}

fn daemon_layer_settings() -> LayerShellSettings {
    LayerShellSettings {
        start_mode: StartMode::Background,
        ..LayerShellSettings::default()
    }
}
