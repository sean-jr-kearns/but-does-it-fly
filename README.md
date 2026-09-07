# but-does-it-fly
This project is a toy problem to play around with using rust for an embedded use case. The basic idea is to start with a functioning DIY drone and chunk in features to get better aquanted with embedded rust. I'm starting with the [Hawks F450 Drone Kit](https://www.hawks-work.com/products/f450-drone-kit-to-build-diy-450mm-wheelbase-4-axis-multi-rotor-drone-kit-b). This project will be considered complete when a hover and graceful nearby waypoint command can be issued from a device (e.g. base station pc) to the drone (rpi). Several implementation subgoals are outlined below to achieve automated commanding:

## Software Roadmap
Currently BLE commands are setup to turn a gpio pinned light on and off. Device pairing is a bit tedious right now as each time the sender starts it must re-pair

- `crates/light-show`: red (left)/green (right) lights flashing for external directional awareness of the drone now
- `crates/(protocol,receiver,sender)`: ack light rx (blue) tx (white) for automated commanding
- `TBD`: tie light-show features into arm/disarm commands
- `TBD`: tie automated commanding to switch on controller for safety
- `TBD`: verify light ack responds as expected to controller switch
- `TBD`: setup bridge between pc and the pi (bluetooth)
- `TBD`: test command and resp from pc to pi
- `TBD`: test motor control command (no props) from pc to pi to pixhawk telem?
- `TBD`: test hover command 
- `TBD`: test touchdown command
- `TBD`: test slight manuever command

