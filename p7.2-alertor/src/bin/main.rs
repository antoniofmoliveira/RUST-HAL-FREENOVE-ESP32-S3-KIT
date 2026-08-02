#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::f64::consts::PI;

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::ledc::{LSGlobalClkSource, Ledc, channel, timer};
use esp_hal::main;
use libm::sin;

// because crate wants hal 1.0 but we are using 1.1.0
// downloaded code and put in lib.rs
// use esp_hal_buzzer::Buzzer;
use alertor::Buzzer;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn alert(buzzer: &mut Buzzer) {
    let mut sin_val;
    let mut tone_val;
    let delay = Delay::new();
    for x in (0..359).step_by(10) {
        sin_val = sin(x as f64 * (PI / 180.0));
        tone_val = (2000.0 + (sin_val * 500.0)) as u32;
        buzzer.play(tone_val).unwrap();
        delay.delay_millis(20);
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o esp32-wrover-e -o unstable-hal -o log -o esp-backtrace -o vscode

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
    // These GPIO pins are in use by soesp32-wrover-e-me feature of the module and should not be used.
    let _ = peripherals.GPIO6;
    let _ = peripherals.GPIO7;
    let _ = peripherals.GPIO8;
    let _ = peripherals.GPIO9;
    let _ = peripherals.GPIO10;
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO17;
    let _ = peripherals.GPIO20;

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut buzzer = Buzzer::new(
        &ledc,
        timer::Number::Timer0,
        channel::Number::Channel1,
        peripherals.GPIO14,
    );

    let button_config = InputConfig::default().with_pull(Pull::Up);
    let button4 = Input::new(peripherals.GPIO4, button_config);

    let delay = Delay::new();

    loop {
        if button4.is_low() {
            delay.delay_millis(50);
            if button4.is_low() {
                alert(&mut buzzer);
            }
        } else {
            buzzer.mute();
        }
    }
}
