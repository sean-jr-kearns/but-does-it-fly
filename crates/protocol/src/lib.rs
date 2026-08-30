#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    On,
    Off,
    Status,
}

impl Command {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::On => "ON",
            Self::Off => "OFF",
            Self::Status => "STATUS",
        }
    }

    /// Parses command
    ///
    /// # Arguments
    /// `input` - the command input to be parsed
    ///
    /// # Errors
    /// Returns an error if an invalid command is parsed
    pub fn parse(input: &str) -> Result<Self, String> {
        match input.trim().to_ascii_uppercase().as_str() {
            "ON" => Ok(Self::On),
            "OFF" => Ok(Self::Off),
            "STATUS" => Ok(Self::Status),
            _ => Err("Invalid input".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Ok,
    On,
    Off,
    Error,
}

impl Response {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::On => "ON",
            Self::Off => "OFF",
            Self::Error => "ERR",
        }
    }

    /// Parses response
    ///
    /// # Arguments
    /// `input` - the response input to be parsed
    ///
    /// # Errors
    /// Returns an error if an invalid response is parsed
    pub fn parse(input: &str) -> Result<Self, String> {
        match input.trim().to_ascii_uppercase().as_str() {
            "OK" => Ok(Self::Ok),
            "ON" => Ok(Self::On),
            "OFF" => Ok(Self::Off),
            "ERR" => Ok(Self::Error),
            _ => Err("Invalid response".to_string()),
        }
    }
}
