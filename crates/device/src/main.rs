mod controller;

use bluer::gatt::local::CharacteristicNotifier;
use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID, DEVICE_NAME, SERVICE_UUID, TELEMETRY_CHARACTERISTIC_UUID,
};
use communication_protocol::{Response, prelude::*};
use gpio_lights::{Indicator, Pin};
use std::sync::Arc;
use telemetry::publish;
use tokio::sync::Mutex;

use crate::controller::Controller;

// Define used hardware GPIO pins
const GREEN_LIGHT_PIN: u8 = 17;
const RED_LIGHT_PIN: u8 = 27;
const TELEMETRY_PIN: u8 = 23;

#[allow(clippy::expect_used, reason = "hardware pin connection mapping")]
#[allow(clippy::match_single_binding)] // temporary until new commands fill command loop match statement
#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    // Initialize logger
    env_logger::init();
    // Initialize thread-safe fields used to inject and extract values via bluetooth
    let response = Arc::new(Mutex::new(Response::Off));
    let command = Arc::new(Mutex::new(Command::Status));
    let telemetry_notifier: Arc<Mutex<Option<CharacteristicNotifier>>> = Arc::new(Mutex::new(None));

    // Start publishing telemetry. Is tied to a bluetooth `CharacteristicNotifier`.
    // No one can listen until the BLE application is started with the telemetry
    // `CharacteristicNotify` characteristic (which notifies based on the
    // `CharacteristicNotifier`)
    // GPIO PIN `TELEMETRY_PIN` has a blue LED that flashes while telemetry is publishing
    let tlm = telemetry_notifier.clone();
    tokio::spawn(async move {
        let mut blue_telemetry_pin: Indicator =
            Indicator::new(TELEMETRY_PIN).expect("GPIO hardware conection failed. Terminating");
        loop {
            let _ = blue_telemetry_pin.set_on();
            let _ = publish(tlm.clone()).await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = blue_telemetry_pin.set_off();
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    // Initialize bluetooth controller
    let mut controller = Controller::new().await.map_err(|e| {
        let io_err = std::io::Error::other(e.to_string());
        bluer::Error::from(io_err)
    })?;
    // Advertise Device. When station is started ID pairing auth challenge should
    // occur on both the station and the device
    let () = controller.new_advertisement().await.map_err(|e| {
        let io_err = std::io::Error::other(e.to_string());
        bluer::Error::from(io_err)
    })?;
    // Start command and telemetry bluetooth application
    // Now the station can send commands and receive telemetry
    // (assuming the passkey challenge is accepted on both station and device).
    let telemetry = telemetry_notifier.clone();
    let app_handle = controller
        .new_application(response, command.clone(), telemetry)
        .await
        .map_err(|e| {
            let io_err = std::io::Error::other(e.to_string());
            bluer::Error::from(io_err)
        })?;
    println!("Controller started");
    println!("\tDevice name: {DEVICE_NAME}");
    println!("\tService UUID: {SERVICE_UUID}");
    println!("\tCharacteristic UUIDs:");
    println!("\t\tCommand: {COMMAND_CHARACTERISTIC_UUID}");
    println!("\t\tTelemetry: {TELEMETRY_CHARACTERISTIC_UUID}");
    println!("Waiting for commands...");

    // Command loop
    tokio::spawn(async move {
        // Initialize hardware pin interfaces
        let mut green_safety_pin: Indicator =
            Indicator::new(GREEN_LIGHT_PIN).expect("GPIO hardware conection failed. Terminating");
        let mut red_safety_pin =
            Indicator::new(RED_LIGHT_PIN).expect("GPIO hardware conection failed. Terminating");

        // Hold onto previous command outside of the loop so the device knows when it changes
        let mut last_processed_command = Command::Status;
        loop {
            let current_command = *command.lock().await;
            if current_command == Command::Arm {
                let _ = green_safety_pin.set_on();
                let _ = red_safety_pin.set_on();
            }
            // FUTURE: when movement commands are introduced they will go here
            if current_command != last_processed_command {
                println!("Main entrypoint detected new command: {current_command}");
                match current_command {
                    _ => {
                        println!("Future: some commands are only executed once");
                    }
                }
                last_processed_command = current_command;
            }

            // Safety lights can sit on the same sleep schedule as the command loop
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let _ = green_safety_pin.set_off();
            let _ = red_safety_pin.set_off();
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
    });

    futures::future::pending::<()>().await;

    tokio::signal::ctrl_c().await?;
    println!("Shutting down");
    drop(app_handle);
    Ok(())
}
