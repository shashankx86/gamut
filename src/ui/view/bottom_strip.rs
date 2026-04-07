use super::{
    BOTTOM_STRIP_ICON_SIZE, BottomStripAction, Launcher, Message, action_icon_button,
    bottom_strip_shadcn_theme, bottom_strip_style, container, icon_chevron_down,
    icon_corner_down_left, icon_ellipsis_vertical, row, space, text, tooltip,
};
use iced::{Element, Length};

impl Launcher {
    pub(super) fn view_bottom_strip(&self) -> Element<'_, Message> {
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
                    false,
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
                    false,
                ),
            )
        };

        let logo = container(
            super::svg(self.launcher_logo_handle())
                .width(Length::Fixed(self.layout.logo_width))
                .height(Length::Fixed(self.layout.logo_height)),
        )
        .padding([0, 4])
        .center_y(Length::Fill);

        let primary_action = row![
            text(label_text)
                .size(self.layout.result_primary_text_size)
                .color(appearance.muted_text),
            icon_button,
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(self.layout.bottom_strip_label_gap);

        let show_more = if self.is_expanded() {
            let action_toggle = tooltip(
                action_icon_button(
                    icon_ellipsis_vertical().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                    BottomStripAction::ToggleActions,
                    self.modifiers.alt(),
                ),
                container(
                    text("Toggle action shortcuts")
                        .size(11)
                        .color(appearance.primary_text),
                )
                .padding([4, 8]),
                iced::widget::tooltip::Position::Top,
            );

            let action_trigger: Element<'_, Message> = row![
                text("Actions")
                    .size((self.layout.result_secondary_text_size - 0.5).max(10.0))
                    .color(appearance.muted_text),
                action_toggle,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(self.layout.bottom_strip_label_gap)
            .into();

            container(
                row![primary_action, action_trigger]
                    .align_y(iced::alignment::Vertical::Center)
                    .spacing(self.layout.bottom_strip_label_gap),
            )
            .height(Length::Fill)
            .center_y(Length::Fill)
        } else {
            container(primary_action)
                .height(Length::Fill)
                .center_y(Length::Fill)
        };

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
}
