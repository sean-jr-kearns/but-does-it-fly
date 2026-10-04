pub mod commands;
pub mod error;
pub mod prelude;

pub use prelude::*;
use uuid::Uuid;

pub const SERVICE_UUID: Uuid = Uuid::from_u128(0x1234_5678_1234_5678_1234_5678_9abc_def0);

pub const COMMAND_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x1234_5678_1234_5678_1234_5678_9abc_def1);

pub const TELEMETRY_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x1234_5678_1234_5678_1234_5678_9abc_def2);

pub const DEVICE_NAME: &str = env!("DEVICE_NAME");
