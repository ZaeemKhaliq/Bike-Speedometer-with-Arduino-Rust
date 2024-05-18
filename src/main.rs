#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::{mem, sync::atomic::Ordering};

use arduino_hal::{
    hal::port::{self, PB0, PB1, PB2, PB3, PB4, PB5, PC0, PD2, PD3, PD4, PD5, PD6, PD7},
    port::{
        mode::{Floating, Input},
        Pin,
    },
};
use embedded_hal::digital::v2::{OutputPin, PinState};
use panic_halt as _;

static mut REED_SWITCH_PIN: mem::MaybeUninit<Pin<Input<Floating>, PC0>> =
    mem::MaybeUninit::uninit();
// Calculations related data
// static mut SPEED_IN_MPH: f32 = 0.0;
static mut SPEED_IN_KPH: f32 = 0.0;
static mut TIMER: usize = 0;
const RADIUS: f32 = 9.0; // inches, modify according to your implementation
const CIRCUMFERENCE: f32 = 2.0 * 3.14 * RADIUS;
const MAX_REED_COUNTER: usize = 50; // (milliseconds) minimum time to wait between 2 consecutive inputs
static mut REED_COUNTER: usize = MAX_REED_COUNTER;
#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn TIMER1_COMPA() {
    let reed_pin = unsafe { &mut *REED_SWITCH_PIN.as_mut_ptr() };
    let is_reed_switch_closed = reed_pin.is_low();

    unsafe {
        if is_reed_switch_closed {
            if REED_COUNTER == 0 {
                // SPEED_IN_MPH = (56.8 * CIRCUMFERENCE) / TIMER as f32;
                SPEED_IN_KPH = (91.44 * CIRCUMFERENCE) / TIMER as f32;

                TIMER = 0;
                REED_COUNTER = MAX_REED_COUNTER;
            } else {
                if REED_COUNTER > 0 {
                    REED_COUNTER -= 1;
                }
            }
        } else {
            if REED_COUNTER > 0 {
                REED_COUNTER -= 1;
            }
        }

        if TIMER > 2000 {
            SPEED_IN_KPH = 0.0;
        } else {
            TIMER += 1;
        }
    }
}

