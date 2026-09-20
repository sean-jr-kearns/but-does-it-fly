use bluer::{
    Adapter, Session,
    adv::{Advertisement, AdvertisementHandle},
    gatt::local::{
        Application, ApplicationHandle, Characteristic, CharacteristicNotify,
        CharacteristicNotifyMethod, CharacteristicRead, CharacteristicWrite,
        CharacteristicWriteMethod, Service,
    },
};
use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID, DEVICE_NAME, SERVICE_UUID, TELEMETRY_CHARACTERISTIC_UUID,
};
use communication_protocol::{
    Response,
    prelude::{Command, ProtocolError, deserialize_command, serialize_response},
};
use futures::FutureExt;
use safety_lights::SafetyLights;
use std::sync::Arc;
use telemetry::Telemetry;
use tokio::sync::{Mutex, broadcast};

pub struct Controller<'a> {
    _session: Session,
    pub adapter: Adapter,
    pub advertisements: Vec<AdvertisementHandle>,
    pub safety_lights: &'a Arc<Mutex<SafetyLights>>,
    pub telemetry: &'a Arc<Mutex<Telemetry>>,
}

impl<'a> Controller<'a> {
    pub async fn new(
        safety_lights: &'a Arc<Mutex<SafetyLights>>,
        telemetry: &'a Arc<Mutex<Telemetry>>,
    ) -> Result<Self, ProtocolError> {
        let session = bluer::Session::new()
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message))?;
        let adapter = session
            .default_adapter()
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message))?;

        adapter
            .set_powered(true)
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message))?;

        println!("Bluetooth adapter: {}", adapter.name());
        println!(
            "Bluetooth address: {}",
            adapter
                .address()
                .await
                .map_err(|e| ProtocolError::Bluetooth(e.message))?
        );

        Ok(Self {
            _session: session,
            adapter,
            advertisements: Vec::new(),
            safety_lights,
            telemetry,
        })
    }

    pub async fn new_advertisement(&mut self) -> Result<(), ProtocolError> {
        let advertisement = Advertisement {
            service_uuids: vec![SERVICE_UUID].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some(DEVICE_NAME.to_string()),
            ..Default::default()
        };
        self.advertisements.push(
            self.adapter
                .advertise(advertisement)
                .await
                .map_err(|e| ProtocolError::Bluetooth(e.message))?,
        );

        Ok(())
    }

    pub async fn new_application(
        &self,
        value_read: Arc<Mutex<Vec<u8>>>,
        value_write: Arc<Mutex<Vec<u8>>>,
    ) -> Result<ApplicationHandle, ProtocolError> {
        let application = Application {
            services: vec![Service {
                uuid: SERVICE_UUID,
                primary: true,
                characteristics: vec![
                    self.command_response_characteristic(value_read, value_write),
                    self.telemetry_characteristic(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        };

        let handle = self
            .adapter
            .serve_gatt_application(application)
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message))?;
        Ok(handle)
    }

    pub fn command_response_characteristic(
        &self,
        value_read: Arc<Mutex<Vec<u8>>>,
        value_write: Arc<Mutex<Vec<u8>>>,
    ) -> Characteristic {
        let safety = self.safety_lights.clone();
        Characteristic {
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
                    let safety = safety.clone();
                    async move {
                        let command = deserialize_command(&data)
                            .map_err(|_| bluer::gatt::local::ReqError::Failed)?;
                        let mut safety_guard = safety.lock().await;
                        let response = match command {
                            Command::Arm => {
                                let () = safety_guard.run();
                                Response::Ok
                            }
                            Command::Disarm => {
                                let _ = safety_guard.stop();
                                Response::Ok
                            }
                            _ => {
                                println!("Command not implemented");
                                Response::Ok
                            }
                        };
                        *value.lock().await = serialize_response(response);
                        println!("command={command:?} state={response:?}");
                        drop(safety_guard);
                        Ok(())
                    }
                    .boxed()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn telemetry_characteristic(&self) -> Characteristic {
        let telemetry = self.telemetry.clone();
        Characteristic {
            uuid: TELEMETRY_CHARACTERISTIC_UUID,
            notify: Some(CharacteristicNotify {
                notify: true,
                method: CharacteristicNotifyMethod::Fun(Box::new(move |mut notifier| {
                    let tx = telemetry.clone();
                    async move {
                        let mut rx = tx.lock().await.subscribe();
                        loop {
                            match rx.recv().await {
                                Ok(data) => {
                                    if notifier.notify(data).await.is_err() {
                                        break;
                                    }
                                }
                                Err(broadcast::error::RecvError::Lagged(n)) => {
                                    eprintln!("BLE telemetry receiver lagged by {n} messages");
                                }
                                Err(broadcast::error::RecvError::Closed) => {
                                    break;
                                }
                            }
                        }
                    }
                    .boxed()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
