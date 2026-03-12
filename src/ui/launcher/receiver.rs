use super::{AppCommandReceiverHandle, IpcReceiverHandle, Message};
use iced::futures::{SinkExt, StreamExt, channel::mpsc, stream::BoxStream};
use iced::stream;

pub(super) fn ipc_command_stream(handle: &IpcReceiverHandle) -> BoxStream<'static, Message> {
    receiver_stream(handle.receiver.clone(), Message::IpcCommand)
}

pub(super) fn app_command_stream(handle: &AppCommandReceiverHandle) -> BoxStream<'static, Message> {
    receiver_stream(handle.receiver.clone(), Message::AppCommand)
}

fn receiver_stream<T, F>(
    receiver: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<T>>>,
    map: F,
) -> BoxStream<'static, Message>
where
    T: Send + 'static,
    F: Fn(T) -> Message + Send + 'static + Copy,
{
    stream::channel(100, async move |mut output| {
        let (tx, mut rx) = mpsc::unbounded::<T>();

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
            let _ = output.send(map(command)).await;
        }
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::Message;
    use crate::core::app_command::AppCommand;
    use crate::core::ipc::IpcCommand;

    #[test]
    fn message_mappers_preserve_payloads() {
        let ipc = IpcCommand::Ping;
        let app = AppCommand::Quit;

        match Message::IpcCommand(ipc.clone()) {
            Message::IpcCommand(value) => assert_eq!(value, ipc),
            _ => panic!("expected IPC command message"),
        }

        match Message::AppCommand(app.clone()) {
            Message::AppCommand(value) => assert_eq!(value, app),
            _ => panic!("expected app command message"),
        }
    }
}
