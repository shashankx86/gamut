use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(in crate::ui) struct ReceiverHandle<T> {
    id: u64,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> ReceiverHandle<T> {
    pub(in crate::ui) fn new(id: u64, receiver: Receiver<T>) -> Self {
        Self {
            id,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub(in crate::ui) fn receiver(&self) -> Arc<Mutex<Receiver<T>>> {
        Arc::clone(&self.receiver)
    }
}

impl<T> Hash for ReceiverHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
