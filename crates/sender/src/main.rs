use protocol::{Command, Response};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process;

const DEFAULT_PORT: u16 = 7878;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  sender <PI_IP> <ON|OFF|STATUS>");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  sender <rpi-address> ON");
        process::exit(1);
    }

    let pi_ip = &args[1];
    let command_str = &args[2];

    let Ok(command) = Command::parse(command_str) else {
        eprintln!("Unknown command: {command_str}");
        eprintln!("Valid commands: ON, OFF, STATUS");
        process::exit(1);
    };

    let address = format!("{pi_ip}:{DEFAULT_PORT}");

    println!("Connecting to {address}...");

    let mut stream = match TcpStream::connect(&address) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("Could not connect to Raspberry Pi: {err}");
            process::exit(1);
        }
    };

    let message = format!("{}\n", command.as_str());

    if let Err(err) = stream.write_all(message.as_bytes()) {
        eprintln!("Failed to send command: {err}");
        process::exit(1);
    }

    let mut reader = BufReader::new(stream);
    let mut response = String::new();

    if let Err(err) = reader.read_line(&mut response) {
        eprintln!("Failed to read response: {err}");
        process::exit(1);
    }

    match Response::parse(&response) {
        Ok(response) => {
            println!("Pi response: {}", response.as_str());
        }
        Err(_) => {
            println!("Pi response: {}", response.trim());
        }
    }
}
