#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
};

// use log::info;

use esp_backtrace as _;

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

    let mut adc_config = AdcConfig::new();
    let mut adc1_pin32 = adc_config.enable_pin(peripherals.GPIO32, Attenuation::_0dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc_config);

    let mut led2 = Output::new(peripherals.GPIO2, Level::High, OutputConfig::default());

    let delay = Delay::new();
    
    let use_as_doorbell = false;

    loop {
        let result = nb::block!(adc1.read_oneshot(&mut adc1_pin32));
        match result {
            Ok(value) => {
                let vol = (value as f32 * 3.3) / 4095.0;
                // esp_println::println!("Voltage: {}, ADC: {}", vol, value)
                if use_as_doorbell {
                    if vol > 2.0 {
                        led2.set_low();
                    } else {
                        led2.set_high();
                    }
                } else {
                    if vol > 2.0 {
                        led2.toggle();
                    }
                }
            }
            Err(e) => {
                esp_println::println!("Error reading ADC: {:?}", e);
            }
        }
        delay.delay_millis(500);
    }
}
