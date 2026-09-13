use btleplug::api::{
    Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use communication_protocol::{COMMAND_CHARACTERISTIC_UUID, ProtocolError, SERVICE_UUID};
use futures::stream::StreamExt;
use tracing::info;

pub async fn find_adapter() -> Result<Adapter, ProtocolError> {
    let manager = Manager::new().await.map_err(|_| {
        ProtocolError::Bluetooth("adapter manager initialization unsuccessful".to_string())
    })?;
    let adapters = manager
        .adapters()
        .await
        .map_err(|_| ProtocolError::Bluetooth("available adapters not found".to_string()))?;
    if adapters.is_empty() {
        return Err(ProtocolError::Bluetooth(
            "failed to find adapter.".to_string(),
        ));
    }
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or(None)
        .map_err(|_: Option<String>| {
            ProtocolError::Bluetooth("bluetooth adapter access unsuccessful".to_string())
        })?;
    Ok(adapter)
}

pub async fn find_device(adapter: &Adapter) -> Result<Option<PeripheralId>, ProtocolError> {
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|_| ProtocolError::Bluetooth("device peripheral scan unsuccessful".to_string()))?;
    let mut events = adapter.events().await.map_err(|_| {
        ProtocolError::Bluetooth("device peripheral event access unsuccessful".to_string())
    })?;
    let mut target: Option<PeripheralId> = None;
    info!("Scanning for device service UUID...");
    while let Some(event) = events.next().await {
        if let CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) = event
            && let Some(peripheral) = adapter
                .peripherals()
                .await
                .map_err(|_| {
                    ProtocolError::Bluetooth("device peripheral access unsuccessful".to_string())
                })?
                .iter()
                .find(|p| p.id() == id)
            && let Ok(Some(properties)) = peripheral.properties().await
        {
            let matches = properties
                .services
                .iter()
                .any(|u| u.to_string().to_lowercase() == SERVICE_UUID.to_string().to_lowercase());
            if matches {
                info!("Device found");
                target = Some(id);
                break; // Exit immediately
            }
        }
    }

    std::mem::drop(events);
    let _ = adapter.stop_scan().await;
    Ok(target)
}

pub async fn find_device_peripheral(
    adapter: &Adapter,
    target: &Option<PeripheralId>,
) -> Result<Peripheral, ProtocolError> {
    if let Some(addr) = target {
        info!("Injecting target ({addr}) into clean adapter...");
        // Get clean handle
        let peripherals = adapter.peripherals().await.map_err(|_| {
            ProtocolError::Bluetooth("device peripheral access unsuccessful".to_string())
        })?;
        if let Some(peripheral) = peripherals.into_iter().find(|p| p.id() == *addr) {
            info!("Starting isolated GattSession to device...");
            return Ok(peripheral);
        }
    }

    Err(ProtocolError::Bluetooth(
        "unable to establish gatt session".to_string(),
    ))
}

/// Get peripheral characteristic by id
///
/// # Errors
/// Returns [`ProtocolError`] if unable to find the characteristic
pub fn find_advertised_characteristic_by_id(
    peripheral: Peripheral,
) -> Result<Characteristic, ProtocolError> {
    let characteristic = peripheral
        .characteristics()
        .into_iter()
        .find(|characteristic| characteristic.uuid == COMMAND_CHARACTERISTIC_UUID)
        .ok_or("command characteristic not found")?;
    Ok(characteristic)
}

/// Writes packet to peripheral characteristic
///
/// # Returns [`ProtocolError`] if unable to write the packet
pub async fn write_to_peripheral(
    peripheral: Peripheral,
    characteristic: &Characteristic,
    packet: &Vec<u8>,
) -> Result<(), ProtocolError> {
    peripheral
        .write(characteristic, packet, WriteType::WithResponse)
        .await
        .map_err(|_| {
            ProtocolError::Bluetooth("unable to write to peripheral characteristic".to_string())
        })?;
    Ok(())
}

pub fn read_from_peripheral() {}
