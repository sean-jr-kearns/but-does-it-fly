mod controller;
mod gpio;
mod safety_lights;
mod telemetry;
use communication_protocol::{
    COMMAND_CHARACTERISTIC_UUID, DEVICE_NAME, SERVICE_UUID, TELEMETRY_CHARACTERISTIC_UUID,
};
use communication_protocol::{Response, prelude::encode_response};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::controller::Controller;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    env_logger::init();

    let mut controller = Controller::new().await.map_err(|e| {
        let io_err = std::io::Error::other(e.to_string());
        bluer::Error::from(io_err)
    })?;
    let _ = controller.new_advertisement().await.map_err(|e| {
        let io_err = std::io::Error::other(e.to_string());
        bluer::Error::from(io_err)
    })?;

    let value: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(encode_response(Response::Off)));
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

    tokio::signal::ctrl_c().await?;

    println!("Shutting down");
    drop(app_handle);
    Ok(())
}
