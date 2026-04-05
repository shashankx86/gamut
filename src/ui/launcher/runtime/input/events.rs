use super::super::super::{Launcher, Message};
use iced::keyboard;
use iced::widget::operation;
use iced::{Task, window};

impl Launcher {
    pub(in crate::ui::launcher) fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            let monitor_size = window::monitor_size(id).map(Message::MonitorSizeLoaded);

            if !self.is_visible {
                return monitor_size;
            }

            self.arm_unfocus_guard();

            return Task::batch(vec![
                monitor_size,
                operation::focus(self.input_id.clone()),
                operation::move_cursor_to_end(self.input_id.clone()),
                self.request_icon_resolution_for_visible(),
            ]);
        }

        Task::none()
    }

    pub(in crate::ui::launcher) fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            self.launcher_window_id = None;
            self.is_visible = false;
            self.clear_window_state();
        }

        Task::none()
    }

    pub(in crate::ui::launcher) fn handle_key_event(
        &mut self,
        id: window::Id,
        event: keyboard::Event,
    ) -> Task<Message> {
        match event {
            keyboard::Event::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                self.sync_alt_action_state_with_modifiers();
                Task::none()
            }
            keyboard::Event::KeyPressed {
                key,
                modifiers,
                physical_key,
                ..
            } => {
                self.modifiers = modifiers;

                if self.is_launcher_window(id) && self.is_visible {
                    return self.handle_launcher_key_press(key, modifiers, physical_key);
                }

                Task::none()
            }
            keyboard::Event::KeyReleased { modifiers, .. } => {
                self.modifiers = modifiers;
                self.sync_alt_action_state_with_modifiers();
                Task::none()
            }
        }
    }

    pub(in crate::ui::launcher) fn handle_window_event(
        &mut self,
        id: window::Id,
        event: window::Event,
    ) -> Task<Message> {
        if self.is_launcher_window(id) && self.is_visible {
            return match event {
                window::Event::Focused => {
                    self.ignore_unfocus_until = None;
                    Task::none()
                }
                window::Event::Unfocused if !self.should_ignore_unfocus() => self.hide_launcher(),
                window::Event::CloseRequested => self.hide_launcher(),
                _ => Task::none(),
            };
        }

        Task::none()
    }
}
