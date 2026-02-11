use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::gpio::{Gpio25, Gpio26, Gpio27, Output, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::sys::EspError;

pub struct L298N {
    ena: PinDriver<'static, Gpio27, Output>,
    in1: PinDriver<'static, Gpio26, Output>,
    in2: PinDriver<'static, Gpio25, Output>,
}

impl L298N {
    pub fn new(peripherals: Peripherals) -> Result<Self, EspError> {
        Ok(Self {
            ena: PinDriver::output(peripherals.pins.gpio27)?,
            in1: PinDriver::output(peripherals.pins.gpio26)?,
            in2: PinDriver::output(peripherals.pins.gpio25)?,
        })
    }

    pub fn enable_motor(&mut self) -> Result<(), EspError> {
        self.ena.set_high()?;
        Ok(())
    }

    pub fn open_door(&mut self) -> Result<(), EspError> {
        self.extend()?;
        thread::sleep(Duration::from_millis(2000));
        self.retract()?;
        Ok(())
    }

    fn extend(&mut self) -> Result<(), EspError> {
        self.in1.set_high()?;
        self.in2.set_low()?;
        Ok(())
    }

    fn retract(&mut self) -> Result<(), EspError> {
        self.in1.set_low()?;
        self.in2.set_high()?;
        Ok(())
    }
}
