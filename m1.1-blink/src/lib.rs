#![no_std]

use esp_hal::gpio::Output;

pub struct BlinkLed<
    OutputPin: embedded_hal::digital::OutputPin + embedded_hal::digital::StatefulOutputPin,
> {
    pin: OutputPin,
}

impl<OutputPin: embedded_hal::digital::OutputPin + embedded_hal::digital::StatefulOutputPin>
    BlinkLed<OutputPin>
{
    pub fn new(pin: OutputPin) -> Self {
        Self { pin }
    }

    pub fn toggle(&mut self) -> Result<(), OutputPin::Error> {
        let r = self.pin.toggle();
        match r {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
// ======================

pub struct BlinkLed2<'a> {
    output: Output<'a>,
}

impl<'a> BlinkLed2<'a> {
    pub fn new(output: Output<'a>) -> Self {
        Self { output }
    }
    pub fn toggle(&mut self) {
        self.output.toggle();
    }
}
