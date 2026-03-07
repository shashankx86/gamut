use super::launcher::{Launcher, Message};
use super::styles::{
    backdrop_style, bottom_strip_style, divider_style, panel_style, result_button_style,
    results_scroll_style, search_input_style, show_more_icon_style,
};
use crate::core::desktop::{trim_label, DesktopApp};
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, column, container, image, row, scrollable, svg, text, text_input};
use iced::{window, Color, ContentFit, Element, Length};

const SEARCH_ICON_SVG: &[u8] = br##"<svg width="18" height="18" viewBox="-1.5 -1.5 21 21" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M18 18L13.65 13.65M16 8C16 12.4183 12.4183 16 8 16C3.58172 16 0 12.4183 0 8C0 3.58172 3.58172 0 8 0C12.4183 0 16 3.58172 16 8Z" stroke="#727272" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
const SHOW_MORE_ICON_SVG: &[u8] = br##"<svg width="22" height="22" viewBox="0 0 22 22" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="2" width="18" height="18" rx="4" stroke="#727272" stroke-width="1.4"/><path d="M8.1 9.7L11 12.6L13.9 9.7" stroke="#83878F" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
const ENTER_ICON_SVG: &[u8] = br##"<svg width="22" height="22" viewBox="0 0 22 22" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="2" width="18" height="18" rx="4" stroke="#727272" stroke-width="1.4"/><path d="M14 8V11.5C14 12.0523 13.5523 12.5 13 12.5H8" stroke="#83878F" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/><path d="M10.5 10L8 12.5L10.5 15" stroke="#83878F" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

impl Launcher {
    pub(super) fn view(&self, _window: window::Id) -> Element<'_, Message> {
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

        row![
            svg(SvgHandle::from_memory(SEARCH_ICON_SVG))
                .width(Length::Fixed(self.layout.search_icon_size))
                .height(Length::Fixed(self.layout.search_icon_size)),
            input,
        ]
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

        if filtered.is_empty() {
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
        } else {
            let render_range = self.result_render_range();
            let row_step = self.layout.result_row_scroll_step();
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
                    super::launcher::spacer_height_for_rows(trailing_rows, row_step),
                )));
            }
        }

        let list = scrollable(results)
            .id(self.results_scroll_id.clone())
            .height(Length::Fill)
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

    fn view_result_row(&self, index: usize, first_row: bool) -> Element<'_, Message> {
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

        button(left)
            .on_press(Message::LaunchIndex(index))
            .padding([0, 10])
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.result_row_height))
            .style(move |_theme, status| {
                result_button_style(status, first_row, &self.layout, &appearance)
            })
            .into()
    }

    fn view_bottom_strip(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let (label_text, icon_svg) = if self.query.is_empty() && self.results_target == 0.0 {
            ("Show more", SHOW_MORE_ICON_SVG)
        } else {
            ("Open", ENTER_ICON_SVG)
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
                container(
                    svg(SvgHandle::from_memory(icon_svg))
                        .width(Length::Fixed(self.layout.action_icon_size))
                        .height(Length::Fixed(self.layout.action_icon_size))
                )
                .width(Length::Fixed(self.layout.action_icon_size))
                .height(Length::Fixed(self.layout.action_icon_size))
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .style(|_| show_more_icon_style()),
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

        if super::launcher::is_manual_expansion_in_progress(
            self.normalized_query.is_empty(),
            self.manually_expanded,
            self.results_progress,
            self.results_target,
        ) {
            return super::launcher::expansion_render_range(
                self.scroll_start_rank,
                filtered.len(),
                self.layout.visible_result_rows(),
            );
        }

        0..filtered.len()
    }

    fn results_top_spacer_height(&self) -> f32 {
        super::launcher::spacer_height_for_rows(
            self.result_render_range().start,
            self.layout.result_row_scroll_step(),
        )
    }
}
