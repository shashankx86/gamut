use super::LauncherPreferences;
use crate::core::preferences::{AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference};
use iced::Size;

const REFERENCE_MONITOR_WIDTH: f32 = 1920.0;
const REFERENCE_MONITOR_HEIGHT: f32 = 1080.0;
const MIN_LAYOUT_SCALE: f32 = 0.85;
const MAX_LAYOUT_SCALE: f32 = 1.20;

const DEFAULT_PANEL_WIDTH: f32 = 860.0;
const DEFAULT_PANEL_RADIUS: f32 = 10.0;
const DEFAULT_ITEM_RADIUS: f32 = 8.0;
const DEFAULT_RESULTS_HEIGHT: f32 = 396.0;
const DEFAULT_RESULTS_ANIMATION_SPEED: f32 = 0.25;
const DEFAULT_RESULT_ROW_HEIGHT: f32 = 46.0;
const DEFAULT_RESULT_ROW_GAP: f32 = 2.0;
const DEFAULT_RESULT_ROW_INSET_Y: f32 = 1.0;
const DEFAULT_SEARCH_ICON_SIZE: f32 = 20.0;
const DEFAULT_RESULT_ICON_SIZE: f32 = 30.0;
const DEFAULT_RESULT_ICON_BOX_SIZE: f32 = 34.0;
const DEFAULT_SEARCH_ROW_HEIGHT: f32 = 55.0;
const DEFAULT_SEARCH_ROW_PADDING_X: f32 = 20.0;
const DEFAULT_SEARCH_ROW_GAP: f32 = 10.0;
const DEFAULT_SEARCH_INPUT_PADDING_Y: f32 = 11.0;
const DEFAULT_SEARCH_INPUT_FONT_SIZE: f32 = 20.0;
const DEFAULT_RESULT_PRIMARY_TEXT_SIZE: f32 = 13.0;
const DEFAULT_RESULT_SECONDARY_TEXT_SIZE: f32 = 11.0;
const DEFAULT_EMPTY_STATE_TEXT_SIZE: f32 = 14.0;
const DEFAULT_RESULT_LABEL_MAX_LEN: usize = 56;
const DEFAULT_RESULTS_TOP_BOTTOM_PADDING: f32 = 4.0;
const DEFAULT_RESULTS_SIDE_PADDING: f32 = 8.0;
const DEFAULT_BOTTOM_STRIP_HEIGHT: f32 = 36.0;
const DEFAULT_BOTTOM_STRIP_PADDING_X: f32 = 8.0;
const DEFAULT_BOTTOM_STRIP_LABEL_GAP: f32 = 6.0;
const DEFAULT_LOGO_WIDTH: f32 = 66.0;
const DEFAULT_LOGO_HEIGHT: f32 = 14.0;
const DEFAULT_ACTION_ICON_SIZE: f32 = 22.0;
const DEFAULT_DIVIDER_HEIGHT: f32 = 1.0;
const DEFAULT_OUTER_PADDING_X: f32 = 24.0;
const DEFAULT_OUTER_PADDING_Y: f32 = 8.0;
const DEFAULT_SURFACE_TOP_MARGIN: f32 = 120.0;
const DEFAULT_HIDDEN_SURFACE_HEIGHT: u32 = 1;

pub(in crate::ui) const PANEL_WIDTH_RANGE: (f32, f32) = (640.0, 1280.0);
pub(in crate::ui) const TOP_MARGIN_RANGE: (f32, f32) = (48.0, 240.0);
pub(in crate::ui) const RESULTS_HEIGHT_RANGE: (f32, f32) = (180.0, 520.0);
pub(in crate::ui) const ANIMATION_SPEED_RANGE: (f32, f32) = (0.05, 0.80);

#[derive(Debug, Clone, PartialEq)]
pub(in crate::ui) struct LauncherLayout {
    pub(in crate::ui) panel_width: f32,
    pub(in crate::ui) panel_radius: f32,
    pub(in crate::ui) item_radius: f32,
    pub(in crate::ui) top_margin: i32,
    pub(in crate::ui) outer_padding_x: f32,
    pub(in crate::ui) outer_padding_y: f32,
    pub(in crate::ui) search_row_height: f32,
    pub(in crate::ui) search_row_padding_x: f32,
    pub(in crate::ui) search_row_gap: f32,
    pub(in crate::ui) search_input_padding_y: f32,
    pub(in crate::ui) search_input_font_size: f32,
    pub(in crate::ui) search_icon_size: f32,
    pub(in crate::ui) results_height: f32,
    pub(in crate::ui) results_top_bottom_padding: f32,
    pub(in crate::ui) results_side_padding: f32,
    pub(in crate::ui) result_row_height: f32,
    pub(in crate::ui) result_row_gap: f32,
    pub(in crate::ui) result_row_inset_y: f32,
    pub(in crate::ui) result_primary_text_size: f32,
    pub(in crate::ui) result_secondary_text_size: f32,
    pub(in crate::ui) empty_state_text_size: f32,
    pub(in crate::ui) result_label_max_len: usize,
    pub(in crate::ui) result_icon_size: f32,
    pub(in crate::ui) result_icon_box_size: f32,
    pub(in crate::ui) bottom_strip_height: f32,
    pub(in crate::ui) bottom_strip_padding_x: f32,
    pub(in crate::ui) bottom_strip_label_gap: f32,
    pub(in crate::ui) logo_width: f32,
    pub(in crate::ui) logo_height: f32,
    pub(in crate::ui) action_icon_size: f32,
    pub(in crate::ui) divider_height: f32,
    pub(in crate::ui) results_animation_speed: f32,
    pub(in crate::ui) hidden_surface_height: u32,
}

