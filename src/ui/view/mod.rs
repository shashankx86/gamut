mod actions;
mod bottom_strip;
mod progress;
mod results;
mod search;
mod utils;

use super::launcher::{Launcher, Message};
use super::styles::{
    action_card_style, backdrop_style, bottom_strip_style, calculator_badge_style,
    calculator_card_style, panel_style, result_button_style, results_scroll_style,
    search_input_style,
};
use crate::core::desktop::{DesktopApp, trim_label};
use iced::widget::{
    Space, button, column, container, float, image, keyed_column, opaque, row, rule, scrollable,
    space, stack, svg, text, tooltip,
};
use iced::{Element, Length, Padding, window};
use iced_shadcn::{
    ButtonProps, ButtonRadius, ButtonSize, ButtonVariant, Palette as ShadcnPalette,
    Theme as ShadcnTheme, icon_button,
};
use lucide_icons::iced::{
    icon_chevron_down, icon_corner_down_left, icon_ellipsis_vertical, icon_external_link,
    icon_folder_open, icon_search,
};

use utils::{normalize_result_display_value, number_text_for_value, truncate_middle_with_ellipsis};

const BOTTOM_STRIP_ICON_BUTTON_SIZE: f32 = 20.0;
const BOTTOM_STRIP_ICON_SIZE: f32 = 12.0;
const RESULT_META_LABEL_WIDTH: f32 = 96.0;
const RESULT_META_TEXT_MIN_SIZE: f32 = 10.0;
const ACTION_CARD_MIN_WIDTH: f32 = 260.0;
const CALC_MIN_HEADLINE_CHARS: usize = 18;
const CALC_MAX_HEADLINE_CHARS: usize = 56;
const CALC_MIN_BADGE_CHARS: usize = 20;
const CALC_MAX_BADGE_CHARS: usize = 72;
const ACTION_OVERLAY_RIGHT_OFFSET: f32 = 19.0;
const ACTION_OVERLAY_BOTTOM_OFFSET: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomStripAction {
    Expand,
    Launch,
    ToggleActions,
}

impl Launcher {
    pub(super) fn view(&self, window: window::Id) -> Element<'_, Message> {
        let _ = window;
        let appearance = self.resolved_appearance();

        if !self.is_visible {
            return container(space::horizontal())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| backdrop_style())
                .into();
        }

        let mut content = column![self.view_search_header()]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Shrink);

        if self.results_progress() > 0.0 {
            content = content.push(self.view_results_section(self.results_progress()));
        }

        if self.should_render_progress_line() {
            content = content.push(self.view_progress_line());
        }

        content = content.push(self.view_bottom_strip());

        let launcher_panel = container(content)
            .padding(0)
            .style(move |_| panel_style(&self.layout, &appearance))
            .width(Length::Fill)
            .max_width(self.layout.panel_width as u32)
            .height(Length::Shrink);

        let root_panel: Element<'_, Message> = if self.is_expanded() {
            stack![launcher_panel, self.view_action_layer()]
                .width(Length::Fill)
                .height(Length::Shrink)
                .into()
        } else {
            launcher_panel.into()
        };

        container(root_panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([
                self.layout.outer_padding_y as u16,
                self.layout.outer_padding_x as u16,
            ])
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Top)
            .style(|_| backdrop_style())
            .into()
    }
}

fn bottom_strip_shadcn_theme(appearance: &super::theme::ResolvedAppearance) -> ShadcnTheme {
    ShadcnTheme::with_palette(ShadcnPalette {
        background: appearance.panel_background,
        foreground: appearance.muted_text,
        card: appearance.panel_background,
        card_foreground: appearance.primary_text,
        popover: appearance.panel_background,
        popover_foreground: appearance.primary_text,
        border: appearance.panel_border,
        input: appearance.panel_border,
        ring: appearance.accent,
        primary: appearance.primary_text,
        primary_foreground: appearance.panel_background,
        secondary: appearance.first_row_active,
        secondary_foreground: appearance.primary_text,
        accent: appearance.first_row_hover,
        accent_foreground: appearance.primary_text,
        muted: appearance.first_row_active,
        muted_foreground: appearance.muted_text,
        destructive: appearance.accent_strong,
        destructive_foreground: appearance.panel_background,
        chart_1: appearance.accent,
        chart_2: appearance.first_row_hover,
        chart_3: appearance.first_row_pressed,
        chart_4: appearance.row_hover,
        chart_5: appearance.row_pressed,
        sidebar: appearance.panel_background,
        sidebar_foreground: appearance.primary_text,
        sidebar_primary: appearance.primary_text,
        sidebar_primary_foreground: appearance.panel_background,
        sidebar_accent: appearance.first_row_hover,
        sidebar_accent_foreground: appearance.primary_text,
        sidebar_border: appearance.panel_border,
        sidebar_ring: appearance.accent,
    })
}

fn action_icon_button<'a>(
    icon: impl Into<Element<'a, Message>>,
    theme: &ShadcnTheme,
    action: BottomStripAction,
) -> iced::widget::button::Button<'a, Message> {
    icon_button(
        icon,
        Some(match action {
            BottomStripAction::Expand => Message::ExpandResults,
            BottomStripAction::Launch => Message::LaunchFirstMatch,
            BottomStripAction::ToggleActions => Message::ActionButtonPressed,
        }),
        ButtonProps::new()
            .variant(ButtonVariant::Outline)
            .radius(ButtonRadius::Small)
            .size(ButtonSize::Size1),
        theme,
    )
    .width(Length::Fixed(BOTTOM_STRIP_ICON_BUTTON_SIZE))
    .height(Length::Fixed(BOTTOM_STRIP_ICON_BUTTON_SIZE))
}
