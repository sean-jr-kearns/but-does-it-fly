use crate::error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    On = 0x01,
    Off = 0x02,
    Status = 0x03,
}

impl TryFrom<u8> for Command {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Off),
            0x03 => Ok(Self::Status),
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
pub fn encode_command(command: Command) -> Vec<u8> {
    vec![command.into()]
}

/// Decodes command
///
/// # Errors
/// Returns [`ProtocolError`] error if invalid data received
pub fn decode_command(data: &[u8]) -> Result<Command, ProtocolError> {
    let byte = *data.first().ok_or(ProtocolError::EmptyCommand)?;
    Command::try_from(byte)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_round_trip() {
        for command in [Command::On, Command::Off, Command::Status] {
            let encoded = encode_command(command);
            assert_eq!(decode_command(&encoded), Ok(command));
        }
    }

    #[test]
    fn rejects_unknown_command() {
        assert_eq!(
            decode_command(&[0xff]),
            Err(ProtocolError::UnknownCommand(0xff))
        );
    }

    #[test]
    fn rejects_empty_command() {
        assert_eq!(decode_command(&[]), Err(ProtocolError::EmptyCommand));
    }

    #[test]
    fn extra_payload_is_ignored_for_mvp() {
        assert_eq!(decode_command(&[Command::On as u8, 0xaa]), Ok(Command::On));
    }
}
