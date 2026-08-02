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
    main,
    mcpwm::{
        McPwm, PeripheralClockConfig,
        operator::{PwmPin, PwmPinConfig},
        timer::PwmWorkingMode,
    },
    peripherals::MCPWM0,
    time::Rate,
};

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

    let mut adc_config = AdcConfig::new();
    let mut adc1_pin32 = adc_config.enable_pin(peripherals.GPIO32, Attenuation::_11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc_config);

    let clock_cfg_result = PeripheralClockConfig::with_frequency(Rate::from_mhz(32));
    let clock_cfg = clock_cfg_result.unwrap();

    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);

    mcpwm.operator0.set_timer(&mcpwm.timer0);

    let mut pwm_pin = mcpwm
        .operator0
        .with_pin_a(peripherals.GPIO33, PwmPinConfig::UP_ACTIVE_HIGH);

    let timer_clock_cfg_result =
        clock_cfg.timer_clock_with_frequency(19999, PwmWorkingMode::Increase, Rate::from_hz(50));
    let timer_clock_cfg = timer_clock_cfg_result.unwrap();

    mcpwm.timer0.start(timer_clock_cfg);

    let delay = Delay::new();

    loop {
        let result = nb::block!(adc1.read_oneshot(&mut adc1_pin32));
        match result {
            Ok(value) => {
                let angle = remap(value as u32, 0, 4095, 0, 180);
                set_angle(&mut pwm_pin, angle as u8);
            }
            Err(e) => {
                esp_println::println!("Error reading ADC: {:?}", e);
            }
        }
        delay.delay_millis(15);
    }
}

fn set_angle(pwm_pin: &mut PwmPin<'_, MCPWM0<'_>, 0, true>, angle: u8) {
    let timestamp = remap(angle as u32, 0, 180, 500, 2500);
    pwm_pin.set_timestamp(timestamp as u16);
    log::info!("angle: {} timestamp: {}", angle, timestamp);
}

fn remap(value: u32, old_min: u32, old_max: u32, new_min: u32, new_max: u32) -> u32 {
    return ((value - old_min) * (new_max - new_min) / (old_max - old_min)) + new_min;
}
