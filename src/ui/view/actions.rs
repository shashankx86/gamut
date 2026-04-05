use super::{
    action_card_style, column, container, float, icon_external_link, icon_folder_open, opaque, row,
    space, text, Launcher, Message, ACTION_CARD_MIN_COMPACT_WIDTH, ACTION_CARD_MIN_WIDTH,
    ACTION_OVERLAY_BOTTOM_OFFSET, ACTION_OVERLAY_RIGHT_OFFSET,
};
use iced::{Element, Length};

impl Launcher {
    pub(super) fn view_action_layer(&self) -> Element<'_, Message> {
        let should_show_overlay = self.should_show_action_overlay();
        let right_offset = ACTION_OVERLAY_RIGHT_OFFSET.max(0.0);
        let bottom_offset =
            (self.layout.bottom_strip_height + ACTION_OVERLAY_BOTTOM_OFFSET).max(0.0);
        let panel_width = self.layout.panel_width as u32;

        if !should_show_overlay {
            return container(column![])
                .width(Length::Fill)
                .max_width(panel_width)
                .height(Length::Fill)
                .padding([0, self.layout.bottom_strip_padding_x as u16])
                .into();
        }

        let overlay = float(opaque(self.view_action_overlay_box())).translate(
            move |content_bounds, viewport_bounds| {
                let preferred_x =
                    viewport_bounds.x + viewport_bounds.width - content_bounds.width - right_offset;
                let min_x = viewport_bounds.x;
                let max_x =
                    viewport_bounds.x + (viewport_bounds.width - content_bounds.width).max(0.0);
                let target_x = preferred_x.clamp(min_x, max_x);

                let preferred_y = viewport_bounds.y + viewport_bounds.height
                    - content_bounds.height
                    - bottom_offset;
                let min_y = viewport_bounds.y;
                let max_y =
                    viewport_bounds.y + (viewport_bounds.height - content_bounds.height).max(0.0);
                let target_y = preferred_y.clamp(min_y, max_y);

                iced::Vector::new(target_x - content_bounds.x, target_y - content_bounds.y)
            },
        );

        container(overlay)
            .width(Length::Fill)
            .max_width(panel_width)
            .height(Length::Fill)
            .padding([0, self.layout.bottom_strip_padding_x as u16])
            .into()
    }

    pub(super) fn view_action_overlay_box(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let available_width = (self.layout.panel_width
            - (self.layout.bottom_strip_padding_x * 2.0)
            - ACTION_OVERLAY_RIGHT_OFFSET)
            .max(ACTION_CARD_MIN_COMPACT_WIDTH);
        let preferred_width = (self.layout.panel_width * 0.36).max(ACTION_CARD_MIN_WIDTH);
        let card_width = preferred_width.min(available_width);
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
}