impl LauncherLayout {
    #[cfg(test)]
    pub(in crate::ui) fn fallback() -> Self {
        Self::from_monitor_size(
            None,
            &LauncherPreferences::default(),
            &AppPreferences::default(),
        )
    }

    pub(in crate::ui) fn from_monitor_size(
        monitor_size: Option<Size>,
        preferences: &LauncherPreferences,
        app_preferences: &AppPreferences,
    ) -> Self {
        let scale = layout_scale_for(monitor_size) * size_scale(app_preferences.layout.size);
        let mut layout = Self {
            panel_width: scaled(DEFAULT_PANEL_WIDTH, scale),
            panel_radius: scaled(DEFAULT_PANEL_RADIUS, scale),
            item_radius: scaled(DEFAULT_ITEM_RADIUS, scale),
            top_margin: round_to_i32(scaled(DEFAULT_SURFACE_TOP_MARGIN, scale)),
            outer_padding_x: scaled(DEFAULT_OUTER_PADDING_X, scale),
            outer_padding_y: scaled(DEFAULT_OUTER_PADDING_Y, scale),
            search_row_height: scaled(DEFAULT_SEARCH_ROW_HEIGHT, scale),
            search_row_padding_x: scaled(DEFAULT_SEARCH_ROW_PADDING_X, scale),
            search_row_gap: scaled(DEFAULT_SEARCH_ROW_GAP, scale),
            search_input_padding_y: scaled(DEFAULT_SEARCH_INPUT_PADDING_Y, scale),
            search_input_font_size: scaled(DEFAULT_SEARCH_INPUT_FONT_SIZE, scale),
            search_icon_size: scaled(DEFAULT_SEARCH_ICON_SIZE, scale),
            results_height: scaled(DEFAULT_RESULTS_HEIGHT, scale),
            results_top_bottom_padding: scaled(DEFAULT_RESULTS_TOP_BOTTOM_PADDING, scale),
            results_side_padding: scaled(DEFAULT_RESULTS_SIDE_PADDING, scale),
            result_row_height: scaled(DEFAULT_RESULT_ROW_HEIGHT, scale),
            result_row_gap: scaled(DEFAULT_RESULT_ROW_GAP, scale),
            result_row_inset_y: scaled(DEFAULT_RESULT_ROW_INSET_Y, scale).max(1.0),
            result_primary_text_size: scaled(DEFAULT_RESULT_PRIMARY_TEXT_SIZE, scale),
            result_secondary_text_size: scaled(DEFAULT_RESULT_SECONDARY_TEXT_SIZE, scale),
            empty_state_text_size: scaled(DEFAULT_EMPTY_STATE_TEXT_SIZE, scale),
            result_label_max_len: scaled_usize(DEFAULT_RESULT_LABEL_MAX_LEN, scale),
            result_icon_size: scaled(DEFAULT_RESULT_ICON_SIZE, scale),
            result_icon_box_size: scaled(DEFAULT_RESULT_ICON_BOX_SIZE, scale),
            bottom_strip_height: scaled(DEFAULT_BOTTOM_STRIP_HEIGHT, scale),
            bottom_strip_padding_x: scaled(DEFAULT_BOTTOM_STRIP_PADDING_X, scale),
            bottom_strip_label_gap: scaled(DEFAULT_BOTTOM_STRIP_LABEL_GAP, scale),
            logo_width: scaled(DEFAULT_LOGO_WIDTH, scale),
            logo_height: scaled(DEFAULT_LOGO_HEIGHT, scale),
            action_icon_size: scaled(DEFAULT_ACTION_ICON_SIZE, scale),
            divider_height: DEFAULT_DIVIDER_HEIGHT,
            results_animation_speed: DEFAULT_RESULTS_ANIMATION_SPEED,
            hidden_surface_height: DEFAULT_HIDDEN_SURFACE_HEIGHT,
        };

        if let Some(panel_width) = preferences.panel_width {
            layout.panel_width = panel_width;
        }

        if let Some(top_margin) = preferences.top_margin {
            layout.top_margin = round_to_i32(top_margin);
        }

        if let Some(results_height) = preferences.results_height {
            layout.results_height = results_height;
        }

        if let Some(animation_speed) = preferences.animation_speed {
            layout.results_animation_speed = animation_speed;
        }

        apply_radius_preferences(&mut layout, app_preferences);
        apply_placement_preferences(&mut layout, monitor_size, preferences, app_preferences);

        layout
    }

