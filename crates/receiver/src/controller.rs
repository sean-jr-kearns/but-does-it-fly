use crate::safety_lights::SafetyLights;
use crate::telemetry::Telemetry;
use bluer::{
    Adapter, Session,
    adv::{Advertisement, AdvertisementHandle},
    gatt::local::{
        Application, ApplicationHandle, Characteristic, CharacteristicRead, CharacteristicWrite,
        CharacteristicWriteMethod, Service,
    },
};
use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID,
    DEVICE_NAME,
    SERVICE_UUID, //TELEMETRY_CHARACTERISTIC_UUID,
};
use communication_protocol::{
    Response,
    prelude::{Command, ProtocolError, decode_command, encode_response},
};
use futures::FutureExt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Controller {
    _session: Session,
    pub adapter: Adapter,
    pub advertisements: Vec<AdvertisementHandle>,
    pub safety_lights: Arc<Mutex<SafetyLights>>,
    pub telemetry: Arc<Mutex<Telemetry>>,
}

impl Controller {
    pub async fn new() -> Result<Self, ProtocolError> {
        let session = bluer::Session::new()
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message.to_string()))?;
        let adapter = session
            .default_adapter()
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message.to_string()))?;

        adapter
            .set_powered(true)
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message.to_string()))?;

        println!("Bluetooth adapter: {}", adapter.name());
        println!(
            "Bluetooth address: {}",
            adapter
                .address()
                .await
                .map_err(|e| ProtocolError::Bluetooth(e.message.to_string()))?
        );

        let safety_lights = Arc::new(Mutex::new(SafetyLights::new(17, 27)?));
        let telemetry = Arc::new(Mutex::new(Telemetry::new(23)?));

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
                .map_err(|e| ProtocolError::Bluetooth(e.message.to_string()))?,
        );

        Ok(())
    }

    pub async fn new_application(
        &mut self,
        value_read: Arc<Mutex<Vec<u8>>>,
        value_write: Arc<Mutex<Vec<u8>>>,
    ) -> Result<ApplicationHandle, ProtocolError> {
        let application = Application {
            services: vec![Service {
                uuid: SERVICE_UUID,
                primary: true,
                characteristics: vec![
                    self.command_response_characteristic(value_read, value_write)
                        .await
                        .map_err(|e| ProtocolError::Bluetooth(e.to_string()))?,
                ],
                ..Default::default()
            }],
            ..Default::default()
        };

        let handle = self
            .adapter
            .serve_gatt_application(application)
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message.to_string()))?;
        Ok(handle)
    }

    pub async fn command_response_characteristic(
        &mut self,
        value_read: Arc<Mutex<Vec<u8>>>,
        value_write: Arc<Mutex<Vec<u8>>>,
    ) -> Result<Characteristic, ProtocolError> {
        let telemetry = self.telemetry.clone();
        let safety = self.safety_lights.clone();
        Ok(Characteristic {
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
                    let tx = telemetry.clone();
                    let safety = safety.clone();
                    async move {
                        let command = decode_command(&data)
                            .map_err(|_| bluer::gatt::local::ReqError::Failed)?;
                        let mut tx_guard = tx.lock().await;
                        let mut safety_guard = safety.lock().await;
                        let response = match command {
                            Command::On => {
                                tx_guard
                                    .run()
                                    .await
                                    .map_err(|_| bluer::gatt::local::ReqError::Failed)?;
                                Response::On
                            }
                            Command::Off => {
                                tx_guard
                                    .stop()
                                    .await
                                    .map_err(|_| bluer::gatt::local::ReqError::Failed)?;
                                Response::Off
                            }
                            Command::Status => {
                                if tx_guard.is_on {
                                    Response::On
                                } else {
                                    Response::Off
                                }
                            }
                            Command::Arm => {
                                let _ = safety_guard.run().await;
                                Response::Ok
                            }
                            Command::Disarm => {
                                let _ = safety_guard.stop().await;
                                Response::Ok
                            }
                            _ => {
                                println!("Command not implemented");
                                Response::Ok
                            }
                        };
                        *value.lock().await = encode_response(response);
                        println!("command={command:?} state={response:?}");
                        Ok(())
                    }
                    .boxed()
                })),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
