use rppal::gpio::Gpio;
use std::{error::Error, thread, time::Duration};

const GREEN: u8 = 17;
const RED: u8 = 27;

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;

    let mut green = gpio.get(GREEN)?.into_output();
    let mut red = gpio.get(RED)?.into_output();

    loop {
        // ON — simultaneously
        green.set_high();
        red.set_high();

        thread::sleep(Duration::from_millis(1000));

        // OFF — simultaneously
        green.set_low();
        red.set_low();

        thread::sleep(Duration::from_millis(1000));
    }
}