## Hardware
- [Hawks F450 Drone Kit](https://www.hawks-work.com/products/f450-drone-kit-to-build-diy-450mm-wheelbase-4-axis-multi-rotor-drone-kit-b) 
- [Raspberry Pi 4B](https://www.raspberrypi.com/products/raspberry-pi-4-model-b/) 
- [Micro SSD](https://www.samsung.com/us/memory-storage/memory-card/pro-endurance-adapter-microsdxc-64gb-sku-mb-mj64ka-am/)
- [Accelerometer](https://www.amazon.com/MPU-6050-Accelerometer-Gyroscope-Compatible-Raspberry/dp/B0FS7959C4/ref=dp_prsubs_d_sccl_2/146-5467147-6875625?pd_rd_w=dMZ5n&content-id=amzn1.sym.1d1ff68c-b3a9-481c-9b7c-7ddc350fb821&pf_rd_p=1d1ff68c-b3a9-481c-9b7c-7ddc350fb821&pf_rd_r=3VMWCBYE5RV765WB1X59&pd_rd_wg=RQx0y&pd_rd_r=bfd16dbe-36cc-474a-9360-3af9b8a12cae&pd_rd_i=B0FS7959C4&psc=1)
- [Camera](https://www.amazon.com/dp/B0CQ4DTQZ2?ref=clp_cat_p_1)
- lidar - TBD
- sound - TBD
- power button - TBD
- power step down (rpi to 11v battery from drone kit) - TBD

## Layout

### Overall
Follows drone kit setup guide (excluding rpi, camera, accelerometer, other peripherals).
 
#### Hardware
##### Complete
- Drone general setup
- Rpi general setup

##### TODO
- Drone landing gear
- Stepdown power from drone frame (where battery is connected) to rpi 
- Camera to rpi
- IMU to rpi
- Power button to rpi

#### Software
- Mod out bluetooth in protocol with non-interactive device auth
- Make sender service persistent so that the bluetooth connection persists appropriately
- Figure out how to set SERVICE_UUID/COMMAND_CHARACTERISTIC_UUID at build time 
- Basically need to figure out how to handle device startup & bluetooth connectivity gracefully/securely
- Create ARM/DISARM command
- Pull light-show pins/lights into the ARM/DISARM commands (protocol and receiver)
- Tie in Pixhawk telem on receive over bluetooth
- Tie in motor command 

### Bluetooth Setup

#### Rpi (drone)
This project uses BLE via `bluer` to send commands and receive telemetry. The rpi may require bluetooth setup as seen below.

```bash
# Install deps
sudo apt update
sudo apt install -y bluetooth bluez 
# Enable Bluetooth:
sudo systemctl enable bluetooth
sudo systemctl start bluetooth
# Check the adapter:
bluetoothctl list
# ...
# controller <MAC_ADDRESS> <DEVICE_NAME> [default]
bluetoothctl show
#... 
# Powered: yes
```

#### PC (ground station)
Follow device pairing prompts. Right now each time the `sender` service runs the device pairing lingers but does not work for the following service execution (still need to fix this).

### light-show Setup
Lower resistence here so that the safety lights are vibrant

- rpi gpio 17 -> 220 ohm resistor -> green led -> ground
- rpi gpio 27 -> 220 ohm resistor -> red led -> ground

#### tx (In Progress)
- base station (builder pc)

#### rx (In Progress)
Higher resistence here for pin safety due to lower visibility requirement

- rpi gpio 22 -> 303 ohm resistor -> blue led -> ground

## Building
The `protocol`, `sender`, and `reciever` crates in this project are built for various targets based on their usage. Supported targets are as follows: 

- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-gnu`


This is because the base station is `win/linux x86_64` and the onboard device is `arm64`. 

### Adding Targets
See `.devcontainer/Dockerfile` for target pre-requisites.

```bash
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu
```

### light-show
```bash
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build -p light-show --target aarch64-unknown-linux-gnu --release
```

### protocol

#### Linux ARM64
```bash
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
DEVICE_NAME="flyer-1" \
SERVICE_UUID="0x1234_5678_1234_5678_1234_5678_9abc_def0" \
COMMAND_CHARACTERISTIC_UUID="0x1234_5678_1234_5678_1234_5678_9abc_def1" \
cargo build -p communication-protocol --target aarch64-unknown-linux-gnu --release
```

#### Linux x86_64
```bash
DEVICE_NAME="flyer-1" \
SERVICE_UUID="0x1234_5678_1234_5678_1234_5678_9abc_def0" \
COMMAND_CHARACTERISTIC_UUID="0x1234_5678_1234_5678_1234_5678_9abc_def1" \
cargo build -p communication-protocol --target x86_64-unknown-linux-gnu --release
```

### Sender
#### Linux ARM64
```bash
PKG_CONFIG_DIR="" \
PKG_CONFIG_SYSROOT_DIR="/" \
PKG_CONFIG_LIBDIR="/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig" \
PKG_CONFIG_ALLOW_CROSS=1 \
BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/aarch64-linux-gnu" \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
cargo build -p sender --target aarch64-unknown-linux-gnu --release
```

#### Linux x86_64
```bash
cargo build -p sender --target x86_64-unknown-linux-gnu --release
```

#### Windows x86_64
```bash
cargo build -p sender --target x86_64-pc-windows-gnu --release
```

### Receiver
**Note** the service/command uuids and device name are examples.

#### Linux ARM64
```bash
PKG_CONFIG_DIR="" \
PKG_CONFIG_SYSROOT_DIR="/" \
PKG_CONFIG_LIBDIR="/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig" \
PKG_CONFIG_ALLOW_CROSS=1 \
BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/aarch64-linux-gnu" \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
cargo build -p receiver --target aarch64-unknown-linux-gnu --release
```

#### Linux x86_64
```bash
cargo build -p receiver --target x86_64-unknown-linux-gnu --release
```

## Running
**Note**: the settings (timing, ports, etc) are all fixed right now. One of the first tasks after proving the pin commands works is to allow runtime configuration where appropriate.

### RPI

To get the service on the rpi I have been `scp`'ing it with"

```bash
scp ./target/aarch64-unknown-linux-gnu/release/sender user@ip:/opt/but-does-it-fly
```

To run 

```bash
sudo <service-path>/receiver
# ...
# Bluetooth adapter: hci0
# Bluetooth address: XX:XX:XX:XX:XX:XX
# BLE light receiver started.
# Device name: <?>
# GPIO BCM pin: 23
# Service UUID: <?>
# Characteristic UUID: <?>
# Waiting for BLE commands...
```

### Ground Station
To send from the ground station pc bluetooth must be running

```bash
# To start in container
sudo service dbus start
sudo service bluetooth start
# To start local (linux)
# sudo systemctl start dbus
# sudo systemctl start bluetooth
# To enable local (linux)
#sudo systemctl enable dbus
#sudo systemctl enable bluetooth
# To verify local (linux)
#ls -la /run/dbus/system_bus_socket
```

```bash
cargo run -p sender -- on
cargo run -p sender -- off
cargo run -p sender -- status
```

## Linting
```bash 
cargo fmt # Format
cargo machete # Unused depenency check 
cargo sort # Sort imports
cargo clippy --all-targets # Lint
```

## Testing
Trait mocks are used for hardware interactions. Test the stack with

```bash
cargo test # --doc # or --doc to just test examples
```

## Docs
Build the docs with 

```bash
cargo doc --lib --no-deps
```

## Troubleshooting
Sometimes on devcontainer restart may need to `sudo chown -R $(whoami):$(id -g) /usr/local/cargo` if different user