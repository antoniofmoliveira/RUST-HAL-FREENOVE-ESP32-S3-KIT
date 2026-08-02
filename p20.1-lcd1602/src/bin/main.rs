#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use ag_lcd::{Cursor, LcdDisplay};
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::{clock::CpuClock, main, time::Instant};
use numtoa::NumToA;
use port_expander::dev::pcf8574::Pcf8574;

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

    let boot_instant = Instant::now();

    let config = Config::default().with_frequency(Rate::from_khz(100));
    let i2c_bus = I2c::new(peripherals.I2C0, config)
        .unwrap()
        .with_sda(peripherals.GPIO14)
        .with_scl(peripherals.GPIO13);
    let mut i2c_expander = Pcf8574::new(i2c_bus, true, true, true);
    let delay = esp_hal::delay::Delay::new();

    let mut lcd: LcdDisplay<_, _> = LcdDisplay::new_pcf8574(&mut i2c_expander, delay)
        .with_cursor(Cursor::Off)
        .build();
    lcd.print("hello world!");
    delay.delay_millis(1000);

    let mut buf = [0u8; 20];

    loop {
        lcd.clear();
        lcd.set_position(0, 1);
        lcd.print("Counter: ");
        let secs_from_boot = boot_instant.elapsed().as_secs();
        let secs_as_str = secs_from_boot.numtoa_str(10, &mut buf);
        lcd.print(secs_as_str);
        delay.delay_millis(1000);
    }
}
