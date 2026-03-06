use super::layout::LauncherLayout;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};

const NAMESPACE: &str = "gamut-launcher";

pub(super) fn namespace() -> String {
    NAMESPACE.to_string()
}

pub(super) fn launcher_visible_surface_settings(
    layout: &LauncherLayout,
    results_visible: bool,
) -> NewLayerShellSettings {
    let (width, height) = if results_visible {
        layout.expanded_surface_size()
    } else {
        layout.collapsed_surface_size()
    };

    launcher_surface_settings_for(width, height, layout.top_margin)
}

pub(super) fn launcher_hidden_surface_settings(layout: &LauncherLayout) -> NewLayerShellSettings {
    let (width, height) = layout.hidden_surface_size();
    launcher_surface_settings_for(width, height, 0)
}

fn launcher_surface_settings_for(
    width: u32,
    height: u32,
    top_margin: i32,
) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some((width, height)),
        layer: Layer::Overlay,
        anchor: Anchor::Top,
        exclusive_zone: None,
        margin: Some((top_margin, 0, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        events_transparent: false,
        namespace: Some(namespace()),
        ..NewLayerShellSettings::default()
    }
}
