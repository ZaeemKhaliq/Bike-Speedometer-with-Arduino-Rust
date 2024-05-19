# Bike speedometer with Arduino and Rust

The digit numbers are quite flickering in the following video because the max FPS of the camera is 60fps. In real life, the digits display fine and there is only a little bit of flickering.

https://github.com/ZaeemKhaliq/Bike-Speedometer-with-Arduino-Rust/assets/57555591/15ac6d68-2fc3-45c3-9ff6-23b479487148



To get started, we'll first setup Rust project and then build the arduino circuit. The Rust program containing the logic for speed calculation and displaying the result on LED is located in **/src/main.rs** file in this repo.

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

Finally, run the following command in the project root directory to flash the Rust program onto Arduino. The program flashed is the initial boilerplate code in `/src/main.rs` (this is where we'll write the code for speed calculation):

```
cargo run
```

If you get no errors, then the program will be working correctly and can be tested.

## Building the Arduino Circuit

To build the prototype circuit, we'll need:

- Arduino Uno (with its USB cable)
- Reed Switch (Normally Open)
- Breadboard
- Jumper wires
- 4 Digit 7-Segment LEDs
- A magnet (it will be attached to bike's wheel and will pass by the fixed reed switch)

![WhatsA![Uploading 4-digit 7-segment Speedometer with Arduino.drawio.svg…]()
pp Image 2024-05-19 at 03 03 22_c51b60fa](https://github.com/ZaeemKhaliq/Bike-Speedometer-with-Arduino-Rust/assets/57555591/8fde1f0f-3bef-411a-8ea8-3b2eb011d2ab)

### Schematic

![4-digit 7-segment Speedometer with Arduino-1](https://github.com/ZaeemKhaliq/Bike-Speedometer-with-Arduino-Rust/assets/57555591/1d39688c-5242-4998-bdb3-bcddce09378e)

12 Digital Pins of Arduino are connected to 12 pins of 7-segment LEDs. The first four are control pins for each digit, making it ON or OFF. Since it's a common cathode 7-segment, the digits are ON when the input to the pins is LOW and OFF when input is HIGH. For example, if the first four digital pins have output 1000 (first pin is HIGH and others are LOW), the last three of the digits on 7-segment will be lit up.

The rest of the 8 pins (D4-D11) are used to control each segment's display to display the speed value. The 8th pin (DP on 7-segment) is for the decimal point.

Also attached is an external pull-up resistor, which keeps the **Analog Input** pin (A0) to HIGH state when reed switch is open. We are using LOW state on the A0 pin for detection of when the magnet passes by the reed switch. When the magnet passes by the switch, it closes it, causing the current to direct towards the **GND** (away from analog input pin) on the Arduino, hence the A0 pin becomes LOW and we use that to perform calculations for speed.
