pub trait Pin {
    /// Turns pin high
    ///
    /// # Errors
    /// Returns error if unable to manipulate pin state
    fn set_on(&mut self) -> Result<(), rppal::gpio::Error>;
    /// Turns pin low
    ///
    /// # Errors
    /// Returns error if unable to manipulate pin state
    fn set_off(&mut self) -> Result<(), rppal::gpio::Error>;
    #[warn(unused)]
    fn is_on(&self) -> bool;
}

#[cfg(test)]
pub struct MockPin {
    on: bool,
}

#[cfg(test)]
impl MockPin {
    #[must_use]
    pub const fn new() -> Self {
        Self { on: false }
    }
}

#[cfg(test)]
impl Default for MockPin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl Pin for MockPin {
    fn set_on(&mut self) -> Result<(), rppal::gpio::Error> {
        self.on = true;
        Ok(())
    }

    fn set_off(&mut self) -> Result<(), rppal::gpio::Error> {
        self.on = false;
        Ok(())
    }

    fn is_on(&self) -> bool {
        self.on
    }
}

#[cfg(target_os = "linux")]
pub struct Indicator {
    pin: rppal::gpio::OutputPin,
}

#[cfg(target_os = "linux")]
impl Indicator {
    /// Creates new gpio pin
    ///
    /// # Errors
    /// Returns error if unable to create new pin representation
    pub fn new(gpio_pin: u8) -> Result<Self, rppal::gpio::Error> {
        let gpio = rppal::gpio::Gpio::new()?;

        let pin = gpio.get(gpio_pin)?.into_output_low();

        Ok(Self { pin })
    }
}

#[cfg(target_os = "linux")]
impl Pin for Indicator {
    fn set_on(&mut self) -> Result<(), rppal::gpio::Error> {
        self.pin.set_high();
        Ok(())
    }

    fn set_off(&mut self) -> Result<(), rppal::gpio::Error> {
        self.pin.set_low();
        Ok(())
    }

    fn is_on(&self) -> bool {
        !self.pin.is_set_low()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_starts_off() {
        let light = MockPin::new();

        assert!(!light.is_on());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mock_turns_on() {
        let mut light = MockPin::new();

        light.set_on().unwrap();

        assert!(light.is_on());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mock_turns_off() {
        let mut light = MockPin::new();

        light.set_on().unwrap();
        light.set_off().unwrap();

        assert!(!light.is_on());
    }
}
