use tokio::sync::broadcast;


use crate::db::SMSPreview;

#[derive(Clone)]
pub struct SseManager {
    tx: broadcast::Sender<SMSPreview>,
}

impl SseManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SMSPreview> {
        self.tx.subscribe()
    }

    pub fn send(&self, msg: SMSPreview) {
        let _ = self.tx.send(msg);
    }
}