    pub(in crate::ui) fn collapsed_surface_size(&self) -> (u32, u32) {
        (
            round_to_u32(self.panel_width),
            self.collapsed_surface_height(),
        )
    }

    pub(in crate::ui) fn expanded_surface_size(&self) -> (u32, u32) {
        (
            round_to_u32(self.panel_width),
            self.expanded_surface_height(),
        )
    }

    pub(in crate::ui) fn collapsed_surface_height(&self) -> u32 {
        round_to_u32(
            self.search_row_height
                + self.divider_height
                + self.bottom_strip_height
                + (self.outer_padding_y * 2.0),
        )
    }

    pub(in crate::ui) fn expanded_surface_height(&self) -> u32 {
        self.collapsed_surface_height() + round_to_u32(self.results_height)
    }

    pub(in crate::ui) fn result_row_scroll_step(&self) -> f32 {
        self.result_row_height + self.result_row_gap
    }

    pub(in crate::ui) fn result_row_button_height(&self) -> f32 {
        (self.result_row_height - (self.result_row_inset_y * 2.0)).max(1.0)
    }

    pub(in crate::ui) fn results_viewport_height(&self) -> f32 {
        (self.results_height - (self.results_top_bottom_padding * 2.0)).max(0.0)
    }

    #[cfg(test)]
    pub(in crate::ui) fn visible_result_rows(&self) -> usize {
        ((self.results_viewport_height() / self.result_row_scroll_step()).ceil() as usize).max(1)
    }

    pub(in crate::ui) fn should_recreate_surface(&self, previous: &Self) -> bool {
        self.top_margin != previous.top_margin
    }
}

fn size_scale(size: LauncherSize) -> f32 {
    match size {
        LauncherSize::Small => 0.88,
        LauncherSize::Medium => 1.0,
        LauncherSize::Large => 1.12,
    }
}

fn apply_radius_preferences(layout: &mut LauncherLayout, app_preferences: &AppPreferences) {
    let panel_radius = match app_preferences.appearance.radius {
        RadiusPreference::None => 0.0,
        RadiusPreference::Small => DEFAULT_PANEL_RADIUS,
        RadiusPreference::Medium => 16.0,
        RadiusPreference::Large => 22.0,
    };
    let item_radius = match app_preferences.appearance.radius {
        RadiusPreference::None => 0.0,
        RadiusPreference::Small => DEFAULT_ITEM_RADIUS,
        RadiusPreference::Medium => 12.0,
        RadiusPreference::Large => 16.0,
    };

    layout.panel_radius = panel_radius;
    layout.item_radius = item_radius;
}

fn apply_placement_preferences(
    layout: &mut LauncherLayout,
    monitor_size: Option<Size>,
    preferences: &LauncherPreferences,
    app_preferences: &AppPreferences,
) {
    if preferences.top_margin.is_some() {
        return;
    }

    layout.top_margin = match app_preferences.layout.placement {
        LauncherPlacement::RaisedCenter => layout.top_margin,
        LauncherPlacement::Custom => round_to_i32(app_preferences.layout.custom_top_margin),
        LauncherPlacement::Center => centered_top_margin(layout, monitor_size),
    };
}

fn centered_top_margin(layout: &LauncherLayout, monitor_size: Option<Size>) -> i32 {
    let Some(size) = monitor_size else {
        return layout.top_margin;
    };

    let centered = ((size.height - layout.expanded_surface_height() as f32) / 2.0).max(0.0);
    round_to_i32(centered)
}

fn layout_scale_for(monitor_size: Option<Size>) -> f32 {
    let Some(size) = monitor_size else {
        return 1.0;
    };

    if !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        return 1.0;
    }

    let width_scale = size.width / REFERENCE_MONITOR_WIDTH;
    let height_scale = size.height / REFERENCE_MONITOR_HEIGHT;

    width_scale
        .min(height_scale)
        .clamp(MIN_LAYOUT_SCALE, MAX_LAYOUT_SCALE)
}

fn scaled(value: f32, scale: f32) -> f32 {
    (value * scale).round()
}

fn scaled_usize(value: usize, scale: f32) -> usize {
    ((value as f32) * scale).round().max(1.0) as usize
}

fn round_to_u32(value: f32) -> u32 {
    value.round().max(1.0) as u32
}

fn round_to_i32(value: f32) -> i32 {
    value.round() as i32
}
