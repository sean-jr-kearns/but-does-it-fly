mod gpio;

use communication_protocol::prelude::{
    Command, LightState, ProtocolError, decode_command, encode_state,
};
use communication_protocol::{COMMAND_CHARACTERISTIC_UUID, DEVICE_NAME, SERVICE_UUID};

use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicRead, CharacteristicWrite,
        CharacteristicWriteMethod, Service,
    },
};

use futures::FutureExt;
use std::sync::Arc;

use tokio::sync::Mutex;

use gpio::{Indicator, LightOutput};

const GPIO_PIN: u8 = 23;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    env_logger::init();

    let light = Indicator::new(GPIO_PIN).map_or_else(
        |_| {
            Err(ProtocolError::InvalidPeripheral(
                "failed to setup indicator".to_string(),
            ))
        },
        |light| Ok(Arc::new(Mutex::new(Box::new(light)))),
    );

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;

    adapter.set_powered(true).await?;

    println!("Bluetooth adapter: {}", adapter.name());

    println!("Bluetooth address: {}", adapter.address().await?);

    let advertisement = Advertisement {
        service_uuids: vec![SERVICE_UUID].into_iter().collect(),

        discoverable: Some(true),

        local_name: Some(DEVICE_NAME.to_string()),

        ..Default::default()
    };

    let adv_handle = adapter.advertise(advertisement).await?;

    let value = Arc::new(Mutex::new(encode_state(LightState::Off)));

    let value_read = value.clone();
    let value_write = value.clone();

    let light_write = light.map_err(|e| {
        let io_err = std::io::Error::other(e.to_string());
        bluer::Error::from(io_err)
    })?;

    let application = Application {
        services: vec![Service {
            uuid: SERVICE_UUID,

            primary: true,

            characteristics: vec![Characteristic {
                uuid: COMMAND_CHARACTERISTIC_UUID,

                read: Some(CharacteristicRead {
                    read: true,

                    fun: Box::new(move |_request| {
                        let value = value_read.clone();

                        async move { Ok(value.lock().await.clone()) }.boxed()
                    }),

                    ..Default::default()
                }),

                write: Some(CharacteristicWrite {
                    write: true,

                    write_without_response: true,

                    method: CharacteristicWriteMethod::Fun(Box::new(move |data, _request| {
                        let value = value_write.clone();

                        let light = light_write.clone();

                        async move {
                            let command = decode_command(&data)
                                .map_err(|_| bluer::gatt::local::ReqError::Failed)?;

                            let mut light = light.lock().await;

                            let state = match command {
                                Command::On => {
                                    light
                                        .set_on()
                                        .map_err(|_| bluer::gatt::local::ReqError::Failed)?;

                                    LightState::On
                                }

                                Command::Off => {
                                    light
                                        .set_off()
                                        .map_err(|_| bluer::gatt::local::ReqError::Failed)?;

                                    LightState::Off
                                }

                                Command::Status => {
                                    if light.is_on() {
                                        LightState::On
                                    } else {
                                        LightState::Off
                                    }
                                }
                            };

                            *value.lock().await = encode_state(state);

                            println!("command={command:?} state={state:?}");

                            Ok(())
                        }
                        .boxed()
                    })),

                    ..Default::default()
                }),

                ..Default::default()
            }],

            ..Default::default()
        }],

        ..Default::default()
    };

    let app_handle = adapter.serve_gatt_application(application).await?;

    println!("Receiver started.");
    println!("Device name: {DEVICE_NAME}");
    println!("GPIO BCM pin: {GPIO_PIN}");
    println!("Service UUID: {SERVICE_UUID}");
    println!("Characteristic UUID: {COMMAND_CHARACTERISTIC_UUID}");
    println!("Waiting for BLE commands...");

    tokio::signal::ctrl_c().await?;

    println!("Shutting down.");

    drop(app_handle);
    drop(adv_handle);

    Ok(())
}
