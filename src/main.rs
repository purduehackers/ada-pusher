use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use log::*;

const DOOR_SERVICE_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb30");
const DOOR_COMMAND_CHAR_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb31");

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

    server.on_connect(|server, desc| {
        info!("Client connected: {:?}", desc);
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

    // Set up callback for when data is written to the characteristic
    door_command_char.lock().on_write(|args| {
        let data = args.recv_data();
        info!("Received data: {:?}", data);

        // Check for your specific command (e.g., "OPEN")
        if data == b"OPEN" {
            info!("Opening door!");
            open_door();
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

fn open_door() {
    info!("Would've opened door...");
}
