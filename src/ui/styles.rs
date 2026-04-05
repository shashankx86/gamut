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
        shadow: Shadow {
            color: Color {
                a: if appearance.panel_background.r > 0.5 {
                    0.16
                } else {
                    0.34
                },
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 18.0),
            blur_radius: 42.0,
        },
        ..container::Style::default()
    }
}

pub(super) fn bottom_strip_style(appearance: &ResolvedAppearance) -> container::Style {
    container::Style {
        text_color: Some(appearance.muted_text),
        ..container::Style::default()
    }
}

pub(super) fn action_card_style(
    layout: &LauncherLayout,
    appearance: &ResolvedAppearance,
) -> container::Style {
    container::Style {
        text_color: Some(appearance.primary_text),
        background: Some(Background::Color(Color {
            a: if appearance.panel_background.r > 0.5 {
                0.97
            } else {
                0.92
            },
            ..appearance.panel_background
        })),
        border: Border {
            color: appearance.accent_soft,
            width: 1.0,
            radius: layout.item_radius.into(),
        },
        shadow: Shadow {
            color: Color {
                a: if appearance.panel_background.r > 0.5 {
                    0.12
                } else {
                    0.3
                },
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 22.0,
        },
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
        selection: appearance.accent_soft,
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
            color: if selected {
                appearance.accent_soft
            } else {
                Color::TRANSPARENT
            },
            width: if selected { 1.0 } else { 0.0 },
            radius: layout.item_radius.into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn calculator_card_style(
    layout: &LauncherLayout,
    appearance: &ResolvedAppearance,
) -> container::Style {
    container::Style {
        text_color: Some(appearance.primary_text),
        background: Some(Background::Color(Color {
            a: if appearance.panel_background.r > 0.5 {
                0.05
            } else {
                0.1
            },
            ..appearance.panel_background
        })),
        border: Border {
            color: appearance.panel_border,
            width: 1.0,
            radius: layout.item_radius.into(),
        },
        ..container::Style::default()
    }
}

pub(super) fn calculator_badge_style(appearance: &ResolvedAppearance) -> container::Style {
    container::Style {
        text_color: Some(appearance.muted_text),
        background: Some(Background::Color(appearance.first_row_active)),
        border: Border {
            color: appearance.accent_soft,
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    }
}

pub(super) fn results_scroll_style(
    theme: &Theme,
    appearance: &ResolvedAppearance,
    show_scrollbar: bool,
    status: scrollable::Status,
) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);

    if !show_scrollbar {
        style.container = container::Style::default();
        style.vertical_rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
            },
        };
        style.horizontal_rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
            },
        };
        style.gap = None;
        return style;
    }

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

    let show_track = matches!(
        status,
        scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged: true,
            ..
        }
    );

    let vertical_rail = scrollable::Rail {
        background: if show_track {
            Some(Background::Color(Color {
                a: appearance.scrollbar_track.a * 0.52,
                ..appearance.scrollbar_track
            }))
        } else {
            None
        },
        border: Border {
            color: Color {
                a: appearance.scrollbar_track_border.a * 0.0,
                ..appearance.scrollbar_track_border
            },
            width: 0.0,
            radius: 0.0.into(),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_background),
            border: Border {
                color: Color {
                    a: scroller_border.a * 0.55,
                    ..scroller_border
                },
                width: 0.0,
                radius: 0.0.into(),
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
