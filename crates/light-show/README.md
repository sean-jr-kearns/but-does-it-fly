# light-show
Given pins blinks a red and green light on the drone so that observers can identify left (red) and right (green) sides of the drone. The lights must blink between 40 and 100 times per second. 

## Target
An `ARM64 Raspberry Pi 4B` is the target for these projects so run the following command to add the appropriate target. 

```bash
rustup target add aarch64-unknown-linux-gnu
```

To build for this target either setup the devcontainer with the environment variable `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc` or add it to the start of your build command (see examples below)

```bash 
# Debug
cargo build --target aarch64-unknown-linux-gnu
# Release
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --target aarch64-unknown-linux-gnu --release
```

Currently building on the base station and then SCP'ing the executable to the device. That is, 

```bash
scp ./target/aarch64-unknown-linux-gnu/release/light-show user@ip:/opt/but-does-it-fly
```

## Troubleshooting
Ran into some issues where the user didn't have `gpiomem` permissions. To update run and verify with:

```bash
sudo usermod -aG gpio $USER
exec su -l $USER
ls -l /dev/gpiomem
```

## Running
And then on the actual device the service can be started by running

```bash
/opt/but-does-it-fly/light-show
```