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
    gpio::{Level, Output, OutputConfig},
    ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer},
    main,
    rmt::{PulseCode, Rmt, RxChannelConfig, RxChannelCreator},
    time::Rate,
};
use esp_println::println;

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

    // buzzer
    let mut buzzer = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());

    // led
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
    let mut led_channel: channel::Channel<'_, LowSpeed> =
        ledc.channel(channel::Number::Channel0, peripherals.GPIO12);

    let channelo_config = channel::config::Config {
        timer: &lstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull,
    };

    let _ = led_channel.configure(channelo_config);

    // ir receiver
    // Configure frequency based on chip type
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    let rx_config = RxChannelConfig::default()
        .with_clk_divider(80)
        .with_idle_threshold(50000)
        .with_filter_threshold(10);
    let mut ir_channel = rmt
        .channel0
        .configure_rx(&rx_config)
        .unwrap()
        .with_pin(peripherals.GPIO13);
    let mut data: [PulseCode; 48] = [PulseCode::default(); 48];

    // others
    let delay = Delay::new();

    loop {
        for entry in data.iter_mut() {
            entry.reset();
        }
        let transaction = ir_channel.receive(&mut data).unwrap();

        match transaction.wait() {
            Ok((symbol_count, channel_res)) => {
                ir_channel = channel_res;

                println!("Received {} symbols", symbol_count);

                if symbol_count == 0 {
                    continue;
                }
                if symbol_count == 2 {
                    println!("Repeat signal detected (Button held)");
                    continue;
                }

                if let Some(code) = decode_nec(&data) {
                    handle_control(&mut buzzer, &led_channel, code);
                }
            }
            Err((_err, channel_res)) => {
                ir_channel = channel_res;
                println!("Receive error, retrying...");
            }
        }

        // Optional: Small delay to prevent watchdog triggers or busy-looping
        delay.delay_millis(10);
    }
}

// NEC protocol 
// Thresholds in microseconds (since clk_divider=80 makes 1 tick = 1µs)
const HEADER_PULSE_MIN: u16 = 8000; // 8ms
const HEADER_SPACE_MIN: u16 = 3500; // 3.5ms
const BIT_1_SPACE_MIN: u16 = 1200; // >1.2ms is a '1' (Midpoint between 560 and 1680)
const BIT_PULSE_MIN: u16 = 400; // Valid pulse must be >400µs

fn decode_nec(data: &[esp_hal::rmt::PulseCode]) -> Option<u32> {
    let mut value: u32 = 0;
    let mut bit_count = 0;
    let mut header_found = false;

    for entry in data {
        let pulse = entry.length1(); // Low duration (Active)
        let space = entry.length2(); // High duration (Idle)

        if pulse == 0 {
            break;
        }

        // 1. Detect Header (9ms Pulse, 4.5ms Space)
        if !header_found {
            if pulse > HEADER_PULSE_MIN && space > HEADER_SPACE_MIN {
                header_found = true;
                // println!("Header detected!");
            }
            continue;
        }

        // 2. Decode Data Bits
        // NEC encodes data in the SPACE length. Pulse is constant (~560µs).
        if space == 0 {
            break;
        } // End of message

        // Skip if pulse is too short (noise) or too long (error)
        if pulse < BIT_PULSE_MIN {
            continue;
        }

        value <<= 1;
        if space > BIT_1_SPACE_MIN {
            value |= 1; // Long space = Logic 1
        } else {
            // Short space = Logic 0
        }

        bit_count += 1;
        if bit_count == 32 {
            return Some(value);
        }
    }
    None
}

// handle control based on the received IR code
fn handle_control(buzzer: &mut Output, led_channel: &channel::Channel<'_, LowSpeed>, code: u32) {
    let delay = esp_hal::delay::Delay::new();
    buzzer.set_high();
    delay.delay_millis(100);
    buzzer.set_low();

    match code {
        0x00FF9867 => {
            let r = led_channel.set_duty(0 as u8);
            match r {
                Ok(_) => {}
                Err(e) => {
                    esp_println::println!("Error setting duty: {:?}", e);
                }
            }
        }
        0x00FFA25D => {
            let duty = remap(7, 0, 255, 0, 100);
            let r = led_channel.set_duty(duty as u8);
            match r {
                Ok(_) => {}
                Err(e) => {
                    esp_println::println!("Error setting duty: {:?}", e);
                }
            }
        }
        0x00FF629D => {
            let duty = remap(63, 0, 255, 0, 100);
            let r = led_channel.set_duty(duty as u8);
            match r {
                Ok(_) => {}
                Err(e) => {
                    esp_println::println!("Error setting duty: {:?}", e);
                }
            }
        }
        0x00FFE21D => {
            let r = led_channel.set_duty(100);
            match r {
                Ok(_) => {}
                Err(e) => {
                    esp_println::println!("Error setting duty: {:?}", e);
                }
            }
        }
        _ => {}
    }
}

fn remap(value: u32, old_min: u32, old_max: u32, new_min: u32, new_max: u32) -> u32 {
    return ((value - old_min) * (new_max - new_min) / (old_max - old_min)) + new_min;
}
