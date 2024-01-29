// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Error};
use dog_mode_status::stream_dog_mode_status;
use examples_common::intent_brokering::{
    api::{GrpcIntentBrokering, IntentBrokering, IntentBrokeringExt as _},
    value::Value,
};
use tokio::{select, time::sleep_until};
use tokio_stream::StreamExt;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

mod dog_mode_status;

// Namespaces
const VDT_NAMESPACE: &str = "sdv.vdt";
pub const KEY_VALUE_STORE_NAMESPACE: &str = "sdv.kvs";

// Dog mode boundary conditions
const LOW_BATTERY_LEVEL: i32 = 19;
const MIN_TEMPERATURE: i32 = 20;
const MAX_TEMPERATURE: i32 = 26;

// Method names
const ACTIVATE_AIR_CONDITIONING_ID: &str = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
const SEND_NOTIFICATION_ID: &str = "send_notification";
const SET_UI_MESSAGE_ID: &str = "set_ui_message";

// Event identifiers
pub const DOG_MODE_STATUS_ID: &str = "Feature.DogMode.Status";
const CABIN_TEMPERATURE_ID: &str = "Vehicle.Cabin.HVAC.AmbientAirTemperature";
const AIR_CONDITIONING_STATE_ID: &str = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
const BATTERY_LEVEL_ID: &str = "Vehicle.OBD.HybridBatteryRemaining";

static FUNCTION_INVOCATION_THROTTLING_DURATION: Duration = Duration::from_secs(5);
static AIR_CONDITIONING_ACTIVATION_TIMEOUT: Duration = Duration::from_secs(10);
static TIMEOUT_EVALUATION_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DogModeState {
    temperature: i32,
    dogmode_status: bool,
    battery_level: i32,
    air_conditioning_active: bool,
    air_conditioning_activation_time: Option<Instant>,
    last_air_conditioning_invocation_time: Instant,
    write_dog_mode_status: bool,
    send_notification_disabled: bool,
    set_ui_message_disabled: bool,
}

