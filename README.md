# but-does-it-fly
This project is a toy problem to play around with using rust for an embedded use case. The basic idea is to start with a functioning DIY drone and chunk in features to get better aquanted with embedded rust. I'm starting with the [Hawks F450 Drone Kit](https://www.hawks-work.com/products/f450-drone-kit-to-build-diy-450mm-wheelbase-4-axis-multi-rotor-drone-kit-b). This project will be considered complete when a hover and graceful nearby waypoint command can be issued from a device (e.g. base station pc) to the drone (rpi). Several implementation subgoals are outlined below to achieve automated commanding:

## Software 

- `crates/light-show`: red (left)/green (right) lights flashing for external directional awareness of the drone now
- `crates/(protocol,receiver,sender)`: ack light rx (blue) tx (white) for automated commanding
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
 
#### Complete
- Drone general setup
- Rpi general setup

#### TODO
- Drone landing gear
- Stepdown power from drone frame (where battery is connected) to rpi 
- Camera to rpi
- IMU to rpi
- Power button to rpi

### light-show
Lower resistence here so that the safety lights are vibrant

- rpi gpio 17 -> 220 ohm resistor -> green led -> ground
- rpi gpio 27 -> 220 ohm resistor -> red led -> ground

### tx (In Progress)
- base station (builder pc)

### rx (In Progress)
Higher resistence here for pin safety due to lower visibility requirement

- rpi gpio 22 -> 303 ohm resistor -> blue led -> ground

## Building
The `protocol`, `sender`, and `reciever` crates in this project are meant to be built for at least 2 linux targets: 

- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`

This is because the base station is `x86_64` and the onboard device is `arm64`. `light-show` however is `arm64` only as it is specifically for `gpio` manipulation.

```bash
# light-show
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build -p light-show --target aarch64-unknown-linux-gnu --release
# protocol
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build -p protocol --target aarch64-unknown-linux-gnu --release
cargo build -p protocol --target x86_64-unknown-linux-gnu --release
# sender
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build -p sender --target aarch64-unknown-linux-gnu --release
cargo build -p sender --target x86_64-unknown-linux-gnu --release
# receiver
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build -p receiver --target aarch64-unknown-linux-gnu --release
cargo build -p receiver --target x86_64-unknown-linux-gnu --release
```

## Running
**Note**: the settings (timing, ports, etc) are all fixed right now. One of the first tasks after proving the pin commands works is to allow runtime configuration where appropriate.



TODO