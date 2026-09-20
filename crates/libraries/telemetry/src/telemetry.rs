use crate::{
    error::TelemetryError,
    telemetry_packet::{TelemetryPacket, serialize},
};
use futures::Stream;
use gpio_lights::{Indicator, Pin};
use std::{convert::Infallible, sync::Arc};
use tokio::{sync::Mutex, sync::broadcast};

pub struct Telemetry {
    pub tx_pin: Arc<Mutex<Indicator>>,
    pub sender: broadcast::Sender<Vec<u8>>,
}

impl Telemetry {
    /// Creates new telemetry publisher and tx pin indicator
    ///
    /// # Errors
    /// Returns [`TelemetryError`] if unable to publish telemetry or access telemetry peripherals
    pub fn new(tx_pin: u8, sender: broadcast::Sender<Vec<u8>>) -> Result<Self, TelemetryError> {
        let tx_pin = Indicator::new(tx_pin).map_or_else(
            |_| {
                Err(TelemetryError::Unknown(
                    "tx indicator setup unsuccessful".to_string(),
                ))
            },
            |light| Ok(Arc::new(Mutex::new(light))),
        )?;

        Ok(Self { tx_pin, sender })
    }

    /// Publishes mock telemetry
    ///
    /// # Errors
    /// Returns [`TelemetryError`] if unable to publish telemetry
    pub async fn publish(&mut self) -> Result<(), TelemetryError> {
        self.telemetry_gpio(true).await;
        let telemetry = Self::temp_telemetry_generator();
        // Publish to BLE subscribers
        let bytes = serialize(&telemetry)?;
        let _ = self.sender.send(bytes);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        self.telemetry_gpio(false).await;
        Ok(())
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.sender.subscribe()
    }

    /// Initializes notification stream for BLE
    pub fn notification_stream(
        &mut self,
        mut channel: broadcast::Receiver<Vec<u8>>,
    ) -> impl Stream<Item = Result<Vec<u8>, Infallible>> {
        async_stream::stream! {
            loop {
                match channel.recv().await {
                    Ok(data) => {
                        yield Ok(data)
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {},
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    async fn telemetry_gpio(&self, on: bool) {
        let tx_pin: Arc<Mutex<Indicator>> = Arc::clone(&self.tx_pin);
        let mut tx_pin_guard = tx_pin.lock().await;
        if on {
            let _ = tx_pin_guard.set_on();
        } else {
            let _ = tx_pin_guard.set_off();
        }
    }

    fn temp_telemetry_generator() -> TelemetryPacket {
        TelemetryPacket::default()
    }
}
