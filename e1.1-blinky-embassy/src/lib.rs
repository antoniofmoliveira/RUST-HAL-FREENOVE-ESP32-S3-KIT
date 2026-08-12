#![no_std]

use esp_hal::gpio::Output;
use embassy_time::{ Timer};

pub struct BlinkLed<'a> {
    output: Output<'a>,
}

impl<'a> BlinkLed<'a> {
    pub fn new(output: Output<'a>) -> Self {
        Self { output }
    }
    pub async fn toggle(&mut self) {
        self.output.toggle();
        Timer::after_millis(500).await;
    }
}
