use communication_protocol::prelude::{Command, decode_state, encode_command};
use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID, /*DEVICE_NAME,*/ ProtocolError, SERVICE_UUID,
};

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::{env, error::Error};

fn parse_command() -> Result<Command, Box<dyn Error>> {
    let argument = env::args().nth(1).ok_or("usage: sender <on|off|status>")?;

    match argument.to_lowercase().as_str() {
        "on" => Ok(Command::On),
        "off" => Ok(Command::Off),
        "status" => Ok(Command::Status),

        _ => Err("command must be on, off, or status".into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command from service command line arguments
    let command = parse_command()?;
    // Setup device adapter and connection
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    if adapters.is_empty() {
        eprintln!("Error: No Bluetooth adapters found on this machine.");
        return Ok(());
    }
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or(None)
        .map_err(|_: Option<String>| {
            ProtocolError::Bluetooth("failed to get bluetooth adapter".to_string())
        })?;

    // Start scanning just to discover the device's address
    adapter.start_scan(ScanFilter::default()).await?;
    let mut events = adapter.events().await?;

    let mut found_target_id: Option<btleplug::platform::PeripheralId> = None;

    println!("Scanning for device service UUID...");

    while let Some(event) = events.next().await {
        if let CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) = event
            && let Some(peripheral) = adapter.peripherals().await?.iter().find(|p| p.id() == id)
            && let Ok(Some(properties)) = peripheral.properties().await
        {
            let matches = properties
                .services
                .iter()
                .any(|u| u.to_string().to_lowercase() == SERVICE_UUID.to_string().to_lowercase());

            if matches {
                println!("Device found! Capturing literal MAC Address...");
                found_target_id = Some(id);
                break; // Exit immediately
            }
        }
    }

    std::mem::drop(events);
    let _ = adapter.stop_scan().await;

    // Sometimes on windows WinRT subsystem needs
    // a moment to completely unload the scan thread
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;

    // register device via direct address
    if let Some(addr) = found_target_id {
        println!("Injecting target address ({addr}) directly into the clean adapter...");

        // Get clean handle and connect
        let fresh_peripherals = adapter.peripherals().await?;
        if let Some(peripheral) = fresh_peripherals.iter().find(|p| p.id() == addr) {
            println!("Opening isolated GattSession to device...");

            match peripheral.connect().await {
                Ok(()) => {
                    println!("SUCCESS: Connected device cleanly!");
                    peripheral.discover_services().await?;
                    println!("GATT Services synchronized.");

                    let characteristic = peripheral
                        .characteristics()
                        .into_iter()
                        .find(|characteristic| characteristic.uuid == COMMAND_CHARACTERISTIC_UUID)
                        .ok_or("command characteristic not found")?;

                    let packet = encode_command(command);

                    peripheral
                        .write(&characteristic, &packet, WriteType::WithResponse)
                        .await?;

                    println!("Sent command: {command:?}");

                    let state = peripheral.read(&characteristic).await?;

                    let state = decode_state(&state)?;

                    println!("Light state: {state:?}");

                    peripheral.disconnect().await?;
                }
                Err(e) => {
                    println!("Machine refused the direct session hook: {e:?}");
                }
            }
        }
    } else {
        println!("Could not discover the device's advertisement packets.");
    }

    Ok(())
}
