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
    gpio::{DriveMode, Level, Output, OutputConfig},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace},
        timer::{self, TimerIFace},
    },
    main,
};

use esp_println::println;

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
    // generator parameters: --chip esp32 -o esp32-wrover-e -o log -o esp-backtrace -o vscode -o unstable-hal

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

    // potentiometers
    let mut adc_config = AdcConfig::new();
    let mut pin32 = adc_config.enable_pin(peripherals.GPIO32, Attenuation::_11dB);
    let mut pin33 = adc_config.enable_pin(peripherals.GPIO33, Attenuation::_11dB);
    let mut pin34 = adc_config.enable_pin(peripherals.GPIO34, Attenuation::_11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc_config);

    // LEDs
    let pin21 = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());
    let pin22 = Output::new(peripherals.GPIO22, Level::Low, OutputConfig::default());
    let pin23 = Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default());

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
        Err(e) => println!("Error configuring timer0: {:?}", e),
    }

    let mut channel0: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel0, pin21);
    let mut channel1: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel2, pin22);
    let mut channel2: Channel<'_, LowSpeed> = ledc.channel(channel::Number::Channel4, pin23);
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

    let r = channel1.configure(channel_config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring channel2: {:?}", e),
    }

    let r = channel2.configure(channel_config);
    match r {
        Ok(_) => {}
        Err(e) => println!("Error configuring channel4: {:?}", e),
    }

    let delay = Delay::new();
    
    loop {
        let pot1_value = nb::block!(adc1.read_oneshot(&mut pin32));
        let pot2_value = nb::block!(adc1.read_oneshot(&mut pin33));
        let pot3_value = nb::block!(adc1.read_oneshot(&mut pin34));

        let duty1 = remap(pot1_value.unwrap() as u32, 0, 4095, 0, 100);
        let duty2 = remap(pot2_value.unwrap() as u32, 0, 4095, 0, 100);
        let duty3 = remap(pot3_value.unwrap() as u32, 0, 4095, 0, 100);

        let r = channel0.set_duty(duty1 as u8);
        match r {
            Ok(_) => {}
            Err(e) => println!("Error setting channel0 duty: {:?}", e),
        }

        let r = channel1.set_duty(duty2 as u8);
        match r {
            Ok(_) => {}
            Err(e) => println!("Error setting channel2 duty: {:?}", e),
        }

        let r = channel2.set_duty(duty3 as u8);
        match r {
            Ok(_) => {}
            Err(e) => println!("Error setting channel4 duty: {:?}", e),
        }

        delay.delay_millis(500u32);
    }
}
