mod constants;
mod launcher;
mod styles;
mod view;

use constants::{SHOWN_HEIGHT, SURFACE_TOP_MARGIN};
use iced::Theme;
use iced::window;
use iced_layershell::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use launcher::Launcher;
use std::error::Error;
use styles::launcher_base_style;

type DynError = Box<dyn Error>;

pub fn run_daemon() -> Result<(), DynError> {
    daemon(Launcher::new, namespace, Launcher::update, Launcher::view)
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

fn namespace() -> String {
    "gamut-launcher".to_string()
}

fn daemon_layer_settings() -> LayerShellSettings {
    LayerShellSettings {
        start_mode: StartMode::Background,
        ..LayerShellSettings::default()
    }
}

fn launcher_surface_settings() -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some((0, SHOWN_HEIGHT)),
        layer: Layer::Overlay,
        anchor: Anchor::Top | Anchor::Left | Anchor::Right,
        exclusive_zone: None,
        margin: Some((SURFACE_TOP_MARGIN, 0, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        events_transparent: false,
        namespace: Some(namespace()),
        ..NewLayerShellSettings::default()
    }
}
