use super::constants::{ITEM_RADIUS, PANEL_RADIUS};
use super::launcher::Launcher;
use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme};

pub(super) fn launcher_base_style(_state: &Launcher, _theme: &Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::from_rgb(0.92, 0.93, 0.95),
    }
}

pub(super) fn backdrop_style() -> container::Style {
    container::Style::default()
}

pub(super) fn panel_style() -> container::Style {
    container::Style {
        text_color: Some(Color::from_rgb(0.92, 0.93, 0.95)),
        background: Some(Background::Color(Color::from_rgba(
            21.0 / 255.0,
            21.0 / 255.0,
            22.0 / 255.0,
            1.0,
        ))),
        border: Border {
            color: Color::from_rgba(48.0 / 255.0, 49.0 / 255.0, 52.0 / 255.0, 0.95),
            width: 1.0,
            radius: PANEL_RADIUS.into(),
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

pub(super) fn divider_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            46.0 / 255.0,
            46.0 / 255.0,
            46.0 / 255.0,
            0.86,
        ))),
        ..container::Style::default()
    }
}

pub(super) fn bottom_strip_style() -> container::Style {
    container::Style {
        text_color: Some(Color::from_rgb(131.0 / 255.0, 135.0 / 255.0, 143.0 / 255.0)),
        ..container::Style::default()
    }
}

pub(super) fn search_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let placeholder = match status {
        text_input::Status::Focused { .. } => {
            Color::from_rgb(138.0 / 255.0, 142.0 / 255.0, 149.0 / 255.0)
        }
        _ => Color::from_rgb(131.0 / 255.0, 135.0 / 255.0, 143.0 / 255.0),
    };

    text_input::Style {
        background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        icon: Color::from_rgb(114.0 / 255.0, 114.0 / 255.0, 114.0 / 255.0),
        placeholder,
        value: Color::from_rgb(205.0 / 255.0, 208.0 / 255.0, 214.0 / 255.0),
        selection: Color::from_rgba(0.31, 0.31, 0.34, 0.88),
    }
}

pub(super) fn show_more_icon_style() -> container::Style {
    container::Style::default()
}

pub(super) fn result_button_style(status: button::Status, first_row: bool) -> button::Style {
    let active_bg = if first_row {
        Color::from_rgba(32.0 / 255.0, 32.0 / 255.0, 32.0 / 255.0, 0.92)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.0)
    };

    let hover_bg = if first_row {
        Color::from_rgba(38.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0, 0.94)
    } else {
        Color::from_rgba(24.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 0.28)
    };

    let background = match status {
        button::Status::Hovered => hover_bg,
        button::Status::Pressed => {
            if first_row {
                Color::from_rgba(45.0 / 255.0, 45.0 / 255.0, 45.0 / 255.0, 0.94)
            } else {
                Color::from_rgba(32.0 / 255.0, 32.0 / 255.0, 32.0 / 255.0, 0.55)
            }
        }
        _ => active_bg,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb(0.94, 0.95, 0.96),
        border: Border {
            color: Color::from_rgba(57.0 / 255.0, 57.0 / 255.0, 57.0 / 255.0, 0.56),
            width: if first_row { 1.0 } else { 0.0 },
            radius: ITEM_RADIUS.into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn results_scroll_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    let (scroller_background, scroller_border) = match status {
        scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged: true,
            ..
        } => (
            Color::from_rgba(68.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.90),
            Color::from_rgba(95.0 / 255.0, 95.0 / 255.0, 95.0 / 255.0, 0.62),
        ),
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } => (
            Color::from_rgba(78.0 / 255.0, 78.0 / 255.0, 78.0 / 255.0, 0.88),
            Color::from_rgba(108.0 / 255.0, 108.0 / 255.0, 108.0 / 255.0, 0.60),
        ),
        _ => (
            Color::from_rgba(92.0 / 255.0, 92.0 / 255.0, 92.0 / 255.0, 0.84),
            Color::from_rgba(120.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 0.56),
        ),
    };

    let vertical_rail = scrollable::Rail {
        background: Some(Background::Color(Color::from_rgba(0.08, 0.08, 0.08, 0.38))),
        border: Border {
            color: Color::from_rgba(40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 0.42),
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
