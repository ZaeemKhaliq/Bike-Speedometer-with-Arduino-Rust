# Bike speedometer with Arduino and Rust

To get started, we'll first setup Rust project, build the arduino circuit and then program the logic in Rust.

## Rust project setup

You should have `cargo` installed. Cargo is a package manager for Rust packages.

Install the following dependencies:

### Windows

You can use `winget` on Window 10 & Windows 11:

```
winget install AVRDudes.AVRDUDE ZakKemble.avr-gcc
```

OR with `scoop`

```
scoop install avr-gcc
scoop install avrdude
```

For Ubuntu or Macos, you can refer to the official doc here:
https://github.com/Rahix/avr-hal?tab=readme-ov-file#quickstart

Afterwards, install the following cargo packages:

```
cargo install cargo-generate
cargo install ravedude
```

To bootstrap an Embedded Rust project, we'll use a base template provided by `avr-hal` (Github: https://github.com/Rahix/avr-hal), which is embedded-hal abstractions for AVR microcontrollers:

```
cargo generate --git https://github.com/Rahix/avr-hal-template.git
```

Once the project is generated, navigate to `/.cargo/config.toml`, inside it you'll see `target = "avr-specs/avr-atmega328p.json"`, the default target MCU is `atmega328p`, hence the code will contain abstractions relevant to this MCU. All the other available supported MCUs can be found in `/avr-specs` folder.

Now we need to do some modifications in `/Cargo.toml` file:

- Add the following code after `[dependencies.arduino-hal]` section:

```
[dependencies.avr-device]
version = "0.5.4"
features = ['atmega32u4']
```

- Add the following code after `profile.dev` section:

```
[profile.dev.package.compiler_builtins]
overflow-checks = false
```

Now, we'll run the project once to see if everything works fine. For that, make sure you've connected the USB cable on your system to Arduino. Afterwards, find the Port Number of the connected USB, for example, in my case it's `COM6`. We'll set this port number as value for `RAVEDUDE_PORT` environment variable in the terminal. On Windows PowerShell, we'll do it with the following command:

```
$env:RAVEDUDE_PORT='COM6'
```

Finally, run the following command in the project root directory to flash the Rust program onto Arduino. The program flashed is the initial boilerplate code in `/src/main.rs`:

```
cargo run
```

If you get no errors, then the program will be working correctly and can be tested.

## Building the Arduino Circuit

Things we'll need:
- Arduino Uno (with its USB cable)
- Reed Switch (Normally Open)
- Breadboard
- Jumper wires
- 4 Digit 7-Segment LEDs

![WhatsApp Image 2024-05-19 at 03 03 22_c51b60fa](https://github.com/ZaeemKhaliq/Bike-Speedometer-with-Arduino-Rust/assets/57555591/8fde1f0f-3bef-411a-8ea8-3b2eb011d2ab)


