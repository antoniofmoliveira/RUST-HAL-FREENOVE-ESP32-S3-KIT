#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{DriveMode, OutputConfig};
use esp_hal::ledc::LSGlobalClkSource;
use esp_hal::main;
use esp_hal::rng::TrngSource;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output},
    ledc::{
        Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace},
        timer::{self, TimerIFace},
    },
    rng::Trng,
};
use esp_println::println;

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
    // generator parameters: --chip esp32 -o esp32-wrover-e -o vscode -o unstable-hal -o esp-backtrace

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let mut peripherals = esp_hal::init(config);

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

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let pin2 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let pin0 = Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default());
    let pin4 = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    let trng_source = TrngSource::new(peripherals.RNG, peripherals.ADC1.reborrow());
    let trng = Trng::try_new().unwrap();
    let rng = trng.downgrade();
    core::mem::drop(trng_source);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let config = timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: esp_hal::time::Rate::from_khz(24),
    };
    let r = lstimer0.configure(config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring timer0: {:?}", e),
    }

    let mut channel0: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel0, pin0);
    let mut channel2: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel2, pin2);
    let mut channel4: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel4, pin4);
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
    let r = channel2.configure(channel_config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring channel2: {:?}", e),
    }
    let r = channel4.configure(channel_config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring channel4: {:?}", e),
    }

    let delay = Delay::new();

    loop {
        let pseudo_random_number = rng.random();
        let duty = pseudo_random_number % 100;

        let r = channel0.set_duty(duty as u8);
        match r {
            Ok(_) => {}
            Err(e) => println!("Error setting channel0 duty: {:?}", e),
        }
        let r = channel2.set_duty(duty as u8);
        match r {
            Ok(_) => {}
            Err(e) => println!("Error setting channel2 duty: {:?}", e),
        }
        let r = channel4.set_duty(duty as u8);
        match r {
            Ok(_) => {}
            Err(e) => println!("Error setting channel4 duty: {:?}", e),
        }
        delay.delay_micros(1000000);
    }
}
