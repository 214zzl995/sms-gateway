use tokio::sync::broadcast;

use crate::db::Conversation;

#[derive(Clone)]
pub struct SseManager {
    tx: broadcast::Sender<Vec<Conversation>>,
}

impl SseManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<Conversation>> {
        self.tx.subscribe()
    }

    pub fn send(&self, msg: Vec<Conversation>) {
        let _ = self.tx.send(msg);
    }
}
