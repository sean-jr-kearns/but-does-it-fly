use crate::gpio::{Indicator, Pin};
use communication_protocol::{ProtocolError, telemetry::TelemetryPacket};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

pub struct Telemetry {
    pub tx_pin: Arc<Mutex<Indicator>>,
    pub is_on: bool,
    cancellation_token: CancellationToken,
    handle: Option<JoinHandle<Result<(), rppal::gpio::Error>>>,
}

impl Telemetry {
    pub fn new(tx_pin: u8) -> Result<Self, ProtocolError> {
        let token = CancellationToken::new();
        let tx_pin = Indicator::new(tx_pin).map_or_else(
            |_| {
                Err(ProtocolError::InvalidPeripheral(
                    "tx indicator setup unsuccessful".to_string(),
                ))
            },
            |light| Ok(Arc::new(Mutex::new(light))),
        )?;

        Ok(Self {
            tx_pin,
            is_on: false,
            cancellation_token: token,
            handle: None,
        })
    }

    pub async fn run(&mut self) -> Result<(), rppal::gpio::Error> {
        // Clone used vars for spawned thread lifetime purposes
        let tx_pin: Arc<Mutex<Indicator>> = self.tx_pin.clone();
        let token = self.cancellation_token.clone();

        let handle = tokio::spawn(async move {
            println!("Telemetry sender started");

            loop {
                tokio::select! {
                    // Instantly exit if the stop command is issued
                    _ = token.cancelled() => {
                        println!("Stop command received. Safely shutting down telemetry sender...");
                        // Ensure pins are left in a safe/default state
                        let mut tx_pin = tx_pin.lock().await;
                        {
                            let _ = tx_pin.set_off();
                        }
                        break;
                    }

                    // Toggling Sequence
                    _ = async {
                        // Turn on telemetry tx indicator
                        let mut tx_pin = tx_pin.lock().await;
                        let _ = tx_pin.set_on();
                        // Send dummy telemetry
                        let _packet = TelemetryPacket::default();
                        // Turn off telemetry tx indicator
                        let _ = tx_pin.set_off();
                        sleep(Duration::from_millis(1000)).await;
                    } => {}
                }
            }

            Ok::<(), rppal::gpio::Error>(())
        });

        self.handle = Some(handle);
        self.is_on = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<bool, rppal::gpio::Error> {
        // Issue the stop command
        println!("Issuing telemetry stop command...");
        self.cancellation_token.cancel();

        // Wait for the loop to finish its cleanup
        let _ = self.handle;
        self.is_on = false;
        println!("Telemetry sender stopped");
        Ok(!self.is_on)
    }
}
