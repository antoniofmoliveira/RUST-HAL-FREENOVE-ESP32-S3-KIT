#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    main,
    time::{Duration, Instant},
};
use log::info;

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
    // generator parameters: --chip esp32 -o esp32-wrover-e -o unstable-hal -o log -o esp-backtrace -o zed -o vscode

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

    const MAX_DISTANCE: u64 = 700;
    const TIME_OUT: Duration = Duration::from_micros(MAX_DISTANCE * 60);

    let mut trigger = Output::new(peripherals.GPIO12, Level::High, OutputConfig::default());
    let echo = Input::new(
        peripherals.GPIO13,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
    );
    let delay = Delay::new();
    loop {
        delay.delay_millis(1000);
        let mut out_of_range = false;
        trigger.set_low();
        delay.delay_micros(2);
        trigger.set_high();
        delay.delay_micros(10);
        trigger.set_low();

        while echo.is_low() {}
        let echo_start = Instant::now();
        while echo.is_high() {
            if echo_start.elapsed() > TIME_OUT {
                out_of_range = true;
                break;
            }
        }
        let echo_end = Instant::now();

        if out_of_range {
            info!("Out of range");
        } else {
            let echo_duration = echo_end - echo_start;
            let distance = (echo_duration.as_micros() as f32) / 58.0;
            info!(
                "Distance: {} cm, echo_duration: {} us",
                distance,
                echo_duration.as_micros()
            );
        }
    }
}
