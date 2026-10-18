use crate::btle::write_to_peripheral;
use btleplug::{
    api::{Characteristic, Peripheral as _},
    platform::Peripheral,
};
use communication_protocol::{
    ProtocolError,
    prelude::{deserialize_response, parse_command, serialize_command},
};
use std::io::{self, Write};
use tracing::info;

pub async fn process_command(
    peripheral: &Peripheral,
    characteristic: &Characteristic,
) -> Result<(), ProtocolError> {
    // persist connection so that auth challenge from windows persists for the lifetime of the service
    println!("Enter command: ");
    // Flush stdout to guarantee the prompt prints immediately
    io::stdout()
        .flush()
        .map_err(|e| ProtocolError::UnknownCommand(e.to_string()))?;
    let input = input_buffer()?;
    let command = parse_command(&input)?;
    let packet = serialize_command(command);
    let () = write_to_peripheral(peripheral, characteristic, &packet).await?;
    info!("Sent command: {command:?}");
    let response: Vec<u8> = peripheral
        .read(characteristic)
        .await
        .map_err(|e| ProtocolError::Bluetooth(e.to_string()))?;
    let response = deserialize_response(&response)?;
    info!("Response: {response:?}");
    Ok(())
}

fn input_buffer() -> Result<String, ProtocolError> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| ProtocolError::Unknown(e.to_string()))?;
    Ok(input.trim().to_string())
}
