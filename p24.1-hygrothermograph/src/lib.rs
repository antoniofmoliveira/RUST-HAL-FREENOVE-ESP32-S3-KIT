#![no_std]
use esp_hal::system::Error;

pub enum PinState {
    High,
    Low,
}

pub struct Dht11<
    PIN: embedded_hal::digital::OutputPin + embedded_hal::digital::InputPin,
    DELAY: embedded_hal::delay::DelayNs,
> {
    pin: PIN,
    delay: DELAY,
}

pub struct SensorReading {
    pub temperature: i8,
    pub humidity: u8,
    pub temperature_decimal: f32,
    pub humidity_decimal: f32,
}

impl<
    PIN: embedded_hal::digital::OutputPin + embedded_hal::digital::InputPin,
    DELAY: embedded_hal::delay::DelayNs,
> Dht11<PIN, DELAY>
{
    pub fn new(pin: PIN, delay: DELAY) -> Self {
        Self { pin, delay }
    }

    pub fn read(&mut self) -> Result<SensorReading, Error> {
        // todo!();

        // Start communication: pull pin low for 18ms, then release.
        let _ = self.pin.set_low();
        self.delay.delay_ms(18);
        let _ = self.pin.set_high();

        // Wait for sensor to respond.
        self.delay.delay_us(48);

        // Sync with sensor: wait for high then low signals.
        let _ = self.wait_until_state(PinState::High);
        let _ = self.wait_until_state(PinState::Low);

        // Start reading 40 bits
        let humidity_integer = self.read_byte();
        let humidity_decimal = self.read_byte();
        let temperature_integer = self.read_byte();
        let temperature_decimal = self.read_byte();

        Ok(SensorReading {
            humidity: humidity_integer.unwrap(),
            temperature: temperature_integer.unwrap() as i8,
            humidity_decimal: humidity_decimal.unwrap() as f32,
            temperature_decimal: temperature_decimal.unwrap() as f32,
        })
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut byte: u8 = 0;
        for n in 0..8 {
            let _ = self.wait_until_state(PinState::High);
            self.delay.delay_us(30);
            let is_bit_1 = self.pin.is_high();
            if is_bit_1.unwrap() {
                let bit_mask = 1 << (7 - (n % 8));
                byte |= bit_mask;
                let _ = self.wait_until_state(PinState::Low);
            }
        }
        Ok(byte)
    }

    fn wait_until_state(&mut self, state: PinState) -> Result<(), Error> {
        loop {
            match state {
                PinState::Low => {
                    if self.pin.is_low().unwrap() {
                        break;
                    }
                }
                PinState::High => {
                    if self.pin.is_high().unwrap() {
                        break;
                    }
                }
            };
            self.delay.delay_us(1);
        }
        Ok(())
    }
}
