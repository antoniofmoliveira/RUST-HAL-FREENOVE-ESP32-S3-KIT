#![no_std]

use esp_hal::gpio::{Input, Output};

pub struct BlinkLed<
    OutputPin: embedded_hal::digital::OutputPin + embedded_hal::digital::StatefulOutputPin,
    InputPin: embedded_hal::digital::InputPin,
> {
    led: OutputPin,
    button: InputPin,
}

impl<
    OutputPin: embedded_hal::digital::OutputPin + embedded_hal::digital::StatefulOutputPin,
    InputPin: embedded_hal::digital::InputPin,
> BlinkLed<OutputPin, InputPin>
{
    pub fn new(led: OutputPin, button: InputPin) -> Self {
        Self { led, button }
    }

    pub fn handle_button_press(&mut self) -> Result<(), OutputPin::Error> {
        if self.button.is_high().unwrap_or(false) {
            self.led.set_high()?;
        } else {
            self.led.set_low()?;
        }
        Ok(())
    }
}

pub struct BlinkLed2<'a> {
    led: Output<'a>,
    button: Input<'a>,
}

impl<'a> BlinkLed2<'a> {
    pub fn new(led: Output<'a>, button: Input<'a>) -> Self {
        Self { led, button }
    }

    pub fn handle_button_press(&mut self) {
        if self.button.is_high() {
            self.led.set_high();
        } else {
            self.led.set_low();
        }
    }
}
