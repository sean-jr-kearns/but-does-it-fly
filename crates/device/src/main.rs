mod controller;

use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID, DEVICE_NAME, SERVICE_UUID, TELEMETRY_CHARACTERISTIC_UUID,
};
use communication_protocol::{Response, prelude::serialize_response};
use safety_lights::SafetyLights;
use std::sync::Arc;
use telemetry::Telemetry;
use tokio::sync::{Mutex, broadcast};

use crate::controller::Controller;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    env_logger::init();
    let (sender, _) = broadcast::channel::<Vec<u8>>(16);
    let safety_lights =
        Arc::new(Mutex::new(SafetyLights::new(17, 27).map_err(|e| {
            bluer::Error::from(std::io::Error::other(e.to_string()))
        })?));
    let telemetry =
        Arc::new(Mutex::new(Telemetry::new(23, sender).map_err(|e| {
            bluer::Error::from(std::io::Error::other(e.to_string()))
        })?));

    let publisher = Arc::clone(&telemetry);
    tokio::spawn(async move {
        {
            loop {
                let mut telemetry_guard = publisher.lock().await;
                let _ = telemetry_guard.publish().await;
                drop(telemetry_guard);
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }
    });

    let mut controller = Controller::new(&safety_lights, &telemetry)
        .await
        .map_err(|e| {
            let io_err = std::io::Error::other(e.to_string());
            bluer::Error::from(io_err)
        })?;
    let () = controller.new_advertisement().await.map_err(|e| {
        let io_err = std::io::Error::other(e.to_string());
        bluer::Error::from(io_err)
    })?;

    let value: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(serialize_response(Response::Off)));
    let value_read = value.clone();
    let value_write = value.clone();
    let app_handle = controller
        .new_application(value_read, value_write)
        .await
        .map_err(|e| {
            let io_err = std::io::Error::other(e.to_string());
            bluer::Error::from(io_err)
        })?;

    println!("Controller started");
    println!("Device name: {DEVICE_NAME}");
    println!("Service UUID: {SERVICE_UUID}");
    println!(
        "Characteristic UUIDs: Command: {COMMAND_CHARACTERISTIC_UUID}, Telemetry: {TELEMETRY_CHARACTERISTIC_UUID}"
    );
    println!("Waiting for commands...");

    futures::future::pending::<()>().await;

    tokio::signal::ctrl_c().await?;

    println!("Shutting down");
    drop(app_handle);
    Ok(())
}
