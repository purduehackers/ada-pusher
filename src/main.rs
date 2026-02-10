use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use log::*;

const DOOR_SERVICE_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb30");
const DOOR_COMMAND_CHAR_UUID: BleUuid = uuid128!("7e783540-f3ab-431f-adff-566767b8bb31");

fn get_nvs() -> Result<EspNvs<NvsDefault>, EspError> {
    let nvs_default_partition: EspNvsPartition<NvsDefault> = EspNvsPartition::<NvsDefault>::take()?;
    EspNvs::new(nvs_default_partition, "storage", true)
}

fn get_past_pair_id() -> Result<Option<String>, EspError> {
    let nvs = get_nvs()?;
    let mut buffer = [0u8; 64];
    if let Some(mac_id) = nvs.get_str("first-pair-mac-id", &mut buffer)? {
        println!("This ada-pusher was paired before to MAC ID: {}", mac_id);
        return Ok(Some(mac_id.to_string()));
    }
    Ok(None)
}

fn save_pair_id(id: String) -> Result<(), EspError> {
    let mut nvs = get_nvs()?;
    nvs.set_str("first-pair-mac-id", &id)?;
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
