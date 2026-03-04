use super::constants::{PANEL_WIDTH, RESULTS_HEIGHT};
use super::launcher::{Launcher, Message};
use super::styles::{
    app_icon_style, backdrop_style, footer_key_chip, footer_style, header_style, panel_style,
    result_button_style, results_scroll_style, search_input_style,
};
use crate::desktop::trim_label;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Length, window};

impl Launcher {
    pub(super) fn view(&self, _window: window::Id) -> Element<'_, Message> {
        let launcher_panel = container(
            column![
                self.view_search_header(),
                self.view_results_section(),
                self.view_footer_bar(),
            ]
            .spacing(0)
            .width(Length::Fixed(PANEL_WIDTH))
            .height(Length::Shrink),
        )
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
        let search = text_input("Search applications", &self.query)
            .id(self.input_id.clone())
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchFirstMatch)
            .padding([10, 10])
            .size(18)
            .style(search_input_style);

        container(search)
            .padding([10, 12])
            .style(|_| header_style())
            .into()
    }

    fn view_results_section(&self) -> Element<'_, Message> {
        let mut results = column![].spacing(4).width(Length::Fill).padding([0, 4]);

        let filtered = self.filtered_indices();

        if filtered.is_empty() {
            results = results.push(
                container(
                    text("No applications found")
                        .size(16)
                        .color(Color::from_rgb(0.7, 0.72, 0.76)),
                )
                .padding([12, 12]),
            );
        } else {
            for (rank, index) in filtered.into_iter().enumerate() {
                results = results.push(self.view_result_row(index, rank == 0));
            }
        }

        let results_header = container(
            text("Results")
                .size(15)
                .color(Color::from_rgb(0.62, 0.64, 0.67)),
        )
        .padding([10, 4]);

        let list = scrollable(results)
            .height(Length::Fill)
            .style(results_scroll_style);

        container(column![results_header, list].spacing(0))
            .width(Length::Fill)
            .height(Length::Fixed(RESULTS_HEIGHT))
            .padding([4, 12])
            .into()
    }

    fn view_result_row(&self, index: usize, first_row: bool) -> Element<'_, Message> {
        let app = &self.apps[index];
        let icon = app
            .name
            .chars()
            .find(|ch| ch.is_alphanumeric())
            .unwrap_or('A')
            .to_string();

        let left = row![
            container(text(icon).size(12).color(Color::from_rgb(0.88, 0.90, 0.94)),)
                .padding([3, 8])
                .style(|_| app_icon_style()),
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

        let right = text("Application")
            .size(12)
            .color(Color::from_rgb(0.67, 0.69, 0.72));

        button(
            row![left, right]
                .align_y(iced::alignment::Vertical::Center)
                .width(Length::Fill),
        )
        .on_press(Message::LaunchIndex(index))
        .padding([7, 10])
        .width(Length::Fill)
        .style(move |_theme, status| result_button_style(status, first_row))
        .into()
    }

    fn view_footer_bar(&self) -> Element<'_, Message> {
        let status_text = self
            .status
            .as_deref()
            .unwrap_or("Open Application")
            .to_string();

        let actions = row![
            text(status_text)
                .size(13)
                .color(Color::from_rgb(0.84, 0.86, 0.89)),
            text("  ").size(13),
            footer_key_chip("Enter"),
            text("  ").size(13),
            text("Actions")
                .size(12)
                .color(Color::from_rgb(0.62, 0.64, 0.68)),
            text(" ").size(12),
            footer_key_chip("Ctrl+K"),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(0);

        container(actions)
            .padding([7, 12])
            .style(|_| footer_style())
            .into()
    }
}