impl DogModeState {
    pub fn new() -> Self {
        Self {
            temperature: 25,
            air_conditioning_active: false,
            dogmode_status: false,
            battery_level: 100,
            air_conditioning_activation_time: None,
            last_air_conditioning_invocation_time: Instant::now()
                - FUNCTION_INVOCATION_THROTTLING_DURATION,
            write_dog_mode_status: false,
            send_notification_disabled: false,
            set_ui_message_disabled: false,
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder().with_default_directive(Level::INFO.into()).from_env_lossy(),
        )
        .finish()
        .init();

    let mut intent_broker = GrpcIntentBrokering::connect().await?;
    let mut state = DogModeState::new();

    // Inspect vehicle hardware to assert that all requirements are met before
    // executing the dog mode logic.

    {
        const MEMBER_TYPE: &str = "member_type";
        const TYPE: &str = "type";
        const MEMBER_TYPE_COMMAND: &str = "command";
        const MEMBER_TYPE_PROPERTY: &str = "property";

        for (path, member_type, r#type) in [
            (CABIN_TEMPERATURE_ID, /*..........*/ MEMBER_TYPE_PROPERTY, "int32"),
            (AIR_CONDITIONING_STATE_ID, /*.....*/ MEMBER_TYPE_PROPERTY, "bool"),
            (BATTERY_LEVEL_ID, /*..............*/ MEMBER_TYPE_PROPERTY, "int32"),
            (ACTIVATE_AIR_CONDITIONING_ID, /*..*/ MEMBER_TYPE_COMMAND, "IAcmeAirconControl"),
        ] {
            inspect_dependency(
                &mut intent_broker,
                path,
                &[(MEMBER_TYPE, member_type.into()), (TYPE, /*..*/ r#type.into())],
            )
            .await?;
        }

        for (path, r#type, state) in [
            (SEND_NOTIFICATION_ID, "ISendNotification", &mut state.send_notification_disabled),
            (SET_UI_MESSAGE_ID, "ISetUiMessage", &mut state.set_ui_message_disabled),
        ] {
            if let Err(e) = inspect_dependency(
                &mut intent_broker,
                path,
                &[(MEMBER_TYPE, MEMBER_TYPE_COMMAND.into()), (TYPE, /*..*/ r#type.into())],
            )
            .await
            {
                warn!("Error when inspecting for optional dependency {path}: '{e:?}'.");
                *state = true
            }
        }

        if state.send_notification_disabled && state.set_ui_message_disabled {
            Err(anyhow!("Neither {SEND_NOTIFICATION_ID} nor {SET_UI_MESSAGE_ID} are available",))?;
        }

        async fn inspect_dependency(
            intent_broker: &mut GrpcIntentBrokering,
            path: impl Into<Box<str>> + Send,
            expected_properties: &[(&str, Value)],
        ) -> Result<(), Error> {
            let inspection_result = intent_broker.inspect(VDT_NAMESPACE, path).await?;
            inspection_result.iter().map(|m| {
                expected_properties.iter().map(|(expected_key, expected_value)| m.get(*expected_key)
                    .ok_or_else(|| anyhow!("Member does not specify {expected_key:?}."))
                    .and_then(|actual| {
                    if expected_value != actual {
                        Err(anyhow!(
                            "Member is of {expected_key} '{actual:?}' instead of '{expected_value:?}'."
                        ))
                    }
                    else {
                        Ok(())
                    }
                })).reduce(|x, y| x.and(y)).unwrap_or_else(|| Err(anyhow!("Expected properties array was empty.")))
            }).fold(Err(anyhow!("Could not find a single member within the specified path.")), |x, y| x.or(y))
        }
    }

    // Set up the VDT and dog mode status streaming.

    let mut vdt_stream = intent_broker
        .listen(
            VDT_NAMESPACE,
            [
                CABIN_TEMPERATURE_ID.into(),
                AIR_CONDITIONING_STATE_ID.into(),
                BATTERY_LEVEL_ID.into(),
            ],
        )
        .await?;

    let mut dog_mode_status_stream =
        stream_dog_mode_status(intent_broker.clone(), &mut state).await?;

    let mut next_timer_wakeup = Instant::now() + TIMEOUT_EVALUATION_INTERVAL;

    loop {
        // Keeping track of the previous state allows us to use Markov chain style business logic.
        let previous_state = state;

        let state_update = select! {
            _ = sleep_until(next_timer_wakeup.into()) => {
                next_timer_wakeup = Instant::now() + TIMEOUT_EVALUATION_INTERVAL;
                if let Some(new_state) = on_dog_mode_timer(&state, &mut intent_broker).await? {
                    state = new_state;
                }

                None
            },
            dog_mode_status = dog_mode_status_stream.next() => {
                if let Some(dog_mode_status) = dog_mode_status {
                    match dog_mode_status {
                        Ok(dog_mode_status) => Some(DogModeState { dogmode_status: dog_mode_status, ..state  }),
                        Err(err) => { error!("Error when handling dog mode status update: '{err:?}'."); None }
                    }
                } else {
                    return Err(anyhow!("Dog mode status stream broke."));
                }
            }
            event = vdt_stream.next() => {
                if let Some(event) = event {
                    match event {
                        Ok(event) => match event.id.as_ref() {
                            BATTERY_LEVEL_ID => event.data.to_i32().ok().map(|battery_level| DogModeState { battery_level, ..state  }),
                            AIR_CONDITIONING_STATE_ID => event.data.to_bool().ok().map(|air_conditioning_active| DogModeState { air_conditioning_active, ..state  }),
                            CABIN_TEMPERATURE_ID => event.data.to_i32().ok().map(|temperature| DogModeState { temperature, ..state  }),
                            method => { error!("No method '{method}' found."); None }
                        }
                        Err(err) => { error!("Error when handling event: '{err:?}'."); None }
                    }
                }
                else {
                    return Err(anyhow!("Event stream broke."));
                }
            }
        };

        if let Some(new_state) = state_update {
            state = new_state
        }

        match run_dog_mode(&state, &previous_state, &mut intent_broker).await {
            Ok(Some(new_state)) => state = new_state,
            Err(e) => error!("{e:?}"),
            Ok(None) => {}
        }
    }
}

async fn run_dog_mode(
    state: &DogModeState,
    previous_state: &DogModeState,
    intent_broker: &mut impl IntentBrokering,
) -> Result<Option<DogModeState>, Error> {
    if state == previous_state {
        return Ok(None);
    }

    fn log_change<T: Eq + Display>(
        label: &str,
        curr: &DogModeState,
        prev: &DogModeState,
        f: fn(&DogModeState) -> T,
    ) -> Result<(), Error> {
        let v = f(curr);
        if v != f(prev) {
            info!("{label}: {v}");
        }
        Ok(())
    }

    log_change("Dog mode", state, previous_state, |s| s.dogmode_status)?;
    log_change("Cabin Temperature", state, previous_state, |s| s.temperature)?;
    log_change("Air conditioning", state, previous_state, |s| s.air_conditioning_active)?;
    log_change("Battery level", state, previous_state, |s| s.battery_level)?;

    if state.write_dog_mode_status && (state.dogmode_status != previous_state.dogmode_status) {
        intent_broker
            .write(KEY_VALUE_STORE_NAMESPACE, DOG_MODE_STATUS_ID, state.dogmode_status.into())
            .await?;
    }

    // Immediately end, if dog mode is disabled
    if !state.dogmode_status {
        if previous_state.dogmode_status {
            activate_air_conditioning(intent_broker, false).await?;
        }

        return Ok(None);
    }

    let mut output_state = None;

    // If the temperature falls below the set minimum, turn off air conditioning
    if MIN_TEMPERATURE >= state.temperature && state.air_conditioning_active {
        if let Some(last_air_conditioning_invocation_time) =
            activate_air_conditioning_with_throttling(false, state, intent_broker).await?
        {
            output_state = Some(DogModeState { last_air_conditioning_invocation_time, ..*state });
        }
    }

    // If all criteria is fulfilled, activate air conditioning
    if state.temperature > MAX_TEMPERATURE && !state.air_conditioning_active {
        if let Some(last_air_conditioning_invocation_time) =
            activate_air_conditioning_with_throttling(true, state, intent_broker).await?
        {
            output_state = Some(DogModeState {
                last_air_conditioning_invocation_time,
                air_conditioning_activation_time: Some(last_air_conditioning_invocation_time),
                ..*state
            });
        }
    }

    async fn activate_air_conditioning_with_throttling(
        value: bool,
        state: &DogModeState,
        intent_broker: &mut impl IntentBrokering,
    ) -> Result<Option<Instant>, Error> {
        let now = Instant::now();
        if now
            > state.last_air_conditioning_invocation_time + FUNCTION_INVOCATION_THROTTLING_DURATION
        {
            activate_air_conditioning(intent_broker, value).await?;
            return Ok(Some(now));
        }

        Ok(None)
    }

    // Air conditioning state was changed by the provider.
    if state.air_conditioning_active && !previous_state.air_conditioning_active {
        send_notification(intent_broker, "The car is now being cooled.", state).await?;
        set_ui_message(intent_broker, "The car is cooled, no need to worry.", state).await?;
    }

    // If the battery level fell below a threshold value, send a warning to the car owner.
    if previous_state.battery_level > LOW_BATTERY_LEVEL && state.battery_level <= LOW_BATTERY_LEVEL
    {
        send_notification(intent_broker, "The battery is low, please return to the car.", state)
            .await?;
        set_ui_message(intent_broker, "The battery is low, the animal is in danger.", state)
            .await?;
    }

    async fn activate_air_conditioning(
        intent_broker: &mut impl IntentBrokering,
        value: bool,
    ) -> Result<(), Error> {
        _ = intent_broker
            .invoke(VDT_NAMESPACE, ACTIVATE_AIR_CONDITIONING_ID, [value.into()])
            .await?;
        Ok(())
    }

    async fn send_notification(
        intent_broker: &mut impl IntentBrokering,
        message: &str,
        state: &DogModeState,
    ) -> Result<(), Error> {
        if !state.send_notification_disabled {
            _ = intent_broker.invoke(VDT_NAMESPACE, SEND_NOTIFICATION_ID, [message.into()]).await?;
            Ok(())
        } else {
            // as this is an optional method we don't care
            Ok(())
        }
    }

    async fn set_ui_message(
        intent_broker: &mut impl IntentBrokering,
        message: &str,
        state: &DogModeState,
    ) -> Result<(), Error> {
        if !state.set_ui_message_disabled {
            _ = intent_broker.invoke(VDT_NAMESPACE, SET_UI_MESSAGE_ID, [message.into()]).await?;
            Ok(())
        } else {
            // as this is an optional method we don't care
            Ok(())
        }
    }

    Ok(output_state)
}

async fn on_dog_mode_timer(
    state: &DogModeState,
    intent_broker: &mut impl IntentBrokering,
) -> Result<Option<DogModeState>, Error> {
    if let Some(air_conditioning_activation_time) = state.air_conditioning_activation_time {
        if state.air_conditioning_active {
            return Ok(Some(DogModeState { air_conditioning_activation_time: None, ..*state }));
        } else if Instant::now()
            > air_conditioning_activation_time + AIR_CONDITIONING_ACTIVATION_TIMEOUT
        {
            _ = intent_broker
                .invoke(
                    VDT_NAMESPACE,
                    SEND_NOTIFICATION_ID,
                    ["Error while activating air conditioning, please return to the car immediately.".into()],
                )
                .await?;

            return Ok(Some(DogModeState { air_conditioning_activation_time: None, ..*state }));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use examples_common::intent_brokering::{api::Service, inspection::Entry};
    use intent_brokering_common::error::Error;

    use super::*;

    #[derive(PartialEq, Eq, Debug, Default)]
    struct CarControllerMock {
        ui_message: Option<String>,
        notification: Option<String>,
        air_conditioning_state: Option<bool>,
    }

    #[async_trait]
    impl IntentBrokering for CarControllerMock {
        async fn invoke<I: IntoIterator<Item = Value> + Send>(
            &mut self,
            namespace: impl Into<Box<str>> + Send,
            command: impl Into<Box<str>> + Send,
            args: I,
        ) -> Result<Value, Error> {
            let arg = args.into_iter().next().unwrap();
            match (namespace.into().as_ref(), command.into().as_ref()) {
                (VDT_NAMESPACE, SET_UI_MESSAGE_ID) => {
                    if let Ok(value) = arg.into_string() {
                        self.ui_message = Some(value);
                    }
                }
                (VDT_NAMESPACE, SEND_NOTIFICATION_ID) => {
                    if let Ok(value) = arg.into_string() {
                        self.notification = Some(value);
                    }
                }
                (VDT_NAMESPACE, ACTIVATE_AIR_CONDITIONING_ID) => {
                    if let Ok(value) = arg.to_bool() {
                        self.air_conditioning_state = Some(value);
                    }
                }
                _ => {}
            }

            Ok(Value::TRUE)
        }

        async fn subscribe<I: IntoIterator<Item = Box<str>> + Send>(
            &mut self,
            _namespace: impl Into<Box<str>> + Send,
            _channel_id: impl Into<Box<str>> + Send,
            _event_ids: I,
        ) -> Result<(), Error> {
            todo!()
        }

        async fn discover(
            &mut self,
            _namespace: impl Into<Box<str>> + Send,
        ) -> Result<Vec<Service>, Error> {
            todo!()
        }

        async fn inspect(
            &mut self,
            _namespace: impl Into<Box<str>> + Send,
            _query: impl Into<Box<str>> + Send,
        ) -> Result<Vec<Entry>, Error> {
            todo!()
        }

        async fn write(
            &mut self,
            _: impl Into<Box<str>> + Send,
            _: impl Into<Box<str>> + Send,
            _: Value,
        ) -> Result<(), Error> {
            todo!()
        }

        async fn read(
            &mut self,
            _: impl Into<Box<str>> + Send,
            _: impl Into<Box<str>> + Send,
        ) -> Result<Option<Value>, Error> {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_dog_mode_activation_has_no_effect_when_no_conditions_are_met() {
        let mut car_controller: CarControllerMock = Default::default();

        let original_state = DogModeState::new();

        // Act
        let state = DogModeState { dogmode_status: true, ..original_state };

        let result = run_dog_mode(&state, &original_state, &mut car_controller).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(<CarControllerMock as Default>::default(), car_controller);
    }

    #[tokio::test]
    async fn test_air_con_is_turned_on_when_temperature_exceeds_max_threshold() {
        let mut car_controller = Default::default();

        let original_state = DogModeState {
            temperature: MAX_TEMPERATURE,
            battery_level: 100,
            dogmode_status: true,
            ..DogModeState::new()
        };

        // Act
        let state = DogModeState { temperature: MAX_TEMPERATURE + 1, ..original_state };

        let result = run_dog_mode(&state, &original_state, &mut car_controller).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            CarControllerMock { air_conditioning_state: Some(true), ..Default::default() },
            car_controller
        );
    }

    #[tokio::test]
    async fn test_user_is_notified_when_air_con_is_reported_to_be_on() {
        let mut car_controller = Default::default();

        let original_state = DogModeState {
            temperature: MAX_TEMPERATURE + 1,
            battery_level: 100,
            dogmode_status: true,
            ..DogModeState::new()
        };

        // Act
        let state = DogModeState { air_conditioning_active: true, ..original_state };

        let result = run_dog_mode(&state, &original_state, &mut car_controller).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            CarControllerMock {
                air_conditioning_state: None,
                notification: Some("The car is now being cooled.".to_string()),
                ui_message: Some("The car is cooled, no need to worry.".to_string())
            },
            car_controller
        );
    }

    #[tokio::test]
    async fn test_user_is_notified_when_battery_is_low() {
        let mut car_controller = Default::default();

        let original_state = DogModeState {
            dogmode_status: true,
            temperature: MAX_TEMPERATURE + 1,
            battery_level: LOW_BATTERY_LEVEL + 1,
            ..DogModeState::new()
        };

        // Act
        let state = DogModeState { battery_level: LOW_BATTERY_LEVEL, ..original_state };

        let result = run_dog_mode(&state, &original_state, &mut car_controller).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            CarControllerMock {
                air_conditioning_state: Some(true),
                notification: Some("The battery is low, please return to the car.".to_string()),
                ui_message: Some("The battery is low, the animal is in danger.".to_string())
            },
            car_controller
        );
    }

    #[tokio::test]
    async fn test_air_con_is_turned_off_when_temperature_below_min_threshold() {
        let mut car_controller = Default::default();

        let original_state = DogModeState {
            dogmode_status: true,
            temperature: MIN_TEMPERATURE,
            air_conditioning_active: true,
            ..DogModeState::new()
        };

        // Act
        let state = DogModeState { temperature: MIN_TEMPERATURE - 1, ..original_state };

        let result = run_dog_mode(&state, &original_state, &mut car_controller).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            CarControllerMock { air_conditioning_state: Some(false), ..Default::default() },
            car_controller
        );
    }

    #[tokio::test]
    async fn air_conditioning_activation_is_set_when_air_conditioning_turned_on() {
        // arrange
        let mut car_controller: CarControllerMock = Default::default();

        let original_state = DogModeState {
            dogmode_status: true,
            air_conditioning_active: false,
            ..DogModeState::new()
        };

        let state = DogModeState { temperature: MAX_TEMPERATURE + 1, ..original_state };

        // act
        let result = run_dog_mode(&state, &original_state, &mut car_controller).await;

        // assert
        assert_instant(
            Instant::now(),
            result.unwrap().unwrap().air_conditioning_activation_time.unwrap(),
            Duration::from_secs(5),
        );
    }

    #[tokio::test]
    async fn notification_is_sent_when_air_conditioning_timeout_expires() {
        // arrange
        let mut car_controller: CarControllerMock = Default::default();
        let state = DogModeState {
            air_conditioning_active: false,
            air_conditioning_activation_time: Some(
                Instant::now() - AIR_CONDITIONING_ACTIVATION_TIMEOUT * 2,
            ),
            ..DogModeState::new()
        };

        // act
        _ = on_dog_mode_timer(&state, &mut car_controller).await;

        // assert
        assert_eq!(
            CarControllerMock {
                notification: Some("Error while activating air conditioning, please return to the car immediately.".to_owned()),
                ..Default::default()
            },
            car_controller,
        );
    }

    #[tokio::test]
    async fn air_conditioning_activation_timestamp_is_reset_when_air_conditioning_activation_timeout_expires(
    ) {
        // arrange
        let mut car_controller: CarControllerMock = Default::default();
        let state = DogModeState {
            air_conditioning_active: false,
            air_conditioning_activation_time: Some(
                Instant::now() - AIR_CONDITIONING_ACTIVATION_TIMEOUT * 2,
            ),
            ..DogModeState::new()
        };

        // act
        let result = on_dog_mode_timer(&state, &mut car_controller).await;

        // assert
        assert_eq!(None, result.unwrap().unwrap().air_conditioning_activation_time);
    }

    #[tokio::test]
    async fn air_conditioning_activation_timestamp_is_reset_when_air_conditioning_is_activated() {
        // arrange
        let mut car_controller: CarControllerMock = Default::default();
        let state = DogModeState {
            air_conditioning_active: true,
            air_conditioning_activation_time: Some(Instant::now()),
            ..DogModeState::new()
        };

        // act
        let result = on_dog_mode_timer(&state, &mut car_controller).await;

        // assert
        assert_eq!(None, result.unwrap().unwrap().air_conditioning_activation_time);
    }

    #[tokio::test]
    async fn air_conditioning_should_throttle_function_invocations() {
        // arrange
        let mut car_controller: CarControllerMock = Default::default();
        let now = Instant::now();
        let state = DogModeState {
            temperature: 40,
            dogmode_status: true,
            air_conditioning_active: false,
            last_air_conditioning_invocation_time: now,
            ..DogModeState::new()
        };

        // act
        let result = run_dog_mode(&state, &DogModeState::new(), &mut car_controller).await;

        // assert
        assert_eq!(None, result.unwrap());
    }

    #[tokio::test]
    async fn air_conditioning_should_invoke_function_after_throttling_expired() {
        // arrange
        let mut car_controller: CarControllerMock = Default::default();
        let previous_state =
            DogModeState { temperature: 40, dogmode_status: true, ..DogModeState::new() };
        let state = DogModeState {
            temperature: 15,
            dogmode_status: true,
            air_conditioning_active: true,
            last_air_conditioning_invocation_time: Instant::now()
                - FUNCTION_INVOCATION_THROTTLING_DURATION,
            ..DogModeState::new()
        };

        // act
        let result = run_dog_mode(&state, &previous_state, &mut car_controller).await;

        // assert
        let result = result.unwrap().unwrap();
        assert_instant(
            Instant::now(),
            result.last_air_conditioning_invocation_time,
            Duration::from_secs(3),
        );

        assert!(result.air_conditioning_active);
    }

    fn assert_instant(expected: Instant, actual: Instant, margin: Duration) {
        assert!(actual < expected + margin);
        assert!(expected - margin < actual);
    }
}
