use super::{Launcher, Message, container, row, rule};
use iced::{Element, Length};

impl Launcher {
    pub(super) fn view_progress_line(&self) -> Element<'_, Message> {
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
