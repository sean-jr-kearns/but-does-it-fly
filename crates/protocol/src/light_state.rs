use crate::error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LightState {
    Off = 0x00,
    On = 0x01,
}

impl TryFrom<u8> for LightState {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Off),
            0x01 => Ok(Self::On),
            value => Err(ProtocolError::InvalidState(value)),
        }
    }
}

impl From<LightState> for u8 {
    fn from(value: LightState) -> Self {
        value as Self
    }
}

#[must_use]
pub fn encode_state(state: LightState) -> Vec<u8> {
    vec![state.into()]
}

/// Decodes state
///
/// # Errors
/// Returns [`ProtocolError`] error if invalid data received
pub fn decode_state(data: &[u8]) -> Result<LightState, ProtocolError> {
    let byte = *data.first().ok_or(ProtocolError::EmptyCommand)?;
    LightState::try_from(byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trip() {
        for state in [LightState::Off, LightState::On] {
            let encoded = encode_state(state);
            assert_eq!(decode_state(&encoded), Ok(state));
        }
    }

    #[test]
    fn rejects_invalid_state() {
        assert_eq!(
            decode_state(&[0xff]),
            Err(ProtocolError::InvalidState(0xff))
        );
    }
}
