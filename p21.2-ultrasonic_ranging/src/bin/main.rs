#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, OutputConfig, Pull};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, gpio::Output};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use hcsr04::{Hcsr04, NoTemperatureCompensation};
use log::{error, info};
use esp_backtrace as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o esp32-wrover-e -o unstable-hal -o embassy -o log -o esp-backtrace -o vscode -o zed

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

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let trig = peripherals.GPIO12.into();
    let echo = peripherals.GPIO13.into();

    let _ = spawner.spawn(ultrasonic(trig, echo).expect("Failed to spawn ultrasonic_led task"));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
/// Task to measure distance and control LED based on the distance.
///
/// # Arguments
///
/// * `trig` - The trigger pin of the HC-SR04 sensor.
/// * `echo` - The echo pin of the HC-SR04 sensor.
async fn ultrasonic(trig: AnyPin<'static>, echo: AnyPin<'static>) {
    info!("Starting ultrasonic led task");
    // Init GPIO
    let trig = Output::new(trig, Level::Low, OutputConfig::default());
    let echo = Input::new(echo, InputConfig::default().with_pull(Pull::Down));
    // Create a HC-SR04 sensor instance
    let mut hcsr04 = Hcsr04::builder()
        .trig(trig)
        .echo(echo)
        .delay(Delay)
        .temperature(NoTemperatureCompensation)
        .build();
    loop {
        // Measure distance
        let distance = match hcsr04.measure_distance().await {
            Ok(distance) => distance,
            Err(e) => {
                error!("Fail to measure {}", e);
                continue;
            }
        };
        info!("measure distance value is {} cm.", distance);
        Timer::after_millis(1000).await;
    }
}
