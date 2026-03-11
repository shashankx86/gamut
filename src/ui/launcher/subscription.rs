use super::{AppCommandReceiverHandle, IpcReceiverHandle, Launcher, Message};
use crate::core::app_command::AppCommand;
use crate::core::ipc::IpcCommand;
use iced::futures::{SinkExt, StreamExt, channel::mpsc, stream::BoxStream};
use iced::{Event, Subscription, event, stream, time, window};
use std::time::Duration;

const ACTIVE_TICK_MS: u64 = 25;

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
        ];

        if self.needs_fast_tick() {
            subscriptions
                .push(time::every(Duration::from_millis(ACTIVE_TICK_MS)).map(|_| Message::Tick));
        }

        Subscription::batch(subscriptions)
    }
}

fn ipc_command_stream(handle: &IpcReceiverHandle) -> BoxStream<'static, Message> {
    let receiver = handle.receiver.clone();

    stream::channel(100, async move |mut output| {
        let (tx, mut rx) = mpsc::unbounded::<IpcCommand>();

        std::thread::spawn(move || {
            loop {
                let command = {
                    let Ok(guard) = receiver.lock() else {
                        break;
                    };
                    guard.recv()
                };

                match command {
                    Ok(command) => {
                        if tx.unbounded_send(command).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        while let Some(command) = rx.next().await {
            let _ = output.send(Message::IpcCommand(command)).await;
        }
    })
    .boxed()
}

fn app_command_stream(handle: &AppCommandReceiverHandle) -> BoxStream<'static, Message> {
    let receiver = handle.receiver.clone();

    stream::channel(100, async move |mut output| {
        let (tx, mut rx) = mpsc::unbounded::<AppCommand>();

        std::thread::spawn(move || {
            loop {
                let command = {
                    let Ok(guard) = receiver.lock() else {
                        break;
                    };
                    guard.recv()
                };

                match command {
                    Ok(command) => {
                        if tx.unbounded_send(command).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        while let Some(command) = rx.next().await {
            let _ = output.send(Message::AppCommand(command)).await;
        }
    })
    .boxed()
}
