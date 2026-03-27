mod handle;
mod stream;

use crate::core::app_command::AppCommand;
use crate::core::ipc::IpcCommand;
use crate::core::search::ApplicationSearchResponse;
use std::sync::mpsc::Receiver;

use handle::ReceiverHandle;

pub(super) use stream::{app_command_stream, ipc_command_stream, search_results_stream};

const IPC_SUBSCRIPTION_ID: u64 = 1;
const APP_COMMAND_SUBSCRIPTION_ID: u64 = IPC_SUBSCRIPTION_ID + 1;
const APP_SEARCH_SUBSCRIPTION_ID: u64 = IPC_SUBSCRIPTION_ID + 2;

pub(super) type IpcReceiverHandle = ReceiverHandle<IpcCommand>;
pub(super) type AppCommandReceiverHandle = ReceiverHandle<AppCommand>;
pub(super) type SearchResultsReceiverHandle = ReceiverHandle<ApplicationSearchResponse>;

pub(super) fn new_ipc_receiver_handle(receiver: Receiver<IpcCommand>) -> IpcReceiverHandle {
    ReceiverHandle::new(IPC_SUBSCRIPTION_ID, receiver)
}

pub(super) fn new_app_command_handle(receiver: Receiver<AppCommand>) -> AppCommandReceiverHandle {
    ReceiverHandle::new(APP_COMMAND_SUBSCRIPTION_ID, receiver)
}

pub(super) fn new_search_results_handle(
    receiver: Receiver<ApplicationSearchResponse>,
) -> SearchResultsReceiverHandle {
    ReceiverHandle::new(APP_SEARCH_SUBSCRIPTION_ID, receiver)
}
