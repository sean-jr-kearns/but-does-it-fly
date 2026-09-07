use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("empty command")]
    EmptyCommand,

    #[error("unknown command: 0x{0:02x}")]
    UnknownCommand(u8),

    #[error("invalid state: 0x{0:02x}")]
    InvalidState(u8),

    #[error("invalid peripheral error: {0}")]
    InvalidPeripheral(String),

    #[error("bluetooth error {0}")]
    Bluetooth(String),
}
