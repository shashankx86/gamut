mod constants;
mod launcher;
mod layout;
mod preferences_app;
mod styles;
mod surface;
mod theme;
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
    .title(launcher_title)
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

pub fn run_preferences() -> Result<(), DynError> {
    preferences_app::run()
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
