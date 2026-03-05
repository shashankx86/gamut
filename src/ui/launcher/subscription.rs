use super::{Launcher, Message};
use iced::{Event, Subscription, event, time, window};
use std::time::Duration;

const ACTIVE_TICK_MS: u64 = 25;
const IDLE_TICK_MS: u64 = 250;

impl Launcher {
    pub(in crate::ui) fn subscription(&self) -> Subscription<Message> {
        let tick_rate = if self.needs_fast_tick() {
            Duration::from_millis(ACTIVE_TICK_MS)
        } else {
            Duration::from_millis(IDLE_TICK_MS)
        };

        Subscription::batch(vec![
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
            time::every(tick_rate).map(|_| Message::Tick),
        ])
    }
}
