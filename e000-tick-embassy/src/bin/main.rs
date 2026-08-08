#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::timer::timg::TimerGroup;
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
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

    // TODO: Spawn some tasks
    // let _ = spawner;
    spawner.spawn(run().unwrap());
    spawner.spawn(blink(peripherals.GPIO2.into()).unwrap());
    spawner.spawn(blink(peripherals.GPIO13.into()).unwrap());
    spawner.spawn(button_action(peripherals.GPIO0.into()).unwrap());

    loop {
        info!("Hello world!");
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn run() {
    loop {
        info!("tick");
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn blink(pin: AnyPin<'static>) {
    let mut led = Output::new(pin, Level::Low, OutputConfig::default());

    loop {
        // Timekeeping is globally available, no need to mess with hardware timers.
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn button_action(button_pin: AnyPin<'static>) {
    let mut button = Input::new(button_pin, InputConfig::default().with_pull(Pull::Up));

    loop {
        button.wait_for_low().await;
        info!("Button pressed!");

        button.wait_for_high().await;
        info!("Button released!");
    }
}
