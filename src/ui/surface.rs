use super::layout::LauncherLayout;
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

const NAMESPACE: &str = "gamut-launcher";

pub(super) fn namespace() -> String {
    NAMESPACE.to_string()
}

pub(super) fn launcher_visible_surface_settings(
    layout: &LauncherLayout,
    results_visible: bool,
    output_name: Option<&str>,
) -> NewLayerShellSettings {
    let (width, height) = if results_visible {
        layout.expanded_surface_size()
    } else {
        layout.collapsed_surface_size()
    };

    launcher_surface_settings_for(width, height, layout.top_margin, output_name)
}

fn launcher_surface_settings_for(
    width: u32,
    height: u32,
    top_margin: i32,
    output_name: Option<&str>,
) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some((width, height)),
        layer: Layer::Overlay,
        anchor: Anchor::Top,
        exclusive_zone: None,
        margin: Some((top_margin, 0, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        output_option: output_name
            .map(|name| OutputOption::OutputName(name.to_string()))
            .unwrap_or(OutputOption::LastOutput),
        events_transparent: false,
        namespace: Some(namespace()),
        ..NewLayerShellSettings::default()
    }
}
