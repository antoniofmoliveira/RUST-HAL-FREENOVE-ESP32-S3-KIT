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
    delay,
    gpio::{Level, Output, OutputConfig},
    main,
    time::Rate,
};

use esp_backtrace as _;
use esp_hal::spi::{
    Mode,
    master::{Config, Spi},
};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn write_data(spi: &mut Spi<'_, esp_hal::Async>, latch_pin: &mut Output, &data: &u8) {
    latch_pin.set_low();
    spi.write(&[data]).ok();
    latch_pin.set_high();
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

    // 74HC595 pin RCLK 12
    let mut latch_pin = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());

    let mut spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(1000))
            .with_mode(Mode::_0), // Mode 0 (Clock Polarity 0, Clock Phase 0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO14) // clock 74HC595 pin SRCLK 11
    .with_mosi(peripherals.GPIO12) // data 74HC595 pin SER 14
    // .with_miso(peripherals.GPIO2); // 74HC595 is an output-only device
    .into_async();

    let num: [u8; 16] = [
        0b00000011, // 0
        0b10011111, // 1
        0b00100101, // 2
        0b00001101, // 3
        0b10011001, // 4
        0b01001001, // 5
        0b01000001, // 6
        0b00011111, // 7
        0b00000001, // 8
        0b00011001, // 9
        0b00010001, // A
        0b11000001, // B
        0b01100011, // C
        0b10000101, // D
        0b01100001, // E
        0b01110001, // F
    ];
    let delay = delay::Delay::new();

    loop {
        for n in num.iter() {
            write_data(&mut spi, &mut latch_pin, n);
            delay.delay_millis(1000);
            write_data(&mut spi, &mut latch_pin, &0b11111111);
        }

        delay.delay_millis(1000);
    }
}
