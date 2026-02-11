use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_svc::hal::gpio::{Gpio25, Gpio26, Output, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use log::*;

const DOOR_SERVICE_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb30");
const DOOR_COMMAND_CHAR_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb31");
const NVS_FIRST_PAIR_MAC_ID: &str = "fp-mac-id";

fn get_nvs() -> Result<EspNvs<NvsDefault>, EspError> {
    let nvs_default_partition: EspNvsPartition<NvsDefault> = EspNvsPartition::<NvsDefault>::take()?;
    EspNvs::new(nvs_default_partition, "storage", true)
}

fn get_past_pair_id() -> Result<Option<String>, EspError> {
    let nvs = get_nvs()?;
    let mut buffer = [0u8; 64];
    if let Some(mac_id) = nvs.get_str(NVS_FIRST_PAIR_MAC_ID, &mut buffer)? {
        return Ok(Some(mac_id.to_string()));
    }
    Ok(None)
}

fn save_pair_id(id: String) -> Result<(), EspError> {
    let mut nvs = get_nvs()?;
    nvs.set_str(NVS_FIRST_PAIR_MAC_ID, &id)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Initialize BLE and BLE server
    let ble_device = BLEDevice::take();
    let server = ble_device.get_server();
    let ble_advertising = ble_device.get_advertising();

    // Configure L298N driver pins
    let peripherals = Peripherals::take()?;
    let mut ena = PinDriver::output(peripherals.pins.gpio27)?;
    let mut in1 = PinDriver::output(peripherals.pins.gpio26)?;
    let mut in2 = PinDriver::output(peripherals.pins.gpio25)?;

    // Enable motor driver
    ena.set_high()?;

    server.on_connect(|server, desc| {
        info!("Client connected: {:?}", desc);
        if let Ok(Some(first_pair_mac_id)) = get_past_pair_id() {
            if first_pair_mac_id == desc.address().to_string() {
                if let Err(err) = server.disconnect_with_reason(desc.conn_handle(), 1) {
                    error!("Failed to kick off client, error: {err:?}")
                }
            }
        } else {
            match save_pair_id(desc.address().to_string()) {
                Ok(()) => {
                    info!("Saved new client {}", desc.address().to_string());
                }
                Err(err) => {
                    error!("Failed to save new client, error: {err:?}")
                }
            }
        }
        ble_advertising.lock().stop().ok();
    });

    server.on_disconnect(|desc, reason| {
        info!("Client disconnected: {:?}, reason: {:?}", desc, reason);
        ble_advertising.lock().start().ok();
    });

    let door_service = server.create_service(DOOR_SERVICE_UUID);

    let door_command_char = door_service.lock().create_characteristic(
        DOOR_COMMAND_CHAR_UUID,
        NimbleProperties::WRITE | NimbleProperties::READ,
    );

    let (tx, rx) = mpsc::channel::<()>();

    thread::spawn(move || {
        while let Ok(_) = rx.recv() {
            match open_door(&mut in1, &mut in2) {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to open door, {err:?}");
                }
            }
        }
    });

    // Set up callback for when data is written to the characteristic
    door_command_char.lock().on_write(move |args| {
        let data = args.recv_data();
        info!("Received data: {:?}", data);

        if data == b"open" {
            match tx.send(()) {
                Ok(_) => {}
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

    // Run loop
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn extend(
    in1: &mut PinDriver<Gpio26, Output>,
    in2: &mut PinDriver<Gpio25, Output>,
) -> anyhow::Result<()> {
    in1.set_high()?;
    in2.set_low()?;
    Ok(())
}

fn retract(
    in1: &mut PinDriver<Gpio26, Output>,
    in2: &mut PinDriver<Gpio25, Output>,
) -> anyhow::Result<()> {
    in1.set_low()?;
    in2.set_high()?;
    Ok(())
}

fn open_door(
    in1: &mut PinDriver<Gpio26, Output>,
    in2: &mut PinDriver<Gpio25, Output>,
) -> anyhow::Result<()> {
    println!("Opening door...");
    extend(in1, in2)?;
    thread::sleep(Duration::from_millis(2000));
    retract(in1, in2)?;
    Ok(())
}
