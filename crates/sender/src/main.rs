mod ble;

use crate::ble::{
    find_adapter, find_advertised_characteristic_by_id, find_device, find_device_peripheral,
    read_from_peripheral, write_to_peripheral,
};
use btleplug::api::Peripheral as _;
use communication_protocol::{
    ProtocolError,
    prelude::{decode_response, encode_command, parse_command},
};
use std::io::{self, Write};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = find_adapter().await?;
    let target = find_device(&adapter.clone()).await?;
    // Sometimes on windows WinRT subsystem needs
    // a moment to completely unload the scan thread
    //tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let peripheral = find_device_peripheral(&adapter.clone(), &target.clone()).await?;
    // register device via direct address
    match peripheral.connect().await {
        Ok(()) => {
            info!("SUCCESS: connected to device");
            peripheral.discover_services().await?;
            info!("GATT services synchronized");
            let characteristic = find_advertised_characteristic_by_id(peripheral.clone())?;

            // YOU ARE HERE: Left off testing command loop
            // arm disarm works the first time and then never again
            // telemetry never actually turns on the light
            // there are no packets being sent (e.g. telem should be over telem characteristic)
            // that is, another characteristic need to be made and prove in logs on sender that telemetry is showing up
            // should probably rename to device and station

            loop {
                // persist connection so that auth challenge from windows persists for the lifetime of the service
                print!("Enter command: ");

                // Flush stdout to guarantee the prompt prints immediately
                io::stdout().flush().unwrap();
                let input = input_buffer()?;
                let command = parse_command(&input)?;
                let packet = encode_command(command);
                let () = write_to_peripheral(peripheral.clone(), &characteristic, &packet).await?;
                info!("Sent command: {command:?}");

                // move command logic into read frm/write to
                // expand lightstate to be enum of states (rx/tx state, safety state?, commands?)
                // expand command logic into enum struct impls per command arm disarm to start (which replaces light-show)
                let response: Vec<u8> = peripheral.read(&characteristic).await?;

                let response = decode_response(&response)?;

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
