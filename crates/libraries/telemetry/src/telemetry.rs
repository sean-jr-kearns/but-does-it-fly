use crate::{
    error::TelemetryError,
    telemetry_packet::{TelemetryPacket, serialize},
};
use bluer::gatt::local::CharacteristicNotifier;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Publishes telemetry
///
/// # Errors
/// Returns [`TelemetryError`] if unable to publish telemetry
pub async fn publish(
    notifier: Arc<Mutex<Option<CharacteristicNotifier>>>,
) -> Result<(), TelemetryError> {
    let telemetry = temp_telemetry_generator();
    let bytes = serialize(&telemetry)?;
    let mut guard = notifier.lock().await;
    if let Some(notifier) = &mut *guard
        && let Err(e) = notifier.notify(bytes).await
    {
        eprintln!("Failed to send notification: {e}");
    }
    drop(guard);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok(())
}

/// Generates default [`TelemetryPacket`]
fn temp_telemetry_generator() -> TelemetryPacket {
    TelemetryPacket::default()
}
