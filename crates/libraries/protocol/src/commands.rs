use crate::error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    Status = 0x01,
    Preflight = 0x02,
    Arm = 0x03,
    Test = 0x04,
    Hover = 0x05,
    Loiter = 0x06,
    Waypoint = 0x07,
    Disarm = 0x08,
}

impl TryFrom<u8> for Command {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Status),
            0x02 => Ok(Self::Preflight),
            0x03 => Ok(Self::Arm),
            0x04 => Ok(Self::Test),
            0x05 => Ok(Self::Hover),
            0x06 => Ok(Self::Loiter),
            0x07 => Ok(Self::Waypoint),
            0x08 => Ok(Self::Disarm),
            value => Err(ProtocolError::UnknownCommand(value)),
        }
    }
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        value as Self
    }
}

#[must_use]
pub fn serialize_command(command: Command) -> Vec<u8> {
    vec![command.into()]
}

/// Deserializes command
///
/// # Errors
/// Returns [`ProtocolError`] if invalid data received
pub fn deserialize_command(data: &[u8]) -> Result<Command, ProtocolError> {
    let byte = *data.first().ok_or(ProtocolError::EmptyCommand)?;
    Command::try_from(byte)
}

/// Parses command
///
/// # Errors
/// Returns [`ProtocolError`] if unable to parse command
pub fn parse_command(input: &str) -> Result<Command, ProtocolError> {
    match input.to_lowercase().as_str() {
        "status" => Ok(Command::Status),
        "preflight" => Ok(Command::Preflight),
        "arm" => Ok(Command::Arm),
        "test" => Ok(Command::Test),
        "hover" => Ok(Command::Hover),
        "loiter" => Ok(Command::Loiter),
        "waypoint" => Ok(Command::Waypoint),
        "disarm" => Ok(Command::Disarm),
        _ => Err("invalid command".into()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Response {
    Ok = 0x01,
    On = 0x02,
    Off = 0x03,
}

impl TryFrom<u8> for Response {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::r#Ok),
            0x02 => Ok(Self::On),
            0x03 => Ok(Self::Off),
            value => Err(ProtocolError::UnknownCommand(value)),
        }
    }
}

impl From<Response> for u8 {
    fn from(value: Response) -> Self {
        value as Self
    }
}

#[must_use]
pub fn serialize_response(response: Response) -> Vec<u8> {
    vec![response.into()]
}

/// Decodes response
///
/// # Errors
/// Returns [`ProtocolError`] if invalid data received
pub fn deserialize_response(data: &[u8]) -> Result<Response, ProtocolError> {
    let byte = *data.first().ok_or(ProtocolError::EmptyCommand)?;
    Response::try_from(byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_round_trip() {
        for command in [Command::Arm, Command::Disarm, Command::Status] {
            let encoded = serialize_command(command);
            assert_eq!(deserialize_command(&encoded), Ok(command));
        }
    }

    #[test]
    fn rejects_unknown_command() {
        assert_eq!(
            deserialize_command(&[0xff]),
            Err(ProtocolError::UnknownCommand(0xff))
        );
    }

    #[test]
    fn rejects_empty_command() {
        assert_eq!(deserialize_command(&[]), Err(ProtocolError::EmptyCommand));
    }

    #[test]
    fn extra_payload_is_ignored_for_mvp() {
        assert_eq!(
            deserialize_command(&[Command::Arm as u8, 0xaa]),
            Ok(Command::Arm)
        );
    }
}
