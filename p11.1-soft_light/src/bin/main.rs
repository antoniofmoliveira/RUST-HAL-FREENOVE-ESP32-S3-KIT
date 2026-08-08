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
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Level, Output, OutputConfig},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace},
        timer::{self, TimerIFace},
    },
    main,
};

use esp_println::println;
use nb;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn remap(value: u32, old_min: u32, old_max: u32, new_min: u32, new_max: u32) -> u32 {
    return ((value - old_min) * (new_max - new_min) / (old_max - old_min)) + new_min;
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

    // adc1 is connected to GPIO32, which is the pin we will use to read the potentiometer value
    let mut adc_config = AdcConfig::new();
    let mut adc1_pin32 = adc_config.enable_pin(peripherals.GPIO32, Attenuation::_11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc_config);

    // ledc is connected to GPIO2, which is the pin we will use to control the LED
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // timer0 is connected to GPIO2, which is the pin we will use to control the LED
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let config = timer::config::Config {
        duty: timer::config::Duty::Duty14Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: esp_hal::time::Rate::from_hz(50),
    };
    let r = lstimer0.configure(config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring timer0: {:?}", e),
    }

    // channel0 is connected to GPIO2, which is the pin we will use to control the LED
    let pin2 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let mut channel0: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel0, pin2);
    let channel_config = channel::config::Config {
        timer: &lstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull,
    };
    let r = channel0.configure(channel_config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring channel0: {:?}", e),
    }

    let delay = Delay::new();

    loop {
        let result = nb::block!(adc1.read_oneshot(&mut adc1_pin32));
        match result {
            Ok(value) => {
                let duty = remap(value as u32, 0, 4095, 0, 100);
                let r = channel0.set_duty(duty as u8);
                match r {
                    Ok(_) => {}
                    Err(e) => println!("Error setting duty cycle: {:?}", e),
                }
            }
            Err(e) => {
                esp_println::println!("Error reading ADC: {:?}", e);
            }
        }
        delay.delay_millis(200);
    }
}
