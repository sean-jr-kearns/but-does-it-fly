use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TelemetryError {
    #[error("empty or malformed telemetry: {0}")]
    Empty(String),

    #[error("invalid telemetry format")]
    Parse,

    #[error("unknown error: {0}")]
    Unknown(String),
}

impl<'a> From<&'a str> for TelemetryError {
    fn from(s: &'a str) -> Self {
        Self::Unknown(s.to_string())
    }
}
