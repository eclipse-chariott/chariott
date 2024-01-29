// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_stream::try_stream;
use core::panic;
use examples_common::intent_brokering::value::Value;
use intent_brokering_common::error::Error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::select;
use tokio::time::sleep;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::intent_provider::StreamingStore;

pub struct CameraLogic {
    store: Arc<StreamingStore>,
    images_directory: PathBuf,
}

impl CameraLogic {
    pub fn new(store: Arc<StreamingStore>) -> Result<Self, Error> {
        let images_directory: PathBuf = std::env::var("SIMULATED_CAMERA_APP_IMAGES_DIRECTORY")
            .unwrap_or_else(|_| "./images".to_owned())
            .into();

        if !images_directory.is_dir() {
            return Err(Error::new(format!(
                "Images directory {images_directory:?} does not exist."
            )));
        }

        Ok(CameraLogic { store, images_directory })
    }

    // A note for the reader of this example
    // The code that follows is loading in memory all the images included in images folder
    // The approach taken here was aiming to simplicity, while sacrificing the memory consumption
    // A more memory efficient approach would have been to just take the file names and stream them one by one when needed in camera_loop
    fn get_images_from_folder(&self) -> Result<Vec<Value>, Error> {
        let images_directory = std::fs::read_dir(self.images_directory.clone())?;

        let mut images = vec![];
        for entry in images_directory {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let file_bytes = std::fs::read(entry.path())?;
            images.push(Value::new_blob("images/jpeg".to_owned(), file_bytes));
        }

        if images.is_empty() {
            panic!("No images");
        }

        Ok(images)
    }

    pub async fn camera_loop(&self, cancellation_token: CancellationToken) -> Result<(), Error> {
        let images = self.get_images_from_folder()?;
        let mut cycle = images.iter().cycle();
        let mut hashmap: HashMap<Box<str>, (Instant, Duration)> = HashMap::new();
        hashmap.insert("camera.2fpm".into(), (Instant::now(), Duration::from_secs(30)));
        hashmap.insert("camera.6fpm".into(), (Instant::now(), Duration::from_secs(10)));
        hashmap.insert("camera.12fpm".into(), (Instant::now(), Duration::from_secs(5)));
        let loop_cycle = Duration::from_secs(1);

        loop {
            // For simplicity, we are looping through photos only if a timer elapsed
            let mut cycled = false;
            let mut event = Value::new_blob("".to_owned(), vec![]);
            for (event_id, (last_occurrency, interval)) in &mut hashmap {
                if last_occurrency.elapsed().gt(interval) {
                    if !cycled {
                        event = cycle.next().unwrap().to_owned();
                        cycled = true;
                    }
                    self.store.set(event_id.to_owned(), event.clone());
                    *last_occurrency = Instant::now();
                }
            }

            select! {
                _ = sleep(loop_cycle) => {},
                _ = cancellation_token.cancelled() => { break; }
            }
        }

        Ok(())
    }

    pub async fn execute(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        let input_stream = handle_input(cancellation_token.clone());

        tokio::pin!(input_stream);

        while let Some(items) = input_stream.next().await {
            match items {
                Ok((image, frame_rate)) => {
                    let file_bytes = std::fs::read(image.clone());
                    if file_bytes.is_err() {
                        error!("Error reading file: {}", image);
                        continue;
                    }
                    let image = Value::new_blob("images/jpeg".to_owned(), file_bytes.unwrap());
                    self.store.set(frame_rate.into(), image);
                }
                Err(err) => {
                    error!("Error reading stream: {}", err);
                    break;
                }
            }
        }

        Ok(())
    }
}

// In 1.64 or later clippy checks get_first
// as described in this lint rule https://rust-lang.github.io/rust-clippy/master/index.html#get_first
// try_stream macro is still using accessing first element with `$crate::async_stream_impl::try_stream_inner!(($crate) $($tt)*).get(0)`
#[allow(clippy::get_first)]
fn handle_input(
    shutdown_token: CancellationToken,
) -> impl Stream<Item = Result<(String, String), anyhow::Error>> {
    use async_std::io::{prelude::BufReadExt, stdin, BufReader};

    info!("-- Data update input ready --");

    let stdin = BufReader::new(stdin());
    let mut lines = stdin.lines();

    try_stream! {
        loop {
            let input = tokio::select! {
                line = lines.next() => line,
                _ = shutdown_token.cancelled() => break,
            };

            if let Some(input) = input {
                let input = input?;
                let input_list: Vec<&str> = input.split(' ').collect();
                if input_list.len() != 3 {
                    warn!("Please use 'load image_path frame_rate' as input format!");
                    continue;
                }
                let data_type = input_list[0].to_lowercase();
                if let Some(b'#') = data_type.as_bytes().get(0) {
                    continue;
                }
                let image: Box<str> = input_list[1].to_lowercase().trim().into();
                let frame_rate: Box<str> = input_list[2].to_lowercase().trim().into();

                yield (image.into(), frame_rate.into())
            } else {
                debug!("Shutting down input handler.");
                break;
            }
        }
    }
}
