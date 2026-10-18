use crate::error::TelemetryError;
use postcard::to_allocvec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ReferenceFrame {
    #[default]
    Unknown = 0,
    Body = 1,
    Ned = 2,
    Enu = 3,
    Ecef = 4,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub version: u8,
    pub timestamp_ms: u32,
    pub reference_frame: ReferenceFrame,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    /// Body -> reference-frame rotation.
    /// Hamilton convention: (w, x, y, z)
    pub orientation: Quaternion,
}

/// Deserializes telemetry
///
/// # Errors
/// Returns [`TelemetryError`] if invalid telemetry received
pub fn deserialize(data: &[u8]) -> Result<TelemetryPacket, TelemetryError> {
    TelemetryPacket::try_from(data)
}

/// Serializes telemetry
///
/// # Errors
/// Returns [`TelemetryError`] if unable to serialize [`TelemetryPacket`]
pub fn serialize(t: &TelemetryPacket) -> Result<Vec<u8>, TelemetryError> {
    let bytes: Vec<u8> = to_allocvec(&t).map_err(|_| TelemetryError::Parse)?;
    Ok(bytes)
}

impl TryFrom<&[u8]> for TelemetryPacket {
    type Error = TelemetryError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        postcard::from_bytes::<Self>(value).map_or(Err(TelemetryError::Parse), Ok)
    }
}
