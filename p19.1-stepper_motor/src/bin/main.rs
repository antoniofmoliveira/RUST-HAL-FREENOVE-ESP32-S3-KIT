#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::sync::atomic::{AtomicU8, Ordering};
use esp_hal::{
    clock::CpuClock,
    delay::{self, Delay},
    gpio::{Level, Output, OutputConfig},
    main,
};

use esp_backtrace as _;

static OUT_STATE: AtomicU8 = AtomicU8::new(0x01);

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o esp32-wrover-e -o unstable-hal -o log -o esp-backtrace -o vscode -o zed

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO0
    // - GPIO2
    // - GPIO5
    // - GPIO12
    // - GPIO15
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO6;
    let _ = peripherals.GPIO7;
    let _ = peripherals.GPIO8;
    let _ = peripherals.GPIO9;
    let _ = peripherals.GPIO10;
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO17;
    let _ = peripherals.GPIO20;

    let mut pin_a = Output::new(peripherals.GPIO12, Level::Low, OutputConfig::default());
    let mut pin_b = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    let mut pin_c = Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default());
    let mut pin_d = Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default());

    let mut ports = [&mut pin_a, &mut pin_b, &mut pin_c, &mut pin_d];
    let delay = delay::Delay::new();
    loop {
        let clockwise = true;
        move_around(&mut ports, clockwise, 1, 3);
        delay.delay_millis(1000);
        move_around(&mut ports, !clockwise, 1, 3);
        delay.delay_millis(1000);
    }
}

fn move_one_step(ports: &mut [&mut Output; 4], clockwise: bool) {
    // Load current state
    let mut byte = OUT_STATE.load(Ordering::Relaxed);

    if clockwise {
        byte = if byte != 0x08 { byte << 1 } else { 0x01 };
    } else {
        byte = if byte != 0x01 { byte >> 1 } else { 0x08 };
    }
    // Store the new state atomically
    OUT_STATE.store(byte, Ordering::Relaxed);
    for i in 0..4 {
        let bit_is_set = (byte & (0x01 << i)) != 0;

        if bit_is_set {
            ports[i].set_high();
        } else {
            ports[i].set_low();
        }
    }
}

fn move_steps(ports: &mut [&mut Output; 4], clockwise: bool, steps: u32, mut interval: u32) {
    let delay = Delay::new();
    if interval < 3 {
        interval = 3;
    }
    if interval > 20 {
        interval = 20;
    }
    for _ in 0..steps {
        move_one_step(ports, clockwise);
        delay.delay_millis(interval);
    }
}

// The stator in the stepper motor we have supplied has 32 magnetic poles. Therefore, to complete one full
// revolution requires 32 full steps. The rotor (or output shaft) of the stepper motor is connected to a speed
// reduction set of gears and the reduction ratio is 1:64. Therefore, the final output shaft (exiting the stepper
// motor’s housing) requires 32 X 64 = 2048 steps to make one full revolution
fn move_around(ports: &mut [&mut Output; 4], clockwise: bool, turns: u32, interval: u32) {
    for _ in 0..turns {
        move_steps(ports, clockwise, 32 * 64, interval);
    }
}

fn _move_angle(ports: &mut [&mut Output; 4], clockwise: bool, angle: f32, interval: u32) {
    let steps = (angle * 32.0 * 64.0) / 360.0;
    move_steps(ports, clockwise, steps as u32, interval);
}
