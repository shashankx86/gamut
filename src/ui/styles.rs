use super::constants::{ITEM_RADIUS, PANEL_RADIUS};
use super::launcher::{Launcher, Message};
use iced::widget::{button, container, scrollable, text, text_input};
use iced::{Background, Border, Color, Element, Shadow, Theme};

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
        background: Some(Background::Color(Color::from_rgba(0.12, 0.12, 0.13, 0.98))),
        border: Border {
            color: Color::from_rgba(0.51, 0.53, 0.58, 0.50),
            width: 1.0,
            radius: PANEL_RADIUS.into(),
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

pub(super) fn header_style() -> container::Style {
    container::Style {
        border: Border {
            color: Color::from_rgba(0.30, 0.32, 0.36, 0.70),
            width: 1.0,
            radius: iced::border::Radius::default().top(PANEL_RADIUS),
        },
        background: Some(Background::Color(Color::from_rgba(0.13, 0.13, 0.14, 0.98))),
        ..container::Style::default()
    }
}

pub(super) fn footer_style() -> container::Style {
    container::Style {
        border: Border {
            color: Color::from_rgba(0.30, 0.32, 0.36, 0.70),
            width: 1.0,
            radius: iced::border::Radius::default().bottom(PANEL_RADIUS),
        },
        background: Some(Background::Color(Color::from_rgba(0.13, 0.13, 0.14, 0.99))),
        ..container::Style::default()
    }
}

pub(super) fn app_icon_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.19, 0.22, 0.27))),
        border: Border {
            color: Color::from_rgba(0.44, 0.52, 0.64, 0.65),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

pub(super) fn footer_key_chip(label: &'static str) -> Element<'static, Message> {
    container(
        text(label)
            .size(11)
            .color(Color::from_rgb(0.84, 0.86, 0.89)),
    )
    .padding([2, 6])
    .style(|_| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.23, 0.24, 0.27))),
        border: Border {
            color: Color::from_rgba(0.55, 0.57, 0.62, 0.65),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

pub(super) fn search_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => Color::from_rgb(0.55, 0.58, 0.64),
        text_input::Status::Hovered => Color::from_rgb(0.42, 0.45, 0.50),
        _ => Color::from_rgb(0.32, 0.34, 0.38),
    };

    text_input::Style {
        background: Background::Color(Color::from_rgba(0.14, 0.14, 0.15, 0.98)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: Color::from_rgb(0.56, 0.58, 0.63),
        placeholder: Color::from_rgb(0.50, 0.52, 0.56),
        value: Color::from_rgb(0.96, 0.96, 0.97),
        selection: Color::from_rgb(0.29, 0.33, 0.42),
    }
}

pub(super) fn result_button_style(status: button::Status, first_row: bool) -> button::Style {
    let active_bg = if first_row {
        Color::from_rgba(0.24, 0.24, 0.25, 0.96)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.0)
    };

    let hover_bg = Color::from_rgba(0.22, 0.23, 0.25, 0.96);

    let background = match status {
        button::Status::Hovered => hover_bg,
        button::Status::Pressed => Color::from_rgba(0.26, 0.27, 0.29, 0.96),
        _ => active_bg,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb(0.94, 0.95, 0.96),
        border: Border {
            color: Color::from_rgba(0.39, 0.41, 0.45, 0.42),
            width: if first_row { 1.0 } else { 0.0 },
            radius: ITEM_RADIUS.into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn results_scroll_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);

    let rail = scrollable::Rail {
        background: Some(Background::Color(Color::from_rgba(0.18, 0.18, 0.19, 0.66))),
        border: Border {
            color: Color::from_rgba(0.36, 0.36, 0.40, 0.40),
            width: 1.0,
            radius: 10.0.into(),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(Color::from_rgba(0.52, 0.54, 0.59, 0.90)),
            border: Border {
                color: Color::from_rgba(0.65, 0.67, 0.73, 0.55),
                width: 1.0,
                radius: 10.0.into(),
            },
        },
    };

    style.container = container::Style::default();
    style.vertical_rail = rail;
    style.horizontal_rail = rail;
    style.gap = None;
    style
}
