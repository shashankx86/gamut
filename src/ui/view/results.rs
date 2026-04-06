use super::{
    button, calculator_badge_style, calculator_card_style, column, container, image, keyed_column,
    normalize_result_display_value, results_scroll_style, truncate_middle_with_ellipsis,
    DesktopApp, Launcher, Length, Message, Padding, RESULT_META_LABEL_WIDTH,
    RESULT_META_TEXT_MIN_SIZE,
};
use super::{number_text_for_value, row, scrollable, svg, text, trim_label};
use super::{
    CALC_MAX_BADGE_CHARS, CALC_MAX_HEADLINE_CHARS, CALC_MIN_BADGE_CHARS, CALC_MIN_HEADLINE_CHARS,
};
use iced::{ContentFit, Element};

impl Launcher {
    pub(super) fn view_results_section(&self, progress: f32) -> Element<'_, Message> {
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

    pub(super) fn view_result_row(&self, index: usize, selected: bool) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let app = &self.apps[index];

        let left = row![
            container(self.view_app_icon(app, self.layout.result_icon_size))
                .width(Length::Fixed(self.layout.result_icon_box_size))
                .height(Length::Fixed(self.layout.result_icon_box_size))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .padding([0, 2]),
            column![
                text(&app.name)
                    .size(self.layout.result_primary_text_size)
                    .color(appearance.primary_text),
                text(trim_label(&app.exec_line, self.layout.result_label_max_len))
                    .size(self.layout.result_secondary_text_size)
                    .color(appearance.secondary_text),
            ]
            .spacing(0)
            .width(Length::Fill),
            container(
                text(&app.entry_type)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::DEFAULT
                    })
                    .size(
                        (self.layout.result_secondary_text_size + 1.0)
                            .max(RESULT_META_TEXT_MIN_SIZE)
                    )
                    .color(appearance.muted_text),
            )
            .width(Length::Fixed(RESULT_META_LABEL_WIDTH))
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(8)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            button(left)
                .on_press(Message::LaunchIndex(index))
                .padding([0, 8])
                .width(Length::Fill)
                .height(Length::Fixed(self.layout.result_row_button_height()))
                .style(move |_theme, status| {
                    super::result_button_style(status, selected, &self.layout, &appearance)
                }),
        )
        .width(Length::Fill)
        .height(Length::Fixed(self.layout.result_row_height))
        .padding([self.layout.result_row_inset_y as u16, 0])
        .into()
    }

    pub(super) fn view_calculation_row(
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

    pub(super) fn view_app_icon(&self, app: &DesktopApp, size: f32) -> Element<'_, Message> {
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
}
