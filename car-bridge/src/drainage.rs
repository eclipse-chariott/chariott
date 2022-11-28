// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use futures::Future;
use tokio::sync::mpsc::{self, Receiver, Sender};

/// Spawn tasks on the `Drainage` to ensure that the task completed when
/// shutting down.
pub struct Drainage {
    drained_receiver: Receiver<()>,
    drained_sender: Sender<()>,
}

impl Drainage {
    pub fn new() -> Self {
        let (drained_sender, drained_receiver) = mpsc::channel(1);
        Self { drained_receiver, drained_sender }
    }

    /// Tracks a future to allow waiting for it to complete.
    pub fn track<F>(&self, f: F) -> impl Future<Output = F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        let sender = self.drained_sender.clone();

        async move {
            let result = f.await;
            drop(sender);
            result
        }
    }

    /// Waits for all tasks that were spawned via this drainage to finish.
    pub async fn drain(mut self) {
        drop(self.drained_sender);
        // As soon as every sender is dropped, the channel is closed and the
        // reception fails. Based on:
        // https://tokio.rs/tokio/topics/shutdown
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/#disconnection
        _ = self.drained_receiver.recv().await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Drainage;
    use tokio::spawn;
    use tokio::time::sleep;
    use tokio::time::timeout;

    #[tokio::test]
    async fn drain_when_task_is_active_times_out() {
        let drainage = Drainage::new();
        _ = spawn(drainage.track(sleep(Duration::from_millis(500))));
        assert!(timeout(Duration::from_millis(100), drainage.drain()).await.is_err());
    }

    #[tokio::test]
    async fn drain_when_no_task_is_active_succeeds() {
        let drainage = Drainage::new();
        _ = spawn(drainage.track(sleep(Duration::from_millis(100))));
        assert!(timeout(Duration::from_millis(500), drainage.drain()).await.is_ok());
    }
}