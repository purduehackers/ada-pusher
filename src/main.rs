mod l298n;

use std::sync::mpsc;
use std::thread;

use dotenvy_macro::dotenv;
use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::sys::EspError;
use log::*;

use l298n::L298N;

const DOOR_SERVICE_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb30");
const DOOR_COMMAND_CHAR_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb31");
const PAIRING_PIN: &str = dotenv!("PAIRING_PIN");

fn led_blink(led: &mut PinDriver<'_, impl OutputPin, Output>) -> Result<(), EspError> {
    // Rapidly flash the LED five times
    for _ in 0..5 {
        led.set_high()?;
        thread::sleep(std::time::Duration::from_millis(100));
        led.set_low()?;
        thread::sleep(std::time::Duration::from_millis(100));
    }
    led.set_high()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Initialize BLE and BLE server
    BLEDevice::set_device_name("ada-pusher").expect("Failed to set device name");
    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();

    // Configure security
    ble_device
        .security()
        .set_auth(AuthReq::all())
        .set_passkey(PAIRING_PIN.parse().expect("Failed to parse pairing pin"))
        .set_io_cap(SecurityIOCap::DisplayOnly)
        .resolve_rpa();

    let server = ble_device.get_server();

    // Configure L298N driver pins
    let peripherals = Peripherals::take()?;
    let mut onboard_led = PinDriver::output(peripherals.pins.gpio2)?;
    let mut l298n = L298N::new(
        PinDriver::output(peripherals.pins.gpio27)?,
        PinDriver::output(peripherals.pins.gpio26)?,
        PinDriver::output(peripherals.pins.gpio25)?,
    )?;

    server.on_connect(|_server, desc| {
        info!("Client connected: {:?}", desc);
    });

    server.on_disconnect(|desc, reason| {
        info!("Client disconnected: {:?}, reason: {:?}", desc, reason);
    });

    let (tx, rx) = mpsc::channel::<()>();

    thread::spawn(move || {
        while rx.recv().is_ok() {
            match l298n.open_door() {
                Ok(_) => {
                    let _ = led_blink(&mut onboard_led);
                    info!("Sent signal to L298N driver to open door!");
                }
                Err(err) => {
                    error!("Failed to open door, {err:?}");
                }
            }
        }
    });

    let door_service = server.create_service(DOOR_SERVICE_UUID);

    let door_command_char = door_service.lock().create_characteristic(
        DOOR_COMMAND_CHAR_UUID,
        NimbleProperties::READ | NimbleProperties::WRITE | NimbleProperties::WRITE_AUTHEN,
    );

    // Set up callback for when data is written to the characteristic
    door_command_char.lock().on_write(move |args| {
        let data = args.recv_data();
        info!("Received data: {:?}", data);

        if data == b"open" {
            match tx.send(()) {
                Ok(_) => {
                    debug!("Sent signal to door opener thread");
                }
                Err(err) => {
                    error!("Failed to notify door opener thread, {err:?}")
                }
            }
        }
    });

    // Configure advertising
    let mut advertising_data = BLEAdvertisementData::new();
    advertising_data.name("ada-pusher");
    advertising_data.add_service_uuid(DOOR_SERVICE_UUID);
    ble_advertising.lock().set_data(&mut advertising_data).ok();

    // Start advertising
    ble_advertising.lock().start()?;
    info!("BLE advertising started, waiting for connections...");
    info!("Currently bonded: {:?}", ble_device.bonded_addresses());
    // Run loop
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
