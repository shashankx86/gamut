use super::launcher::{Launcher, Message};
use super::styles::{
    backdrop_style, bottom_strip_style, divider_style, panel_style, result_button_style,
    results_scroll_style, search_input_style,
};
use crate::core::desktop::{trim_label, DesktopApp};
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, column, container, image, row, scrollable, svg, text, text_input};
use iced::{window, Color, ContentFit, Element, Length};
use iced_shadcn::{
    icon_button, ButtonProps, ButtonRadius, ButtonSize, ButtonVariant, Palette as ShadcnPalette,
    Theme as ShadcnTheme,
};
use lucide_icons::iced::{icon_chevron_down, icon_corner_down_left, icon_search};

const BOTTOM_STRIP_ICON_BUTTON_SIZE: f32 = 20.0;
const BOTTOM_STRIP_ICON_SIZE: f32 = 12.0;

impl Launcher {
    pub(super) fn view(&self, window: window::Id) -> Element<'_, Message> {
        let _ = window;
        let appearance = self.resolved_appearance();

        if !self.is_visible {
            return container("")
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

        content = content
            .push(
                container("")
                    .height(1)
                    .width(Length::Fill)
                    .style(move |_| divider_style(&appearance)),
            )
            .push(self.view_bottom_strip());

        let launcher_panel = container(content)
            .padding(0)
            .style(move |_| panel_style(&self.layout, &appearance))
            .width(Length::Fill)
            .max_width(self.layout.panel_width as u32)
            .height(Length::Shrink);

        container(launcher_panel)
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
        let mut results = column![]
            .spacing(self.layout.result_row_gap)
            .width(Length::Fill);
        let filtered = self.filtered_indices();

        if filtered.is_empty() && !self.search_in_flight {
            results = results.push(
                container(
                    text("No applications found")
                        .size(self.layout.empty_state_text_size)
                        .color(Color::from_rgb(0.62, 0.64, 0.67)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            );
        } else if !filtered.is_empty() {
            let render_range = self.result_render_range();
            let visible_start = render_range.start;
            let visible_end = render_range.end;

            if visible_start > 0 {
                results = results.push(
                    container("")
                        .width(Length::Fill)
                        .height(Length::Fixed(self.results_top_spacer_height())),
                );
            }

            for (rank, index) in filtered
                .iter()
                .copied()
                .enumerate()
                .skip(visible_start)
                .take(visible_end.saturating_sub(visible_start))
            {
                results = results.push(self.view_result_row(index, rank == self.highlighted_rank));
            }

            let trailing_rows = filtered.len().saturating_sub(visible_end);
            if trailing_rows > 0 {
                results = results.push(container("").width(Length::Fill).height(Length::Fixed(
                    super::launcher::spacer_height_for_rows(
                        trailing_rows,
                        self.layout.result_row_height,
                        self.layout.result_row_gap,
                    ),
                )));
            }
        }

        let list = scrollable(results)
            .id(self.results_scroll_id.clone())
            .height(Length::Fill)
            .on_scroll(Message::ResultsScrolled)
            .direction(iced::widget::scrollable::Direction::Vertical(
                iced::widget::scrollable::Scrollbar::new()
                    .width(10)
                    .scroller_width(6)
                    .spacing(4),
            ))
            .style(move |theme, status| results_scroll_style(theme, &appearance, status));

        container(list)
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.results_height * progress))
            .padding([
                self.layout.results_top_bottom_padding as u16,
                self.layout.results_side_padding as u16,
            ])
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

    fn view_bottom_strip(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let shadcn_theme = bottom_strip_shadcn_theme(&appearance);
        let (label_text, icon_button) = if self.query.is_empty() && self.results_target == 0.0 {
            (
                "Show more",
                action_icon_button(
                    icon_chevron_down().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                ),
            )
        } else {
            (
                "Open",
                action_icon_button(
                    icon_corner_down_left().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                ),
            )
        };

        let logo = container(
            svg(SvgHandle::from_memory(include_bytes!(
                "../../assets/icons/gamut-full-transparent-dark.svg"
            )))
            .width(Length::Fixed(self.layout.logo_width))
            .height(Length::Fixed(self.layout.logo_height)),
        )
        .padding([0, 4])
        .center_y(Length::Fill);

        let show_more = container(
            row![
                text(label_text)
                    .size(self.layout.result_primary_text_size)
                    .color(appearance.muted_text),
                icon_button,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(self.layout.bottom_strip_label_gap),
        )
        .height(Length::Fill)
        .center_y(Length::Fill);

        container(
            row![logo, container("").width(Length::Fill), show_more,]
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

    fn result_render_range(&self) -> std::ops::Range<usize> {
        let filtered = self.filtered_indices();

        if filtered.is_empty() {
            return 0..0;
        }

        super::launcher::render_range_for_viewport(
            self.results_scroll_offset,
            self.layout.results_viewport_height(),
            filtered.len(),
            self.layout.result_row_height,
            self.layout.result_row_gap,
        )
    }

    fn results_top_spacer_height(&self) -> f32 {
        super::launcher::spacer_height_for_rows(
            self.result_render_range().start,
            self.layout.result_row_height,
            self.layout.result_row_gap,
        )
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
        ring: appearance.selection,
        primary: appearance.primary_text,
        primary_foreground: appearance.panel_background,
        secondary: appearance.first_row_active,
        secondary_foreground: appearance.primary_text,
        accent: appearance.first_row_hover,
        accent_foreground: appearance.primary_text,
        muted: appearance.first_row_active,
        muted_foreground: appearance.muted_text,
        destructive: appearance.scrollbar_scroller_dragged_border,
        destructive_foreground: appearance.panel_background,
        chart_1: appearance.selection,
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
        sidebar_ring: appearance.selection,
    })
}

fn action_icon_button<'a>(
    icon: impl Into<Element<'a, Message>>,
    theme: &ShadcnTheme,
) -> iced::widget::button::Button<'a, Message> {
    icon_button(
        icon,
        Some(Message::LaunchFirstMatch),
        ButtonProps::new()
            .variant(ButtonVariant::Outline)
            .radius(ButtonRadius::Small)
            .size(ButtonSize::Size1),
        theme,
    )
    .width(Length::Fixed(BOTTOM_STRIP_ICON_BUTTON_SIZE))
    .height(Length::Fixed(BOTTOM_STRIP_ICON_BUTTON_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use std::sync::mpsc;

    fn app(index: usize) -> DesktopApp {
        DesktopApp::new(
            format!("App {index}"),
            format!("/usr/bin/app-{index} %u"),
            format!("/usr/bin/app-{index}"),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        )
    }

    fn launcher_with_results(total_results: usize) -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (mut launcher, _) = Launcher::new(rx);
        launcher.apps = (0..total_results).map(app).collect();
        launcher.all_app_indices = (0..launcher.apps.len()).collect();
        launcher.filtered_indices = launcher.all_app_indices.clone();
        launcher.results_scroll_offset = 0.0;
        launcher
    }

    #[test]
    fn query_driven_expansion_only_renders_visible_rows_plus_buffer() {
        let launcher = launcher_with_results(20);

        assert_eq!(launcher.result_render_range(), 0..7);
    }

    #[test]
    fn render_range_tracks_scrolled_window_for_large_result_sets() {
        let mut launcher = launcher_with_results(20);
        launcher.results_scroll_offset = 4.0 * launcher.layout.result_row_scroll_step();

        assert_eq!(launcher.result_render_range(), 3..11);
    }

    #[test]
    fn top_spacer_matches_hidden_rows_without_extra_gap() {
        let mut launcher = launcher_with_results(20);
        launcher.results_scroll_offset = 2.1 * launcher.layout.result_row_scroll_step();

        assert_eq!(
            launcher.results_top_spacer_height(),
            launcher.layout.result_row_height
        );
    }
}
