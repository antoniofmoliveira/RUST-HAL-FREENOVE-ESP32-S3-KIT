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
    delay::Delay,
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

    const WIDTH: u8 = 80;

    // Configure frequency based on chip type
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    let rx_config = RxChannelConfig::default()
        .with_clk_divider(WIDTH)
        .with_idle_threshold(50000)
        .with_filter_threshold(10);
    let mut channel = rmt
        .channel0
        .configure_rx(&rx_config)
        .unwrap()
        .with_pin(peripherals.GPIO13);
    let delay = Delay::new();
    let mut data: [PulseCode; 48] = [PulseCode::default(); 48];

    loop {
        for entry in data.iter_mut() {
            entry.reset();
        }
        let transaction = channel.receive(&mut data).unwrap();

        match transaction.wait() {
            Ok((symbol_count, channel_res)) => {
                channel = channel_res;

                println!("Received {} symbols", symbol_count);

                if symbol_count == 0 {
                    continue;
                }
                if symbol_count == 2 {
                    println!("Repeat signal detected (Button held)");
                    continue;
                }

                if let Some(code) = decode_nec(&data) {
                    let addr = (code >> 24) & 0xFF;
                    let inv_addr = (code >> 16) & 0xFF;
                    let cmd = (code >> 8) & 0xFF;
                    let inv_cmd = code & 0xFF;

                    println!("Full Code: 0x{:08X}", code);
                    println!("Addr: 0x{:02X}, Cmd: 0x{:02X}", addr, cmd);

                    if addr == (!inv_addr & 0xFF) && cmd == (!inv_cmd & 0xFF) {
                        println!("✓ Valid NEC Command!");
                    } else {
                        println!("⚠ Checksum mismatch (might be extended NEC or noise)");
                    }
                }
                println!();
            }
            Err((_err, channel_res)) => {
                channel = channel_res;
                println!("Receive error, retrying...");
            }
        }

        // Optional: Small delay to prevent watchdog triggers or busy-looping
        delay.delay_millis(10);
    }
}

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