const NUM_DIGITS: usize = 4;
const NUM_SEGMENTS: usize = 8;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    /* Initialize SevSeg start */
    let mode: u8 = 0; // 0 -> common cathode, 1 -> common anode
    let digit_on: PinState;
    let digit_off: PinState;
    let seg_on: PinState;
    let seg_off: PinState;
    if mode == 1 {
        digit_on = PinState::from(true);
        digit_off = PinState::from(false);
        seg_on = PinState::from(false);
        seg_off = PinState::from(true);
    } else {
        digit_on = PinState::from(false);
        digit_off = PinState::from(true);
        seg_on = PinState::from(true);
        seg_off = PinState::from(false);
    }
    let digit_pins: (
        Pin<Input<Floating>, PD2>,
        Pin<Input<Floating>, PD3>,
        Pin<Input<Floating>, PD4>,
        Pin<Input<Floating>, PD5>,
    ) = (pins.d2, pins.d3, pins.d4, pins.d5);
    let segment_pins: (
        Pin<Input<Floating>, PD6>,
        Pin<Input<Floating>, PD7>,
        Pin<Input<Floating>, PB0>,
        Pin<Input<Floating>, PB1>,
        Pin<Input<Floating>, PB2>,
        Pin<Input<Floating>, PB3>,
        Pin<Input<Floating>, PB4>,
        Pin<Input<Floating>, PB5>,
    ) = (
        pins.d6, pins.d7, pins.d8, pins.d9, pins.d10, pins.d11, pins.d12, pins.d13,
    );
    let mut lights: [[bool; NUM_SEGMENTS]; NUM_DIGITS] = [[false; NUM_SEGMENTS]; NUM_DIGITS];
    let mut nums: [usize; 4] = [0; 4];
    let mut speed_value: f32 = 8888.0;
    let mut dec_place: usize = 0;
    let mut is_negative: bool;
    let brightness: u32 = 4000;

    // Set pin modes as output
    let mut d_pins: [port::Pin<port::mode::Output>; 4] = [
        digit_pins.0.into_output().downgrade(),
        digit_pins.1.into_output().downgrade(),
        digit_pins.2.into_output().downgrade(),
        digit_pins.3.into_output().downgrade(),
    ];
    for d_pin_index in 0..=3 {
        d_pins[d_pin_index].set_state(digit_off).unwrap();
    }

    let mut s_pins: [port::Pin<port::mode::Output>; 8] = [
        segment_pins.0.into_output().downgrade(),
        segment_pins.1.into_output().downgrade(),
        segment_pins.2.into_output().downgrade(),
        segment_pins.3.into_output().downgrade(),
        segment_pins.4.into_output().downgrade(),
        segment_pins.5.into_output().downgrade(),
        segment_pins.6.into_output().downgrade(),
        segment_pins.7.into_output().downgrade(),
    ];
    for s_pin_index in 0..=7 {
        s_pins[s_pin_index].set_state(seg_off).unwrap();
    }

    /* Initialize SevSeg end */

    let reed_pin = pins.a0.into_floating_input(); // Externally pulled-up to set HIGH at no input

    unsafe {
        REED_SWITCH_PIN = mem::MaybeUninit::new(reed_pin);
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }

    // To achieve 1kHz timer interrupt frequency, the OCR value should be:
    // output compare register = [(arduino clock speed) / (pre-scaler * desired frequency)] - 1
    // ocr = [(16*10^6) / (64 * 1000)] - 1 = 249
    let tc1 = dp.TC1;
    tc1.tccr1a
        .write(|w| w.wgm1().bits(0b1011).com1a().match_toggle());
    tc1.tccr1b
        .write(|w| w.wgm1().bits(0b1011).cs1().prescale_64());
    tc1.ocr1a.write(|w| unsafe { w.bits(249) });
    tc1.timsk1.write(|w| unsafe { w.ocie1a().set_bit() });

    unsafe {
        avr_device::interrupt::enable();
    }
    loop {
        /* Print output start */
        let delay_value = brightness;

        for s_pin_index in 0..=7 {
            s_pins[s_pin_index].set_state(seg_on).unwrap();

            for d_pin_index in 0..=3 {
                if lights[d_pin_index][s_pin_index] {
                    d_pins[d_pin_index].set_state(digit_on).unwrap();
                }
            }

            arduino_hal::delay_us(delay_value);

            for d_pin_index in 0..=3 {
                if lights[d_pin_index][s_pin_index] {
                    d_pins[d_pin_index].set_state(digit_off).unwrap();
                }
            }

            s_pins[s_pin_index].set_state(seg_off).unwrap();
        }

        /* Print output end */

        // Set new num
        speed_value = unsafe { SPEED_IN_KPH };
        dec_place = 2;

        /* Find numbs start */
        if speed_value < 0.0 {
            is_negative = true;
            speed_value = speed_value * -1.0;
        } else {
            is_negative = false;
        }

        //If the number is out of range, just display '----'
        if (is_negative == false && speed_value > 9999.0)
            || (is_negative == true && speed_value > 999.0)
        {
            nums[0] = 21;
            nums[1] = 21;
            nums[2] = 21;
            nums[3] = 21;
        } else {
            //Find the four digits
            let mut total = speed_value as usize;
            if is_negative == false {
                nums[0] = (speed_value / 1000.0) as usize;
                total = total - nums[0] * 1000;
            } else {
                nums[0] = 21;
            }
            nums[1] = total / 100;
            total = total - nums[1] * 100;
            nums[2] = total / 10;
            nums[3] = total - nums[2] * 10;

            //If there are zeros, set them to 20 which means a blank
            //However, don't cut out significant zeros
            if is_negative == false {
                if nums[0] == 0 && dec_place < 3 {
                    nums[0] = 20;
                    if nums[1] == 0 && dec_place < 2 {
                        nums[1] = 20;
                        if nums[2] == 0 && dec_place == 0 {
                            nums[2] = 20;
                        }
                    }
                }
            } else {
                if nums[1] == 0 && dec_place < 2 {
                    nums[1] = 20;
                    if nums[2] == 0 && dec_place == 0 {
                        nums[2] = 20;
                    }
                }
            }
        }
        /* Find numbs end */

        /* Create array start */
        for digit in 0..NUM_DIGITS {
            match nums[digit + 4 - NUM_DIGITS] {
                0 => {
                    lights[digit][0] = true;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = true;
                    lights[digit][5] = true;
                    lights[digit][6] = false;
                }
                1 => {
                    lights[digit][0] = false;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = false;
                    lights[digit][4] = false;
                    lights[digit][5] = false;
                    lights[digit][6] = false;
                }
                2 => {
                    lights[digit][0] = true;
                    lights[digit][1] = true;
                    lights[digit][2] = false;
                    lights[digit][3] = true;
                    lights[digit][4] = true;
                    lights[digit][5] = false;
                    lights[digit][6] = true;
                }
                3 => {
                    lights[digit][0] = true;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = false;
                    lights[digit][5] = false;
                    lights[digit][6] = true;
                }
                4 => {
                    lights[digit][0] = false;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = false;
                    lights[digit][4] = false;
                    lights[digit][5] = true;
                    lights[digit][6] = true;
                }
                5 => {
                    lights[digit][0] = true;
                    lights[digit][1] = false;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = false;
                    lights[digit][5] = true;
                    lights[digit][6] = true;
                }
                6 => {
                    lights[digit][0] = true;
                    lights[digit][1] = false;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = true;
                    lights[digit][5] = true;
                    lights[digit][6] = true;
                }
                7 => {
                    lights[digit][0] = true;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = false;
                    lights[digit][4] = false;
                    lights[digit][5] = false;
                    lights[digit][6] = false;
                }
                8 => {
                    lights[digit][0] = true;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = true;
                    lights[digit][5] = true;
                    lights[digit][6] = true;
                }
                9 => {
                    lights[digit][0] = true;
                    lights[digit][1] = true;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = false;
                    lights[digit][5] = true;
                    lights[digit][6] = true;
                }
                20 => {
                    lights[digit][0] = false;
                    lights[digit][1] = false;
                    lights[digit][2] = false;
                    lights[digit][3] = false;
                    lights[digit][4] = false;
                    lights[digit][5] = false;
                    lights[digit][6] = false;
                }
                21 => {
                    lights[digit][0] = false;
                    lights[digit][1] = false;
                    lights[digit][2] = false;
                    lights[digit][3] = false;
                    lights[digit][4] = false;
                    lights[digit][5] = false;
                    lights[digit][6] = true;
                }
                _ => {
                    lights[digit][0] = false;
                    lights[digit][1] = false;
                    lights[digit][2] = true;
                    lights[digit][3] = true;
                    lights[digit][4] = true;
                    lights[digit][5] = false;
                    lights[digit][6] = true;
                }
            }

            //Set the decimal place lights
            if NUM_DIGITS - digit - 1 == dec_place {
                lights[digit][7] = true;
            } else {
                lights[digit][7] = false;
            }
        }
        /* Create array end */
    }
}
