use crate::btle::find_advertised_characteristic_by_id;
use btleplug::{api::Peripheral as _, platform::Peripheral};
use communication_protocol::{ProtocolError, TELEMETRY_CHARACTERISTIC_UUID};
use futures::StreamExt;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::{pin, signal, time};
use tracing::{error, info};

pub fn stream_telemetry_from_peripheral(
    peripheral: &Peripheral,
) -> JoinHandle<std::result::Result<(), ProtocolError>> {
    let telemetry_peripheral = peripheral.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        let ctrl_c = signal::ctrl_c();
        pin!(ctrl_c);

        let telemetry_characteristic = find_advertised_characteristic_by_id(
            &telemetry_peripheral,
            TELEMETRY_CHARACTERISTIC_UUID,
        )?;
        if telemetry_characteristic
            .properties
            .contains(btleplug::api::CharPropFlags::NOTIFY)
        {
            // Subscribe to the characteristic
            telemetry_peripheral
                .subscribe(&telemetry_characteristic)
                .await
                .map_err(|_| {
                    ProtocolError::InvalidPeripheral(
                        "unable to subscribe to peripheral".to_string(),
                    )
                })?;
            info!(
                "Subscribed to characteristic: {}",
                telemetry_characteristic.uuid
            );
            // Get notification stream
            let mut notification_stream =
                telemetry_peripheral.notifications().await.map_err(|_| {
                    ProtocolError::InvalidPeripheral(
                        "unable to get peripheral notifications".to_string(),
                    )
                })?;
            // Process incoming stream items asynchronously
            while let Some(notification) = notification_stream.next().await {
                info!(
                    "Received notification from {} -> Raw bytes: {:?}",
                    notification.uuid, notification.value
                );
            }
        } else {
            info!("The characteristic does not support notifications.");
        }

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match telemetry_peripheral
                        .read(&telemetry_characteristic)
                        .await
                        .map_err(|_| {
                            ProtocolError::InvalidPeripheral("unable to read peripheral stream".to_string())
                        }) {
                        Ok(telemetry) => {
                            info!("telemetry: {telemetry:?}");
                        }
                        Err(e) => {
                            error!("Error processing telemetry: {e:?}...");
                            break;
                        }
                    }
                }
                _ = &mut ctrl_c => {
                    info!("\nCtrl+C received! Shutting down gracefully...");
                    break; // Break the loop to stop the program
                }
            }
        }

        Ok::<(), ProtocolError>(())
    })
}
