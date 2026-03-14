use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputTarget {
    pub(crate) name: String,
}

pub(crate) fn active_output_target() -> Option<OutputTarget> {
    active_output_name().map(|name| OutputTarget { name })
}

fn active_output_name() -> Option<String> {
    active_output_name_via_niri()
        .or_else(active_output_name_via_hyprland)
        .or_else(active_output_name_via_sway)
        .or_else(active_output_name_via_kwin)
}

fn active_output_name_via_niri() -> Option<String> {
    let workspaces: Vec<NiriWorkspace> = command_json("niri", &["msg", "-j", "workspaces"])?;
    let focused = workspaces
        .into_iter()
        .find(|workspace| workspace.is_focused)?;
    (!focused.output.is_empty()).then_some(focused.output)
}

fn active_output_name_via_hyprland() -> Option<String> {
    let monitors: Vec<HyprlandMonitor> = command_json("hyprctl", &["monitors", "-j"])?;
    let focused = monitors.into_iter().find(|monitor| monitor.focused)?;
    (!focused.name.is_empty()).then_some(focused.name)
}

fn active_output_name_via_sway() -> Option<String> {
    let workspaces: Vec<SwayWorkspace> =
        command_json("swaymsg", &["-t", "get_workspaces", "--raw"])?;
    let focused = workspaces.into_iter().find(|workspace| workspace.focused)?;
    (!focused.output.is_empty()).then_some(focused.output)
}

fn active_output_name_via_kwin() -> Option<String> {
    if std::env::var_os("KDE_FULL_SESSION").is_none()
        && std::env::var("XDG_CURRENT_DESKTOP")
            .ok()
            .is_none_or(|desktop| !desktop.contains("KDE"))
    {
        return None;
    }

    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.kde.KWin",
            "--object-path",
            "/KWin",
            "--method",
            "org.kde.KWin.activeOutputName",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_gdbus_single_string(&String::from_utf8_lossy(&output.stdout))
}

fn command_json<T>(program: &str, args: &[&str]) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let output = Command::new(program).args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

fn parse_gdbus_single_string(output: &str) -> Option<String> {
    let trimmed = output.trim();
    let value = trimmed.strip_prefix("('")?.strip_suffix("',)")?;
    (!value.is_empty()).then(|| value.to_string())
}

#[derive(Debug, Deserialize)]
struct NiriWorkspace {
    is_focused: bool,
    output: String,
}

#[derive(Debug, Deserialize)]
struct HyprlandMonitor {
    focused: bool,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SwayWorkspace {
    focused: bool,
    output: String,
}

#[cfg(test)]
mod tests {
    use super::parse_gdbus_single_string;

    #[test]
    fn parses_gdbus_single_string_reply() {
        assert_eq!(
            parse_gdbus_single_string("('HDMI-A-1',)\n"),
            Some("HDMI-A-1".to_string())
        );
    }
}
