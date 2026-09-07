use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ReferenceFrame {
    Unknown = 0,
    Body = 1,
    Ned = 2,
    Enu = 3,
    Ecef = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Telemetry {
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TelemetryPacket {
    pub version: u8,
    pub reference_frame: u8,

    pub timestamp_ms: u32,

    // Position
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,

    // Velocity
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,

    // Acceleration
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,

    // Orientation quaternion: w, x, y, z
    pub quat_w: f32,
    pub quat_x: f32,
    pub quat_y: f32,
    pub quat_z: f32,
}
