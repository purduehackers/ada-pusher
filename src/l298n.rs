use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver};
use esp_idf_svc::sys::EspError;

pub struct L298N<ENA, IN1, IN2>
where
    ENA: OutputPin,
    IN1: OutputPin,
    IN2: OutputPin,
{
    ena: PinDriver<'static, ENA, Output>,
    in1: PinDriver<'static, IN1, Output>,
    in2: PinDriver<'static, IN2, Output>,
}

impl<ENA, IN1, IN2> L298N<ENA, IN1, IN2>
where
    ENA: OutputPin,
    IN1: OutputPin,
    IN2: OutputPin,
{
    pub fn new(
        ena: PinDriver<'static, ENA, Output>,
        in1: PinDriver<'static, IN1, Output>,
        in2: PinDriver<'static, IN2, Output>,
    ) -> Result<Self, EspError> {
        Ok(Self { ena, in1, in2 })
    }

    pub fn open_door(&mut self) -> Result<(), EspError> {
        self.forward()?;
        self.enable_motor()?;
        thread::sleep(Duration::from_millis(2000));
        self.backward()?;
        thread::sleep(Duration::from_millis(2000));
        self.disable_motor()?;
        Ok(())
    }

    fn enable_motor(&mut self) -> Result<(), EspError> {
        self.ena.set_high()?;
        Ok(())
    }

    fn disable_motor(&mut self) -> Result<(), EspError> {
        self.ena.set_low()?;
        Ok(())
    }

    fn forward(&mut self) -> Result<(), EspError> {
        self.in1.set_high()?;
        self.in2.set_low()?;
        Ok(())
    }

    fn backward(&mut self) -> Result<(), EspError> {
        self.in1.set_low()?;
        self.in2.set_high()?;
        Ok(())
    }
}
