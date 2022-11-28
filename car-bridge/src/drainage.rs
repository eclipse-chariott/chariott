use futures::Future;
use tokio::{
    spawn,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

/// Spawn tasks on the `Drainage` to ensure that the task completed when
/// shutting down. Based on https://tokio.rs/tokio/topics/shutdown
pub struct Drainage {
    drained_receiver: Receiver<()>,
    drained_sender: Sender<()>,
}

impl Drainage {
    pub fn new() -> Self {
        let (drained_sender, drained_receiver) = mpsc::channel(1);
        Self { drained_receiver, drained_sender }
    }

    /// Spawns a task while allowing to wait for it to complete when calling
    /// `drain`.
    pub fn spawn<F>(&self, f: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        let sender = self.drained_sender.clone();

        spawn(async move {
            let result = f.await;
            drop(sender);
            result
        })
    }

    /// Waits for all tasks that were spawned via this drainage to finish.
    pub async fn drain(mut self) {
        drop(self.drained_sender);
        _ = self.drained_receiver.recv().await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Drainage;
    use tokio::time::sleep;
    use tokio::time::timeout;

    #[tokio::test]
    async fn drain_when_task_is_active_times_out() {
        let drainage = Drainage::new();
        _ = drainage.spawn(sleep(Duration::from_millis(500)));
        assert!(timeout(Duration::from_millis(100), drainage.drain()).await.is_err());
    }

    #[tokio::test]
    async fn drain_when_no_task_is_active_succeeds() {
        let drainage = Drainage::new();
        _ = drainage.spawn(sleep(Duration::from_millis(100)));
        assert!(timeout(Duration::from_millis(500), drainage.drain()).await.is_ok());
    }
}
