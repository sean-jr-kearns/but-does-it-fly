mod btle;

use crate::btle::{
    find_adapter, find_advertised_characteristic_by_id, find_device, find_device_peripheral,
    write_to_peripheral,
};
use btleplug::api::Peripheral as _;
use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID, ProtocolError, TELEMETRY_CHARACTERISTIC_UUID,
    prelude::{deserialize_response, parse_command, serialize_command},
};
use futures::StreamExt;
use std::io::{self, Write};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let adapter = find_adapter().await?;
    let target = find_device(&adapter.clone()).await?;
    let peripheral = find_device_peripheral(&adapter.clone(), target.clone().as_ref()).await?;
    // Set exit
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);
    // register device via direct address
    match peripheral.connect().await {
        Ok(()) => {
            info!("SUCCESS: connected to device");
            peripheral.discover_services().await?;
            info!("GATT services synchronized");

            //let telemetry_peripheral = peripheral.clone();
            //tokio::spawn(async move {
            //    let telemetry_characteristic = find_advertised_characteristic_by_id(
            //        &telemetry_peripheral,
            //        TELEMETRY_CHARACTERISTIC_UUID,
            //    )?;
            //    if telemetry_characteristic
            //        .properties
            //        .contains(btleplug::api::CharPropFlags::NOTIFY)
            //    {
            //        // Subscribe to the characteristic
            //        telemetry_peripheral
            //            .subscribe(&telemetry_characteristic)
            //            .await
            //            .map_err(|_| {
            //                ProtocolError::InvalidPeripheral(
            //                    "unable to subscribe to peripheral".to_string(),
            //                )
            //            })?;
            //        println!(
            //            "Subscribed to characteristic: {}",
            //            telemetry_characteristic.uuid
            //        );
            //        // Get notification stream
            //        let mut notification_stream =
            //            telemetry_peripheral.notifications().await.map_err(|_| {
            //                ProtocolError::InvalidPeripheral(
            //                    "unable to get peripheral notifications".to_string(),
            //                )
            //            })?;
            //        // Process incoming stream items asynchronously
            //        while let Some(notification) = notification_stream.next().await {
            //            println!(
            //                "Received notification from {} -> Raw bytes: {:?}",
            //                notification.uuid, notification.value
            //            );
            //        }
            //    } else {
            //        println!("The characteristic does not support notifications.");
            //    }
            //    loop {
            //        let telemetry = telemetry_peripheral
            //            .read(&telemetry_characteristic)
            //            .await
            //            .map_err(|_| {
            //                ProtocolError::InvalidPeripheral(
            //                    "unable to read peripheral stream".to_string(),
            //                )
            //            })?;
            //        info!("telemetry: {telemetry:?}");
            //    }
            //    Ok::<(), ProtocolError>(())
            //});

            //need pin/peripheral safe shutdown
            let characteristic =
                find_advertised_characteristic_by_id(&peripheral, COMMAND_CHARACTERISTIC_UUID)?;

            loop {
                // persist connection so that auth challenge from windows persists for the lifetime of the service
                println!("Enter command: ");
                // Flush stdout to guarantee the prompt prints immediately
                io::stdout().flush()?;
                let input = input_buffer()?;
                let command = parse_command(&input)?;
                let packet = serialize_command(command);
                let () = write_to_peripheral(&peripheral, &characteristic, &packet).await?;
                info!("Sent command: {command:?}");
                let response: Vec<u8> = peripheral.read(&characteristic).await?;
                let response = deserialize_response(&response)?;
                info!("Response: {response:?}");
            }
            peripheral.disconnect().await?;
        }
        Err(e) => {
            info!("Machine refused the direct session hook: {e:?}");
        }
    }

    Ok(())
}

fn input_buffer() -> Result<String, ProtocolError> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| ProtocolError::Unknown(e.to_string()))?;
    Ok(input.trim().to_string())
}
