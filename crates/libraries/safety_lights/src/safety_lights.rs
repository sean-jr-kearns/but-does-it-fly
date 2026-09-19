use communication_protocol::ProtocolError;
use gpio_lights::{Indicator, Pin};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

pub struct SafetyLights {
    pub green_pin: Arc<Mutex<Indicator>>,
    pub red_pin: Arc<Mutex<Indicator>>,
    cancellation_token: CancellationToken,
    handle: Option<JoinHandle<Result<(), rppal::gpio::Error>>>,
}

impl SafetyLights {
    /// Creates new safety lights executor given light pin locations
    ///
    /// # Errors
    /// Returns error if unable to instantiate indicators
    pub fn new(green_pin: u8, red_pin: u8) -> Result<Self, ProtocolError> {
        let token = CancellationToken::new();
        let green_pin = Indicator::new(green_pin).map_or_else(
            |_| {
                Err(ProtocolError::InvalidPeripheral(
                    "green indicator setup unsuccessful".to_string(),
                ))
            },
            |light| Ok(Arc::new(Mutex::new(light))),
        )?;

        let red_pin = Indicator::new(red_pin).map_or_else(
            |_| {
                Err(ProtocolError::InvalidPeripheral(
                    "red indicator setup unsuccessful".to_string(),
                ))
            },
            |light| Ok(Arc::new(Mutex::new(light))),
        )?;

        Ok(Self {
            green_pin,
            red_pin,
            cancellation_token: token,
            handle: None,
        })
    }

    pub fn run(&mut self) {
        // Clone used vars for spawned thread lifetime purposes
        let green = self.green_pin.clone();
        let red = self.red_pin.clone();
        let token = self.cancellation_token.clone();

        let handle = tokio::spawn(async move {
            println!("Safety-lights started");

            loop {
                tokio::select! {
                    // Instantly exit if the stop command is issued
                    () = token.cancelled() => {
                        println!("Stop command received. Safely shutting down safety-lights...");
                        // Ensure pins are left in a safe/default state
                        let _green = green.lock().await.set_off();
                        let _red = red.lock().await.set_off();
                        break;
                    }

                    // Toggling Sequence
                    () = async {
                        let mut green = green.lock().await;
                        let mut red = red.lock().await;

                        let _ = green.set_on();
                        let _ = red.set_on();
                        sleep(Duration::from_millis(1000)).await;

                        let _ = green.set_off();
                        let _ = red.set_off();
                        sleep(Duration::from_millis(1000)).await;
                        drop(green);
                        drop(red);
                    } => {}
                }
            }

            Ok::<(), rppal::gpio::Error>(())
        });

        self.handle = Some(handle);
    }

    pub fn stop(&mut self) -> bool {
        // Issue the stop command
        println!("Issuing safety-lights stop command...");
        self.cancellation_token.cancel();

        // Wait for the loop to finish its cleanup
        self.handle = None;
        println!("Safety-lights stopped");
        true
    }
}
