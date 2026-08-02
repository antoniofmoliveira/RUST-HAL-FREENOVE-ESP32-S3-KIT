#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::gpio::DriveMode;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer},
    main,
};
// use log::info;
use esp_backtrace as _;

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

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let config = timer::config::Config {
        duty: timer::config::Duty::Duty14Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: esp_hal::time::Rate::from_hz(50),
    };
    let r = lstimer0.configure(config);
    match r {
        Ok(_) => {}
        Err(e) => {
            esp_println::println!("Error configuring timer: {:?}", e);
        }
    }

    let mut channels = [
        ledc.channel(channel::Number::Channel0, peripherals.GPIO14),
        ledc.channel(channel::Number::Channel1, peripherals.GPIO27),
        ledc.channel(channel::Number::Channel2, peripherals.GPIO26),
        ledc.channel(channel::Number::Channel3, peripherals.GPIO25),
        ledc.channel(channel::Number::Channel4, peripherals.GPIO33),
        ledc.channel(channel::Number::Channel5, peripherals.GPIO32),
        ledc.channel(channel::Number::Channel6, peripherals.GPIO12),
        ledc.channel(channel::Number::Channel7, peripherals.GPIO13),
    ];

    let channelo_config = channel::config::Config {
        timer: &lstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull,
    };

    for channel in channels.iter_mut() {
        let r = channel.configure(channelo_config);
        match r {
            Ok(_) => {}
            Err(e) => {
                esp_println::println!("Error configuring channel: {:?}", e);
            }
        }
    }

    let dutys = [
        0, 0, 0, 0, 0, 0, 0, 0, 1023, 512, 256, 128, 64, 32, 16, 8, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let delay = Delay::new();
    let delay_time = 50;

    let channels_len = channels.len();

    loop {
        for i in 0..15 {
            for j in 0..(channels_len - 1) {
                let duty = remap(dutys[i + j] as u32, 0, 1023, 0, 100);
                let r = channels[j].set_duty(duty as u8);
                match r {
                    Ok(_) => {}
                    Err(e) => {
                        esp_println::println!("Error setting duty: {:?}", e);
                    }
                }
            }
            delay.delay_millis(delay_time);
        }

        for i in 0..15 {
            for j in (0..channels_len - 1).rev() {
                let duty = remap(dutys[i + (channels_len - 1 - j)] as u32, 0, 1023, 0, 100);
                let r = channels[j].set_duty(duty as u8);
                match r {
                    Ok(_) => {}
                    Err(e) => {
                        esp_println::println!("Error setting duty: {:?}", e);
                    }
                }
            }
            delay.delay_millis(delay_time);
        }
    }
}
