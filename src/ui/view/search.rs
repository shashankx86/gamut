use super::{Launcher, Message, icon_search, search_input_style};
use iced::widget::{row, text_input};
use iced::{Element, Length};

impl Launcher {
    pub(super) fn view_search_header(&self) -> Element<'_, Message> {
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
}
