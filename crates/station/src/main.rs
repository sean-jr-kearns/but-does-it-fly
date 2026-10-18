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
use tokio::{signal, time};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

#[allow(clippy::expect_used, reason = "simplify exit signal")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start tracing
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false) // Disable terminal colors for flat text files
        .init();
    info!("Starting application. Type CTRL+C to exit");
    let cancel_token = CancellationToken::new();
    let mut interval = time::interval(Duration::from_secs(1));
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
            let telemetry_token = cancel_token.clone();
            std::mem::drop(tokio::spawn(async move {
                let _ =
                    stream_telemetry_from_peripheral(&telemetry_peripheral, telemetry_token).await;
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
                    () = cancel_token.cancelled() => {
                        info!("Station received cancel signal; shutting down...");
                        break;
                    }
                }
            }
            peripheral.disconnect().await?;
        }
        Err(e) => {
            error!("Machine refused the direct session hook: {e:?}");
        }
    }
    signal::ctrl_c().await.expect("Ctrl+C received");
    info!("All station threads have stopped. Program terminating...");
    Ok(())
}
