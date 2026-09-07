use std::error::Error;

pub trait LightOutput {
    fn set_on(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn set_off(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn is_on(&self) -> bool;
}

#[cfg(test)]
pub struct MockLight {
    on: bool,
}

#[cfg(test)]
impl MockLight {
    pub const fn new() -> Self {
        Self { on: false }
    }
}

#[cfg(test)]
impl Default for MockLight {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl LightOutput for MockLight {
    fn set_on(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.on = true;
        Ok(())
    }

    fn set_off(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    pub fn new(bcm_pin: u8) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let gpio = rppal::gpio::Gpio::new()?;

        let pin = gpio.get(bcm_pin)?.into_output_low();

        Ok(Self { pin })
    }
}

#[cfg(target_os = "linux")]
impl LightOutput for Indicator {
    fn set_on(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.pin.set_high();
        Ok(())
    }

    fn set_off(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
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
        let light = MockLight::new();

        assert!(!light.is_on());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mock_turns_on() {
        let mut light = MockLight::new();

        light.set_on().unwrap();

        assert!(light.is_on());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mock_turns_off() {
        let mut light = MockLight::new();

        light.set_on().unwrap();
        light.set_off().unwrap();

        assert!(!light.is_on());
    }
}
