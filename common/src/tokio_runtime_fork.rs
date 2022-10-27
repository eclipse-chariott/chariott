// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use tokio_util::sync::CancellationToken;

pub struct Fork {
    handle: tokio::runtime::Handle,
    cancellation_token: CancellationToken,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl Fork {
    pub fn handle(&self) -> &tokio::runtime::Handle {
        &self.handle
    }
}

impl Drop for Fork {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        std::mem::take(&mut self.join_handle)
            .unwrap()
            .join()
            .expect("Forked thread for Tokio runtime failed.");
    }
}

pub trait BuilderExt {
    fn fork(&mut self) -> std::io::Result<Fork>;
}

impl BuilderExt for tokio::runtime::Builder {
    fn fork(&mut self) -> std::io::Result<Fork> {
        let runtime = self.build()?;
        let handle = runtime.handle().clone();
        let cancellation_token = CancellationToken::new();
        let join_handle = {
            let cancellation_token = cancellation_token.clone();
            std::thread::spawn(move || {
                runtime.block_on(cancellation_token.cancelled());
            })
        };
        Ok(Fork { handle, cancellation_token, join_handle: Some(join_handle) })
    }
}
