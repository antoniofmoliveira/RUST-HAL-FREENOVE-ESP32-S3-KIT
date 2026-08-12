#![no_std]

use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, Output};

pub struct BlinkLed<'a> {
    led: Output<'a>,
    button: Input<'a>,
}

impl<'a> BlinkLed<'a> {
    pub fn new(led: Output<'a>, button: Input<'a>) -> Self {
        Self { led, button }
    }

    pub async fn handle_button_press(&mut self) {
        if self.button.is_high() {
            self.led.set_high();
        } else {
            self.led.set_low();
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
