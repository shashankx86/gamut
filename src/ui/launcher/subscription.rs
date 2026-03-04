use super::{Launcher, Message};
use iced::{Event, Subscription, event, time, window};
use std::time::Duration;

impl Launcher {
    pub(in crate::ui) fn subscription(&self) -> Subscription<Message> {
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
            time::every(Duration::from_millis(25)).map(|_| Message::Tick),
        ])
    }
}
