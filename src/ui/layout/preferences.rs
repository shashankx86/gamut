use super::metrics::{
    ANIMATION_SPEED_RANGE, PANEL_WIDTH_RANGE, RESULTS_HEIGHT_RANGE, TOP_MARGIN_RANGE,
};
use std::env;

const ENV_PANEL_WIDTH: &str = "GAMUT_LAUNCHER_WIDTH";
const ENV_TOP_MARGIN: &str = "GAMUT_LAUNCHER_TOP_MARGIN";
const ENV_RESULTS_HEIGHT: &str = "GAMUT_LAUNCHER_RESULTS_HEIGHT";
const ENV_ANIMATION_SPEED: &str = "GAMUT_LAUNCHER_ANIMATION_SPEED";

#[derive(Debug, Clone, PartialEq, Default)]
pub(in crate::ui) struct LauncherPreferences {
    pub(in crate::ui) panel_width: Option<f32>,
    pub(in crate::ui) top_margin: Option<f32>,
    pub(in crate::ui) results_height: Option<f32>,
    pub(in crate::ui) animation_speed: Option<f32>,
}

impl LauncherPreferences {
    pub(in crate::ui) fn load_from_env() -> Self {
        Self {
            panel_width: parse_env_f32(ENV_PANEL_WIDTH, PANEL_WIDTH_RANGE),
            top_margin: parse_env_f32(ENV_TOP_MARGIN, TOP_MARGIN_RANGE),
            results_height: parse_env_f32(ENV_RESULTS_HEIGHT, RESULTS_HEIGHT_RANGE),
            animation_speed: parse_env_f32(ENV_ANIMATION_SPEED, ANIMATION_SPEED_RANGE),
        }
    }
}

fn parse_env_f32(name: &str, range: (f32, f32)) -> Option<f32> {
    let raw = env::var(name).ok()?;
    let value = raw.trim().parse::<f32>().ok()?;

    if !value.is_finite() {
        return None;
    }

    Some(value.clamp(range.0, range.1))
}
