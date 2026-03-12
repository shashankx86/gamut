use super::launcher::Launcher;
use super::layout::LauncherLayout;
use super::theme::ResolvedAppearance;
use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme};

pub(super) fn launcher_base_style(state: &Launcher, _theme: &Theme) -> iced::theme::Style {
    let appearance = state.resolved_appearance();

    iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: appearance.primary_text,
    }
}

pub(super) fn backdrop_style() -> container::Style {
    container::Style::default()
}

pub(super) fn panel_style(
    layout: &LauncherLayout,
    appearance: &ResolvedAppearance,
) -> container::Style {
    container::Style {
        text_color: Some(appearance.primary_text),
        background: Some(Background::Color(appearance.panel_background)),
        border: Border {
            color: appearance.panel_border,
            width: 1.0,
            radius: layout.panel_radius.into(),
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

pub(super) fn divider_style(appearance: &ResolvedAppearance) -> container::Style {
    container::Style {
        background: Some(Background::Color(appearance.divider)),
        ..container::Style::default()
    }
}

pub(super) fn bottom_strip_style(appearance: &ResolvedAppearance) -> container::Style {
    container::Style {
        text_color: Some(appearance.muted_text),
        ..container::Style::default()
    }
}

pub(super) fn search_input_style(
    appearance: &ResolvedAppearance,
    status: text_input::Status,
) -> text_input::Style {
    let placeholder = match status {
        text_input::Status::Focused { .. } => appearance.secondary_text,
        _ => appearance.muted_text,
    };

    text_input::Style {
        background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        icon: appearance.search_icon,
        placeholder,
        value: appearance.primary_text,
        selection: appearance.selection,
    }
}

pub(super) fn result_button_style(
    status: button::Status,
    selected: bool,
    layout: &LauncherLayout,
    appearance: &ResolvedAppearance,
) -> button::Style {
    let active_bg = if selected {
        appearance.first_row_active
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.0)
    };

    let hover_bg = if selected {
        appearance.first_row_hover
    } else {
        appearance.row_hover
    };

    let background = match status {
        button::Status::Hovered => hover_bg,
        button::Status::Pressed => {
            if selected {
                appearance.first_row_pressed
            } else {
                appearance.row_pressed
            }
        }
        _ => active_bg,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: appearance.primary_text,
        border: Border {
            color: appearance.panel_border,
            width: if selected { 1.0 } else { 0.0 },
            radius: layout.item_radius.into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn results_scroll_style(
    theme: &Theme,
    appearance: &ResolvedAppearance,
    status: scrollable::Status,
) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    let (scroller_background, scroller_border) = match status {
        scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged: true,
            ..
        } => (
            appearance.scrollbar_scroller_dragged,
            appearance.scrollbar_scroller_dragged_border,
        ),
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } => (
            appearance.scrollbar_scroller_hover,
            appearance.scrollbar_scroller_hover_border,
        ),
        _ => (
            appearance.scrollbar_scroller,
            appearance.scrollbar_scroller_border,
        ),
    };

    let vertical_rail = scrollable::Rail {
        background: Some(Background::Color(appearance.scrollbar_track)),
        border: Border {
            color: appearance.scrollbar_track_border,
            width: 1.0,
            radius: 10.0.into(),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_background),
            border: Border {
                color: scroller_border,
                width: 1.0,
                radius: 10.0.into(),
            },
        },
    };

    let horizontal_rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
        },
    };

    style.container = container::Style::default();
    style.vertical_rail = vertical_rail;
    style.horizontal_rail = horizontal_rail;
    style.gap = None;
    style
}
