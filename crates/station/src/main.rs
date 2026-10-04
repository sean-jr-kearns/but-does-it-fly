mod btle;
mod command;
mod telemetry;

use btle::{
    find_adapter, find_advertised_characteristic_by_id, find_device, find_device_peripheral,
};
use btleplug::api::Peripheral as _;
use command::process_command;
use communication_protocol::COMMAND_CHARACTERISTIC_UUID;
use std::time::Duration;
use telemetry::stream_telemetry_from_peripheral;
use tokio::{pin, signal, time};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start tracing
    tracing_subscriber::fmt::init();
    info!("Starting application. Type CTRL+C to exit");
    let mut interval = time::interval(Duration::from_secs(1));
    let ctrl_c = signal::ctrl_c();
    pin!(ctrl_c);
    info!("Finding adapter and peripherals");
    let adapter = find_adapter().await?;
    let target = find_device(&adapter.clone()).await?;
    let peripheral = find_device_peripheral(&adapter.clone(), target.clone().as_ref()).await?;
    info!("Connecting to pre-defined device peripherals");
    match peripheral.connect().await {
        Ok(()) => {
            info!("SUCCESS: connected to device");
            peripheral.discover_services().await?;
            info!("GATT services synchronized");

            let telemetry_peripheral = peripheral.clone();
            std::mem::drop(tokio::spawn(async move {
                let _ = stream_telemetry_from_peripheral(&telemetry_peripheral).await;
            }));

            //need pin/peripheral safe shutdown
            let characteristic =
                find_advertised_characteristic_by_id(&peripheral, COMMAND_CHARACTERISTIC_UUID)?;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match process_command(&peripheral, &characteristic).await {
                            Ok(()) => { /* Do nothing */ }
                            Err(e) => {
                                warn!("Error processing command: {e:?}. Continuing loop...");
                            }
                        }
                    }
                    _ = &mut ctrl_c => {
                        info!("\nCtrl+C received! Shutting down gracefully...");
                        break; // Break the loop to stop the program
                    }
                }
            }
            peripheral.disconnect().await?;
        }
        Err(e) => {
            error!("Machine refused the direct session hook: {e:?}");
        }
    }

    Ok(())
}
