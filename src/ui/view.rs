use super::constants::{PANEL_WIDTH, RESULT_ROW_GAP, RESULT_ROW_HEIGHT, RESULTS_HEIGHT};
use super::launcher::{Launcher, Message};
use super::styles::{
    backdrop_style, bottom_strip_style, divider_style, panel_style, result_button_style,
    results_scroll_style, search_input_style,
};
use crate::core::desktop::{DesktopApp, trim_label};
use iced::widget::{button, column, container, image, row, scrollable, svg, text, text_input};
use iced::{Color, ContentFit, Element, Length, window};

impl Launcher {
    pub(super) fn view(&self, _window: window::Id) -> Element<'_, Message> {
        let mut content = column![self.view_search_header()]
            .spacing(8)
            .width(Length::Fixed(PANEL_WIDTH))
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
            .padding([10, 14])
            .style(|_| panel_style())
            .width(Length::Fixed(PANEL_WIDTH))
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
        text_input("Search for apps and commands...", &self.query)
            .id(self.input_id.clone())
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchFirstMatch)
            .padding([10, 4])
            .size(16)
            .style(search_input_style)
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
                .padding([8, 0]),
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
            .padding([2, 0])
            .into()
    }

    fn view_result_row(&self, index: usize, first_row: bool) -> Element<'_, Message> {
        let app = &self.apps[index];

        let left = row![
            container(self.view_app_icon(app, 26.0))
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .padding([0, 2]),
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
            .padding([6, 8])
            .width(Length::Fill)
            .height(Length::Fixed(RESULT_ROW_HEIGHT))
            .style(move |_theme, status| result_button_style(status, first_row))
            .into()
    }

    fn view_bottom_strip(&self) -> Element<'_, Message> {
        let selected_app = if self.query.trim().is_empty() {
            None
        } else {
            self.selected_result_index()
                .and_then(|index| self.apps.get(index))
        };

        let (icon, label) = if let Some(app) = selected_app {
            (self.view_app_icon(app, 16.0), app.name.clone())
        } else {
            (
                text("⌕")
                    .size(15)
                    .color(Color::from_rgb(0.66, 0.68, 0.71))
                    .into(),
                "Search".to_string(),
            )
        };

        container(
            row![
                container(icon)
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
                text(label)
                    .size(11)
                    .color(Color::from_rgb(0.82, 0.83, 0.85)),
            ]
            .width(Length::Fill)
            .align_y(iced::alignment::Vertical::Center)
            .spacing(8),
        )
        .padding([6, 2])
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
