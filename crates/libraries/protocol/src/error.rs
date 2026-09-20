use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("empty command")]
    EmptyCommand,

    #[error("unknown command:{0}")]
    UnknownCommand(String),

    #[error("invalid state: 0x{0:02x}")]
    InvalidState(u8),

    #[error("invalid peripheral error: {0}")]
    InvalidPeripheral(String),

    #[error("bluetooth error {0}")]
    Bluetooth(String),

    #[error("pin i/o error {0}")]
    PinIO(String),

    #[error("unknown error")]
    Unknown(String),
}

impl<'a> From<&'a str> for ProtocolError {
    fn from(s: &'a str) -> Self {
        Self::Unknown(s.to_string())
    }
}
