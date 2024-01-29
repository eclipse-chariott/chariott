// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use examples_common::intent_brokering::value::Value;
use intent_brokering_common::error::{Error, ResultExt};
use std::{env, sync::Arc};
use tokio::sync::broadcast::{self, Sender};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::intent_provider::{
    StreamingStore, ACTIVATE_AIR_CONDITIONING_COMMAND, AIR_CONDITIONING_STATE_PROPERTY,
    BATTERY_LEVEL_PROPERTY, CABIN_TEMPERATURE_PROPERTY, SEND_NOTIFICATION_COMMAND,
    SET_UI_MESSAGE_COMMAND,
};

const DEFAULT_COMMAND_CHANNEL_SIZE: usize = 200;

#[derive(Clone)]
pub struct VehicleSimulation {
    key_value_store: Arc<StreamingStore>,
    cmd_sender: Sender<Action>,
}

impl VehicleSimulation {
    pub fn new(key_value_store: Arc<StreamingStore>) -> Self {
        let command_channel_size = env::var("COMMAND_CHANNEL_SIZE")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(DEFAULT_COMMAND_CHANNEL_SIZE);

        let (cmd_sender, _) = broadcast::channel(command_channel_size);

        VehicleSimulation { key_value_store, cmd_sender }
    }

    pub async fn execute(
        &self,
        cancellation_token: CancellationToken,
    ) -> Result<(), anyhow::Error> {
        let mut vehicle_state = VehicleState::new();
        let (done_tx, mut done_rx) = broadcast::channel(1);
        // Ensure cancellation does not leak via cancellation_token. If we do
        // not create a child but use the passed cancellation_token directly to
        // cancel the handle_input, the caller might get notified of "internal"
        // cancellations.
        let scoped_shutdown_token = cancellation_token.child_token();

        let input_handle = tokio::task::spawn(handle_input(
            self.cmd_sender.clone(),
            scoped_shutdown_token.clone(),
            Done(done_tx),
        ));

        tokio::pin!(input_handle);

        enum LoopBreaker {
            Publisher(anyhow::Error),
            Handler(Result<Result<(), anyhow::Error>, tokio::task::JoinError>),
        }

        let mut cmd_receiver = self.cmd_sender.subscribe();

        let res = loop {
            tokio::select! {
                command = cmd_receiver.recv() => {
                    if let Ok(command) = command {
                        match command {
                            Action::Temperature(value) => vehicle_state.temperature = value,
                            Action::BatteryLevel(value) => vehicle_state.battery_level = value,
                            Action::AirConditioning(value) => vehicle_state.air_conditioning_enabled = value,
                        }
                        if let Err(err) = self.publish_data(&vehicle_state) {
                            break Some(LoopBreaker::Publisher(err));
                        }
                    } else {
                        break None;
                    }
                }
                res = &mut input_handle => {
                    break Some(LoopBreaker::Handler(res))
                }
                _ = scoped_shutdown_token.cancelled() => {
                    break Some(LoopBreaker::Handler(Ok(Ok(()))))
                }
            }
        };

        scoped_shutdown_token.cancel();
        debug!("Waiting for all tasks to shutdown.");

        _ = done_rx.recv().await.unwrap_err();
        debug!("Shutdown complete.");

        use LoopBreaker::*;

        match res {
            Some(Publisher(err)) => Err(err),
            Some(Handler(Ok(ok @ Ok(_)))) => ok,
            Some(Handler(Ok(err @ Err(_)))) => err,
            Some(Handler(Err(err))) => Err(err.into()),
            None => Ok(()),
        }
    }

    fn publish_data(&self, vehicle_state: &VehicleState) -> Result<(), anyhow::Error> {
        let publish = |event_id: &str, data: Value| {
            self.key_value_store.set(event_id.into(), data);
        };

        publish(CABIN_TEMPERATURE_PROPERTY, vehicle_state.temperature.into());
        publish(AIR_CONDITIONING_STATE_PROPERTY, vehicle_state.air_conditioning_enabled.into());
        publish(BATTERY_LEVEL_PROPERTY, vehicle_state.battery_level.into());

        Ok(())
    }

    pub async fn invoke(&self, command: &str, args: Vec<Value>) -> Result<Value, Error> {
        let action = match (command, args.as_slice()) {
            (ACTIVATE_AIR_CONDITIONING_COMMAND, [value]) => {
                let value =
                    value.to_bool().map_err(|_| Error::new("Argument must be of type 'Bool'."))?;
                info!("Set air conditioning: {}", value);
                Ok(Some(Action::AirConditioning(value)))
            }
            (SEND_NOTIFICATION_COMMAND, [value]) => {
                let value =
                    value.as_str().map_err(|_| Error::new("Argument must be of type 'String'."))?;
                info!("Sending notification: {}", value);
                Ok(None)
            }
            (SET_UI_MESSAGE_COMMAND, [value]) => {
                let value =
                    value.as_str().map_err(|_| Error::new("Argument must be of type 'String'."))?;
                info!("Setting message in UI: {}", value);
                Ok(None)
            }
            _ => Err(Error::new("No command found which accepts the invocation arguments.")),
        }?;

        if let Some(action) = action {
            self.cmd_sender.send(action).map_err_with("Error when sending a command.")?;
        }

        Ok(Value::TRUE)
    }
}

// Emulates the state of a car:
// Function invocations cause the state to update.
// "Emulation" causes the state to update (e.g. battery drains over time, temperature changes over time).
struct VehicleState {
    temperature: i32,
    battery_level: i32,
    air_conditioning_enabled: bool,
}

impl VehicleState {
    fn new() -> Self {
        Self { temperature: 20, battery_level: 100, air_conditioning_enabled: false }
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Temperature(i32),
    BatteryLevel(i32),
    AirConditioning(bool),
}

async fn handle_input(
    sender: Sender<Action>,
    shutdown_token: CancellationToken,
    _done: Done,
) -> Result<(), anyhow::Error> {
    use async_std::{
        io::{prelude::BufReadExt, stdin, BufReader},
        stream::StreamExt,
    };
    use Action::*;

    info!("-- Data update input ready --");

    let stdin = BufReader::new(stdin());
    let mut lines = stdin.lines();

    loop {
        let input = tokio::select! {
            line = lines.next() => line,
            _ = shutdown_token.cancelled() => break,
        };

        if let Some(input) = input {
            let input = input?;
            let input_list: Vec<&str> = input.split(' ').collect();
            let data_type = input_list[0].to_lowercase();
            if let Some(b'#') = data_type.as_bytes().first() {
                continue;
            }
            let data: Box<str> = input_list[1].to_lowercase().trim().into();

            let command = match data_type.as_str() {
                "temp" => Temperature(str::parse::<i32>(&data).unwrap_or(25)),
                "air_conditioning" => AirConditioning(data.as_ref() == "on"),
                "battery" => BatteryLevel(str::parse::<i32>(&data).unwrap_or(100)),
                _ => {
                    info!("No data update triggered, due to wrong input");
                    continue;
                }
            };

            sender.send(command)?;
        } else {
            break;
        }
    }

    debug!("Shutting down input handler.");
    Ok(())
}

struct Done(broadcast::Sender<()>);
