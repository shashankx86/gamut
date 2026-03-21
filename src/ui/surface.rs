use super::layout::LauncherLayout;
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

const NAMESPACE: &str = "gamut-launcher";
const COMPOSITOR_SELECTED_OUTPUT_SENTINEL: &str = "gamut:auto-output";

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
        output_option: output_option_for(output_name),
        events_transparent: false,
        namespace: Some(namespace()),
        ..NewLayerShellSettings::default()
    }
}

fn output_option_for(output_name: Option<&str>) -> OutputOption {
    output_name
        .map(|name| OutputOption::OutputName(name.to_string()))
        // `layershellev` currently resolves `OutputOption::None`/`LastOutput`
        // before calling `get_layer_surface`, which bypasses compositor output
        // auto-selection. Using a name that cannot exist in protocol-valid
        // output names guarantees no match and keeps the output argument NULL.
        .unwrap_or_else(|| {
            OutputOption::OutputName(COMPOSITOR_SELECTED_OUTPUT_SENTINEL.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::{COMPOSITOR_SELECTED_OUTPUT_SENTINEL, output_option_for};
    use iced_layershell::reexport::OutputOption;

    #[test]
    fn explicit_output_name_is_preserved() {
        assert_eq!(
            output_option_for(Some("DP-1")),
            OutputOption::OutputName("DP-1".to_string())
        );
    }

    #[test]
    fn missing_output_uses_compositor_selected_sentinel() {
        assert_eq!(
            output_option_for(None),
            OutputOption::OutputName(COMPOSITOR_SELECTED_OUTPUT_SENTINEL.to_string())
        );
    }
}
