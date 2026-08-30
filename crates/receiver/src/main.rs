use gpio_cdev::{Chip, LineRequestFlags};
use protocol::{Command, Response};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const TCP_ADDRESS: &str = "0.0.0.0:7878";

// Raspberry Pi GPIO17 = GPIO chip line 17.
const GPIO_LINE: u32 = 17;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening GPIO...");

    let mut chip = Chip::new("/dev/gpiochip0")?;

    let handle =
        chip.get_line(GPIO_LINE)?
            .request(LineRequestFlags::OUTPUT, 0, "embedded-light")?;

    println!("GPIO {GPIO_LINE} initialized.");
    println!("Listening on {TCP_ADDRESS}");

    let listener = TcpListener::bind(TCP_ADDRESS)?;

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                println!("Client connected: {}", stream.peer_addr()?);

                if let Err(err) = handle_client(stream, &handle) {
                    eprintln!("Client error: {err}");
                }

                println!("Client disconnected.");
            }

            Err(err) => {
                eprintln!("Connection failed: {err}");
            }
        }
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    gpio: &gpio_cdev::LineHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut message = String::new();
    reader.read_line(&mut message)?;

    println!("Received: {:?}", message.trim());

    let Ok(command) = Command::parse(&message) else {
        stream.write_all(b"ERR\n")?;
        return Ok(());
    };

    let response = match command {
        Command::On => {
            println!("Turning light ON");

            gpio.set_value(1)?;

            Response::Ok
        }

        Command::Off => {
            println!("Turning light OFF");

            gpio.set_value(0)?;

            Response::Ok
        }

        Command::Status => {
            let value = gpio.get_value()?;

            if value == 1 {
                Response::On
            } else {
                Response::Off
            }
        }
    };

    let response_message = format!("{}\n", response.as_str());

    stream.write_all(response_message.as_bytes())?;

    Ok(())
}
