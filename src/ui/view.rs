use super::constants::{
    PANEL_WIDTH, RESULT_ICON_BOX_SIZE, RESULT_ICON_SIZE, RESULT_ROW_GAP, RESULT_ROW_HEIGHT,
    RESULTS_HEIGHT, RESULTS_SIDE_PADDING, SEARCH_ICON_SIZE,
};
use super::launcher::{Launcher, Message};
use super::styles::{
    backdrop_style, bottom_strip_style, divider_style, panel_style, result_button_style,
    results_scroll_style, search_input_style, show_more_icon_style,
};
use crate::core::desktop::{DesktopApp, trim_label};
use iced::widget::{button, column, container, image, row, scrollable, svg, text, text_input};
use iced::widget::svg::Handle as SvgHandle;
use iced::{Color, ContentFit, Element, Length, window};

const SEARCH_ICON_SVG: &[u8] = br##"<svg width="18" height="18" viewBox="-1.5 -1.5 21 21" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M18 18L13.65 13.65M16 8C16 12.4183 12.4183 16 8 16C3.58172 16 0 12.4183 0 8C0 3.58172 3.58172 0 8 0C12.4183 0 16 3.58172 16 8Z" stroke="#727272" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
const SHOW_MORE_ICON_SVG: &[u8] = br##"<svg width="22" height="22" viewBox="0 0 22 22" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="2" width="18" height="18" rx="4" stroke="#727272" stroke-width="1.4"/><path d="M8.1 9.7L11 12.6L13.9 9.7" stroke="#83878F" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
const ENTER_ICON_SVG: &[u8] = br##"<svg width="22" height="22" viewBox="0 0 22 22" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="2" width="18" height="18" rx="4" stroke="#727272" stroke-width="1.4"/><path d="M14 8V11.5C14 12.0523 13.5523 12.5 13 12.5H8" stroke="#83878F" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/><path d="M10.5 10L8 12.5L10.5 15" stroke="#83878F" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

impl Launcher {
    pub(super) fn view(&self, _window: window::Id) -> Element<'_, Message> {
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
                    .style(|_| divider_style()),
            )
            .push(self.view_bottom_strip());

        let launcher_panel = container(content)
            .padding(0)
            .style(|_| panel_style())
            .width(Length::Fill)
            .max_width(PANEL_WIDTH as u32)
            .height(Length::Shrink);

        container(launcher_panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([8, 24])
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| backdrop_style())
            .into()
    }

    fn view_search_header(&self) -> Element<'_, Message> {
        let input = text_input("Search for apps and commands...", &self.query)
            .id(self.input_id.clone())
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchFirstMatch)
            .padding([11, 0])
            .size(20)
            .style(search_input_style)
            .width(Length::Fill);

        row![
            svg(SvgHandle::from_memory(SEARCH_ICON_SVG))
                .width(Length::Fixed(SEARCH_ICON_SIZE))
                .height(Length::Fixed(SEARCH_ICON_SIZE)),
            input,
        ]
        .width(Length::Fill)
        .height(Length::Fixed(55.0))
        .padding([0, 20])
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }

    fn view_results_section(&self, progress: f32) -> Element<'_, Message> {
        let mut results = column![].spacing(RESULT_ROW_GAP).width(Length::Fill);
        let filtered = self.filtered_indices();

        if filtered.is_empty() {
            results = results.push(
                container(
                    text("No applications found")
                        .size(14)
                        .color(Color::from_rgb(0.62, 0.64, 0.67)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            );
        } else {
            for (rank, index) in filtered.iter().copied().enumerate() {
                results = results.push(self.view_result_row(index, rank == self.selected_rank));
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
            .style(results_scroll_style);

        container(list)
            .width(Length::Fill)
            .height(Length::Fixed(RESULTS_HEIGHT * progress))
            .padding([6, RESULTS_SIDE_PADDING])
            .into()
    }

    fn view_result_row(&self, index: usize, first_row: bool) -> Element<'_, Message> {
        let app = &self.apps[index];

        let left = row![
            container(self.view_app_icon(app, RESULT_ICON_SIZE))
                .width(Length::Fixed(RESULT_ICON_BOX_SIZE))
                .height(Length::Fixed(RESULT_ICON_BOX_SIZE))
                .padding([0, 3]),
            column![
                text(&app.name)
                    .size(14)
                    .color(Color::from_rgb(0.92, 0.93, 0.95)),
                text(trim_label(&app.exec_line, 56))
                    .size(12)
                    .color(Color::from_rgb(0.55, 0.58, 0.61)),
            ]
            .spacing(1)
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill);

        button(left)
            .on_press(Message::LaunchIndex(index))
            .padding([5, 10])
            .width(Length::Fill)
            .height(Length::Fixed(RESULT_ROW_HEIGHT))
            .style(move |_theme, status| result_button_style(status, first_row))
            .into()
    }

    fn view_bottom_strip(&self) -> Element<'_, Message> {
        let (label_text, icon_svg) = if self.query.is_empty() && self.results_target == 0.0 {
            ("Show more", SHOW_MORE_ICON_SVG)
        } else {
            ("Open", ENTER_ICON_SVG)
        };

        let show_more = container(
            row![
                text(label_text)
                    .size(14)
                    .color(Color::from_rgb(131.0 / 255.0, 135.0 / 255.0, 143.0 / 255.0)),
                container(
                    svg(SvgHandle::from_memory(icon_svg))
                        .width(Length::Fixed(22.0))
                        .height(Length::Fixed(22.0))
                )
                .width(Length::Fixed(22.0))
                .height(Length::Fixed(22.0))
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .style(|_| show_more_icon_style()),
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(6),
        )
        .height(Length::Fill)
        .center_y(Length::Fill);

        container(
            row![
                container("").width(Length::Fill),
                show_more,
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Center)
            .spacing(0),
        )
        .height(Length::Fixed(31.0))
        .padding([0, 8])
        .style(|_| bottom_strip_style())
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
            .color(Color::from_rgb(0.88, 0.90, 0.94))
            .into()
    }
}
