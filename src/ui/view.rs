use super::launcher::{Launcher, Message};
use super::styles::{
    action_card_style, backdrop_style, bottom_strip_style, calculator_badge_style,
    calculator_card_style, panel_style, result_button_style, results_scroll_style,
    search_input_style,
};
use crate::core::desktop::{DesktopApp, trim_label};
use iced::widget::{
    Space, button, column, container, float, image, keyed_column, opaque, row, rule, scrollable,
    space, stack, svg, text, text_input, tooltip,
};
use iced::{ContentFit, Element, Length, Padding, window};
use iced_shadcn::{
    ButtonProps, ButtonRadius, ButtonSize, ButtonVariant, Palette as ShadcnPalette,
    Theme as ShadcnTheme, icon_button,
};
use lucide_icons::iced::{
    icon_chevron_down, icon_corner_down_left, icon_ellipsis_vertical, icon_external_link,
    icon_folder_open, icon_search,
};

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

    fn view_search_header(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let input = text_input("Search for apps and commands...", &self.query)
            .id(self.input_id.clone())
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchFirstMatch)
            .padding([self.layout.search_input_padding_y as u16, 0])
            .size(self.layout.search_input_font_size)
            .style(move |_theme, status| search_input_style(&appearance, status))
            .width(Length::Fill);

        row![icon_search().size(self.layout.search_icon_size), input,]
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.search_row_height))
            .padding([0, self.layout.search_row_padding_x as u16])
            .spacing(self.layout.search_row_gap)
            .align_y(iced::alignment::Vertical::Center)
            .into()
    }

    fn view_results_section(&self, progress: f32) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let mut static_rows = Vec::new();
        let mut keyed_rows = Vec::new();
        let calculation_preview = self.calculation_preview();
        let filtered = self.filtered_indices();

        if let Some(preview) = calculation_preview {
            static_rows.push(
                container(
                    text("Calculator")
                        .size(self.layout.result_secondary_text_size)
                        .color(appearance.muted_text),
                )
                .width(Length::Fill)
                .padding([0, 4])
                .into(),
            );
            static_rows.push(self.view_calculation_row(
                preview.expression,
                preview.formatted_value,
                preview.words,
            ));
        } else if filtered.is_empty() && !self.search_in_flight {
            static_rows.push(
                container(
                    text("No applications found")
                        .size(self.layout.empty_state_text_size)
                        .color(appearance.muted_text),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
            );
        } else if !filtered.is_empty() {
            for (rank, index) in filtered.iter().copied().enumerate() {
                keyed_rows.push((
                    index,
                    self.view_result_row(index, rank == self.highlighted_rank),
                ));
            }
        }

        let mut results = column(static_rows)
            .spacing(self.layout.result_row_gap)
            .width(Length::Fill);

        if !keyed_rows.is_empty() {
            results = results.push(
                keyed_column(keyed_rows)
                    .spacing(self.layout.result_row_gap)
                    .width(Length::Fill),
            );
        }

        let show_scrollbar = self.should_show_results_scrollbar();
        let scrollbar = if show_scrollbar {
            iced::widget::scrollable::Scrollbar::new()
                .width(8)
                .scroller_width(4)
                .margin(0)
        } else {
            iced::widget::scrollable::Scrollbar::hidden()
        };

        let padded_results = container(results)
            .width(Length::Fill)
            .padding([0, self.layout.results_side_padding as u16]);

        let list = scrollable(padded_results)
            .id(self.results_scroll_id.clone())
            .height(Length::Fill)
            .on_scroll(Message::ResultsScrolled)
            .direction(iced::widget::scrollable::Direction::Vertical(scrollbar))
            .style(move |theme, status| {
                results_scroll_style(theme, &appearance, show_scrollbar, status)
            });

        container(list)
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.results_height * progress))
            .padding(Padding {
                top: self.layout.results_top_bottom_padding,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .into()
    }

    fn view_result_row(&self, index: usize, selected: bool) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let app = &self.apps[index];

        let left = row![
            container(self.view_app_icon(app, self.layout.result_icon_size))
                .width(Length::Fixed(self.layout.result_icon_box_size))
                .height(Length::Fixed(self.layout.result_icon_box_size))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .padding([0, 3]),
            column![
                text(&app.name)
                    .size(self.layout.result_primary_text_size)
                    .color(appearance.primary_text),
                text(trim_label(&app.exec_line, self.layout.result_label_max_len))
                    .size(self.layout.result_secondary_text_size)
                    .color(appearance.secondary_text),
            ]
            .spacing(1)
            .width(Length::Fill),
            container(
                text(&app.entry_type)
                    .size(
                        (self.layout.result_secondary_text_size + 1.0)
                            .max(RESULT_META_TEXT_MIN_SIZE)
                    )
                    .color(appearance.muted_text),
            )
            .width(Length::Fixed(RESULT_META_LABEL_WIDTH))
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            button(left)
                .on_press(Message::LaunchIndex(index))
                .padding([0, 10])
                .width(Length::Fill)
                .height(Length::Fixed(self.layout.result_row_button_height()))
                .style(move |_theme, status| {
                    result_button_style(status, selected, &self.layout, &appearance)
                }),
        )
        .width(Length::Fill)
        .height(Length::Fixed(self.layout.result_row_height))
        .padding([self.layout.result_row_inset_y as u16, 0])
        .into()
    }

    fn view_calculation_row(
        &self,
        expression: String,
        formatted_value: String,
        words: Option<String>,
    ) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let badge_size = (self.layout.result_secondary_text_size - 1.0).max(10.0);
        let headline_size = self.layout.result_primary_text_size + 4.0;
        let headline_max_chars = self.calculation_headline_max_chars();
        let badge_max_chars = self.calculation_badge_max_chars();
        let expression = truncate_middle_with_ellipsis(&expression, headline_max_chars);
        let formatted_value = normalize_result_display_value(&formatted_value);
        let formatted_value = truncate_middle_with_ellipsis(&formatted_value, headline_max_chars);
        let value_caption = words
            .unwrap_or_else(|| number_text_for_value(&formatted_value))
            .to_string();
        let value_caption = truncate_middle_with_ellipsis(&value_caption, badge_max_chars);

        let left = column![
            text(expression)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .size(headline_size)
                .color(appearance.primary_text),
            container(
                text("Question")
                    .size(badge_size)
                    .color(appearance.muted_text)
            )
            .padding([2, 8])
            .style(move |_| calculator_badge_style(&appearance)),
        ]
        .spacing(6)
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

        let right = column![
            text(formatted_value)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .size(headline_size)
                .color(appearance.primary_text),
            container(
                text(value_caption)
                    .size(badge_size)
                    .color(appearance.muted_text)
            )
            .padding([2, 8])
            .style(move |_| calculator_badge_style(&appearance)),
        ]
        .spacing(6)
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

        let card = row![
            container(left)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            container(right)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .spacing(12)
        .width(Length::Fill)
        .align_y(iced::alignment::Vertical::Center);

        container(
            container(card)
                .padding([16, 18])
                .style(move |_| calculator_card_style(&self.layout, &appearance)),
        )
        .width(Length::Fill)
        .height(Length::Fixed(
            (self.layout.result_row_height * 1.9).max(108.0),
        ))
        .padding([self.layout.result_row_inset_y as u16, 0])
        .into()
    }

    fn calculation_headline_max_chars(&self) -> usize {
        ((self.layout.panel_width / 22.0).floor() as usize)
            .clamp(CALC_MIN_HEADLINE_CHARS, CALC_MAX_HEADLINE_CHARS)
    }

    fn calculation_badge_max_chars(&self) -> usize {
        ((self.layout.panel_width / 16.0).floor() as usize)
            .clamp(CALC_MIN_BADGE_CHARS, CALC_MAX_BADGE_CHARS)
    }

    fn view_bottom_strip(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let shadcn_theme = bottom_strip_shadcn_theme(&appearance);
        let calculator_active = self.calculation_preview().is_some();
        let (label_text, icon_button) = if self.query.is_empty() && self.results_target == 0.0 {
            (
                "Show more",
                action_icon_button(
                    icon_chevron_down().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                    BottomStripAction::Expand,
                ),
            )
        } else {
            (
                if calculator_active {
                    "Copy Result"
                } else {
                    "Open"
                },
                action_icon_button(
                    icon_corner_down_left().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                    BottomStripAction::Launch,
                ),
            )
        };

        let logo = container(
            svg(self.launcher_logo_handle())
                .width(Length::Fixed(self.layout.logo_width))
                .height(Length::Fixed(self.layout.logo_height)),
        )
        .padding([0, 4])
        .center_y(Length::Fill);

        let action_trigger: Element<'_, Message> = if self.is_expanded() {
            let action_toggle = tooltip(
                action_icon_button(
                    icon_ellipsis_vertical().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                    BottomStripAction::ToggleActions,
                ),
                container(
                    text("Toggle action shortcuts")
                        .size(11)
                        .color(appearance.primary_text),
                )
                .padding([4, 8]),
                iced::widget::tooltip::Position::Top,
            );

            row![
                text("Actions")
                    .size((self.layout.result_secondary_text_size - 0.5).max(10.0))
                    .color(appearance.muted_text),
                action_toggle,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(self.layout.bottom_strip_label_gap)
            .into()
        } else {
            Space::new().width(Length::Shrink).into()
        };

        let show_more = container(
            row![
                text(label_text)
                    .size(self.layout.result_primary_text_size)
                    .color(appearance.muted_text),
                icon_button,
                action_trigger,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(self.layout.bottom_strip_label_gap),
        )
        .height(Length::Fill)
        .center_y(Length::Fill);

        container(
            row![logo, space::horizontal(), show_more,]
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .spacing(0),
        )
        .height(Length::Fixed(self.layout.bottom_strip_height))
        .padding([0, self.layout.bottom_strip_padding_x as u16])
        .style(move |_| bottom_strip_style(&appearance))
        .into()
    }

    fn view_action_layer(&self) -> Element<'_, Message> {
        let should_show_overlay = self.should_show_action_overlay();
        let right_offset = ACTION_OVERLAY_RIGHT_OFFSET.max(0.0);
        let bottom_offset =
            (self.layout.bottom_strip_height + ACTION_OVERLAY_BOTTOM_OFFSET).max(0.0);

        if !should_show_overlay {
            return container(column![])
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([0, self.layout.bottom_strip_padding_x as u16])
                .into();
        }

        let overlay = float(opaque(self.view_action_overlay_box())).translate(
            move |content_bounds, viewport_bounds| {
                let target_x =
                    viewport_bounds.x + viewport_bounds.width - content_bounds.width - right_offset;
                let target_y = viewport_bounds.y + viewport_bounds.height
                    - content_bounds.height
                    - bottom_offset;

                iced::Vector::new(target_x - content_bounds.x, target_y - content_bounds.y)
            },
        );

        container(overlay)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([0, self.layout.bottom_strip_padding_x as u16])
            .into()
    }

    fn view_action_overlay_box(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let card_width = (self.layout.panel_width * 0.36).max(ACTION_CARD_MIN_WIDTH);
        let action_text_size = (self.layout.result_secondary_text_size - 0.5).max(10.0);
        let label_size = (self.layout.result_secondary_text_size - 1.0).max(10.0);

        let open_action = row![
            icon_external_link().size(self.layout.action_icon_size * 0.62),
            text("Open")
                .size(self.layout.result_primary_text_size)
                .color(appearance.primary_text),
            space::horizontal(),
            text("Alt + 1")
                .size(action_text_size)
                .color(appearance.muted_text),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(8);

        let open_location_action = row![
            icon_folder_open().size(self.layout.action_icon_size * 0.62),
            text("Open location")
                .size(self.layout.result_primary_text_size)
                .color(appearance.primary_text),
            space::horizontal(),
            text("Alt + 2")
                .size(action_text_size)
                .color(appearance.muted_text),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(8);

        container(
            column![
                row![
                    text("Application Actions")
                        .size(label_size)
                        .color(appearance.muted_text),
                    space::horizontal(),
                    text("hold Alt")
                        .size(label_size)
                        .color(appearance.muted_text),
                ]
                .align_y(iced::alignment::Vertical::Center),
                open_action,
                open_location_action,
            ]
            .spacing(8),
        )
        .width(Length::Fixed(card_width))
        .padding([10, 12])
        .style(move |_| action_card_style(&self.layout, &appearance))
        .into()
    }

    fn view_app_icon(&self, app: &DesktopApp, size: f32) -> Element<'_, Message> {
        if let Some(path) = app.icon_path.as_deref() {
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase());

            if matches!(extension.as_deref(), Some("svg")) {
                return svg(path.to_path_buf())
                    .width(Length::Fixed(size))
                    .height(Length::Fixed(size))
                    .into();
            }

            return image(path.to_path_buf())
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
                .content_fit(ContentFit::Contain)
                .into();
        }

        let fallback = app
            .name
            .chars()
            .find(|ch| ch.is_alphanumeric())
            .unwrap_or('A')
            .to_string();

        text(fallback)
            .size(12)
            .color(self.resolved_appearance().primary_text)
            .into()
    }

    fn view_progress_line(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let (leading_track, active, trailing_track) =
            self.progress_line_widths(self.layout.panel_width);

        row![
            container(
                rule::horizontal(1).style(move |_| iced::widget::rule::Style {
                    color: appearance.progress_track,
                    radius: 0.0.into(),
                    fill_mode: iced::widget::rule::FillMode::Full,
                    snap: true,
                }),
            )
            .width(Length::Fixed(leading_track))
            .height(1),
            container(
                rule::horizontal(1).style(move |_| iced::widget::rule::Style {
                    color: appearance.progress_indicator,
                    radius: 0.0.into(),
                    fill_mode: iced::widget::rule::FillMode::Full,
                    snap: true,
                }),
            )
            .width(Length::Fixed(active))
            .height(1),
            container(
                rule::horizontal(1).style(move |_| iced::widget::rule::Style {
                    color: appearance.progress_track,
                    radius: 0.0.into(),
                    fill_mode: iced::widget::rule::FillMode::Full,
                    snap: true,
                }),
            )
            .width(Length::Fixed(trailing_track))
            .height(1),
        ]
        .width(Length::Fill)
        .height(1)
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

fn truncate_middle_with_ellipsis(value: &str, max_chars: usize) -> String {
    if max_chars <= 3 {
        return "...".to_string();
    }

    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    let visible = max_chars.saturating_sub(3);
    let left_count = visible / 2;
    let right_count = visible.saturating_sub(left_count);

    let left: String = value.chars().take(left_count).collect();
    let right: String = value
        .chars()
        .skip(char_count.saturating_sub(right_count))
        .collect();

    let mut output = left;
    output.push_str("...");
    output.push_str(&right);
    output
}

fn normalize_result_display_value(value: &str) -> String {
    let cleaned: String = value.chars().filter(|ch| *ch != ',').collect();

    if let Ok(integer) = cleaned.parse::<i128>() {
        return crate::ui::format::group_i128(integer);
    }

    if let Some((integer, fraction)) = cleaned.split_once('.')
        && let Ok(integer) = integer.parse::<i128>()
    {
        return format!("{}.{}", crate::ui::format::group_i128(integer), fraction);
    }

    value.to_string()
}

fn number_text_for_value(value: &str) -> String {
    let mut parts = Vec::new();

    for ch in value.chars() {
        let part = match ch {
            '0' => Some("Zero"),
            '1' => Some("One"),
            '2' => Some("Two"),
            '3' => Some("Three"),
            '4' => Some("Four"),
            '5' => Some("Five"),
            '6' => Some("Six"),
            '7' => Some("Seven"),
            '8' => Some("Eight"),
            '9' => Some("Nine"),
            '-' => Some("Negative"),
            '.' => Some("Point"),
            ',' => None,
            _ => None,
        };

        if let Some(part) = part {
            parts.push(part);
        }
    }

    if parts.is_empty() {
        "Result".to_string()
    } else {
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_uses_middle_three_dot_ellipsis() {
        let truncated = truncate_middle_with_ellipsis("abcdef", 5);

        assert_eq!(truncated.chars().count(), 5);
        assert!(truncated.starts_with('a'));
        assert!(truncated.ends_with('f'));
        assert!(truncated.contains("..."));
        assert_eq!(truncate_middle_with_ellipsis("abc", 5), "abc");
    }

    #[test]
    fn value_fallback_converts_digits_to_words() {
        let spoken = number_text_for_value("-10.5");

        assert!(spoken.starts_with("Negative"));
        assert!(spoken.contains("One"));
        assert!(spoken.contains("Zero"));
        assert!(spoken.contains("Point"));
        assert!(spoken.ends_with("Five"));
        assert_eq!(number_text_for_value("abc"), "Result");
    }

    #[test]
    fn result_display_is_grouped_with_commas() {
        assert_eq!(normalize_result_display_value("1234567"), "1,234,567");
        assert_eq!(normalize_result_display_value("1234567.89"), "1,234,567.89");
    }
}
