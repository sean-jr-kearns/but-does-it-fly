use bluer::{
    Adapter, Session,
    adv::{Advertisement, AdvertisementHandle},
    gatt::local::{
        Application, ApplicationHandle, Characteristic, CharacteristicNotifier,
        CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicRead, CharacteristicWrite,
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
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Controller {
    _session: Session,
    pub adapter: Adapter,
    pub advertisements: Vec<AdvertisementHandle>,
}

impl Controller {
    /// Creates a new [`Controller`] which manages a Device's bluetooth session,
    /// adapter, and defined application interfaces.
    ///
    /// # Errors
    ///
    /// Returns a  [`ProtocolError`] if unable to initialize a [`bluer::Session`]
    /// or [`bluer::adapter::Adapter`] within the session.
    pub async fn new() -> Result<Self, ProtocolError> {
        // Establish D-Bus connection to the root of the bluez system service
        // Discovers physically available hardware
        let session = bluer::Session::new()
            .await
            .map_err(|e| ProtocolError::Bluetooth(e.message))?;
        // Given a session specify a bluetooth controller to communicate with
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
        last_response: Arc<Mutex<Response>>,
        next_command: Arc<Mutex<Command>>,
        notifier: Arc<Mutex<Option<CharacteristicNotifier>>>,
    ) -> Result<ApplicationHandle, ProtocolError> {
        let application = Application {
            services: vec![Service {
                uuid: SERVICE_UUID,
                primary: true,
                characteristics: vec![
                    Self::command_response_characteristic(last_response, next_command),
                    Self::telemetry_characteristic(notifier),
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

    /// Defines the command and response characteristic. `Station` publishes a command to `Device`
    /// over D-BUS interface with id [`COMMAND_CHARACTERISTIC_UUID`].
    pub fn command_response_characteristic(
        last_response: Arc<Mutex<Response>>,
        next_command: Arc<Mutex<Command>>,
    ) -> Characteristic {
        Characteristic {
            uuid: COMMAND_CHARACTERISTIC_UUID,
            // What station is trying to read from device
            // e.g. can read current command or command response
            read: Some(CharacteristicRead {
                read: true,
                fun: Box::new({
                    let value = last_response.clone();
                    move |_request| {
                        let value = value.clone();
                        async move { Ok(serialize_response(*value.lock().await)) }.boxed()
                    }
                }),
                ..Default::default()
            }),
            // What station is trying to write to the device
            // e.g. the command
            write: Some(CharacteristicWrite {
                write: true,
                write_without_response: false,
                method: CharacteristicWriteMethod::Fun(Box::new(move |command, _request| {
                    let write_next_command = next_command.clone();
                    let write_last_response = last_response.clone();
                    async move {
                        // Should be fine to shadow this right?
                        let command = deserialize_command(&command)
                            .map_err(|_| bluer::gatt::local::ReqError::Failed)?;
                        let response = match command {
                            Command::Arm | Command::Disarm => Response::Ok,
                            _ => {
                                println!("Command not implemented");
                                Response::Ok
                            }
                        };
                        *write_last_response.lock().await = response;
                        *write_next_command.lock().await = command;
                        println!("command={command:?} state={response:?}");
                        Ok(())
                    }
                    .boxed()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Defines the telemetry notification characteristic. `Device` publishes a notification to `Station`
    /// over D-BUS interface with id [`TELEMETRY_CHARACTERISTIC_UUID`].
    #[allow(clippy::significant_drop_tightening)]
    pub fn telemetry_characteristic(
        notifier: Arc<Mutex<Option<CharacteristicNotifier>>>,
    ) -> Characteristic {
        Characteristic {
            uuid: TELEMETRY_CHARACTERISTIC_UUID,
            notify: Some(CharacteristicNotify {
                notify: true,
                method: CharacteristicNotifyMethod::Fun(Box::new(move |characteristic_notifier| {
                    let telemetry = notifier.clone();
                    async move {
                        let mut tx = telemetry.lock().await;
                        *tx = Some(characteristic_notifier);
                    }
                    .boxed()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
