use super::super::{Launcher, Message, channel};
use channel::{app_command_stream, ipc_command_stream, search_results_stream};
use iced::{Event, Subscription, event, time, window};
use std::time::Duration;

const ACTIVE_TICK_MS: u64 = 25;
const IDLE_TICK_MS: u64 = 1000;

impl Launcher {
    pub(in crate::ui) fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            event::listen_with(|event, _status, id| match event {
                Event::Keyboard(key_event) => Some(Message::KeyboardEvent(id, key_event)),
                _ => None,
            }),
            event::listen_with(|event, _status, id| match event {
                Event::Window(window_event) => Some(Message::WindowEvent(id, window_event)),
                _ => None,
            }),
            window::open_events().map(Message::WindowOpened),
            window::close_events().map(Message::WindowClosed),
            Subscription::run_with(self.app_command_handle(), app_command_stream),
            Subscription::run_with(self.ipc_handle(), ipc_command_stream),
            Subscription::run_with(self.search_results_handle(), search_results_stream),
        ];

        let tick_ms = if self.needs_fast_tick() {
            ACTIVE_TICK_MS
        } else {
            IDLE_TICK_MS
        };

        subscriptions.push(time::every(Duration::from_millis(tick_ms)).map(|_| Message::Tick));

        Subscription::batch(subscriptions)
    }
}
