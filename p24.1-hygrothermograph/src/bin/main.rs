#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::{
    clock::CpuClock,
    gpio::{
        DriveMode::{self},
        InputConfig, Level, Output, OutputConfig, Pull,
    },
    main,
};

use log::info;

use esp_backtrace as _;
use hygrothermograph::Dht11;

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

    let pin21 = Output::new(
        peripherals.GPIO14,
        Level::Low,
        OutputConfig::default()
            .with_drive_mode(DriveMode::OpenDrain)
            .with_pull(Pull::None),
    );
    let mut pin21_flex: esp_hal::gpio::Flex<'_> = pin21.into_flex();
    pin21_flex.apply_input_config(&InputConfig::default());
    pin21_flex.set_input_enable(true);
    pin21_flex.set_output_enable(true);

    let delay = esp_hal::delay::Delay::new();

    let mut dht11 = Dht11::new(pin21_flex, delay);

    loop {
        let reading = dht11.read().unwrap();
        info!(
            "Humidity: {}.{}%, Temperature: {}.{}°C",
            reading.humidity,
            reading.humidity_decimal,
            reading.temperature,
            reading.temperature_decimal
        );

        delay.delay_millis(1000);
    }
}
