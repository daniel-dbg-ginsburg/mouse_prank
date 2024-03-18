use std::sync::Arc;

use esp32_nimble::enums::*;
use esp32_nimble::utilities::mutex::Mutex;
use esp32_nimble::{BLEAdvertisementData, BLECharacteristic, BLEDevice, BLEHIDDevice};

use esp_idf_svc as svc;
use svc::hal;

use rand::prelude::*;

use hid_tools::{
    report_builder::ReportDescriptorBuilder,
    report_descriptor::Collection,
    usage_table::{generic_desktop::GenericDesktopControlsUsage, UsagePage},
};

struct Mouse {
    //    server: &'static mut BLEServer,
    input_mouse: Arc<Mutex<BLECharacteristic>>,
    //    hid: BLEHIDDevice,
}

impl Mouse {
    fn new(device: &mut BLEDevice) -> Self {
        let server = device.get_server();
        let adv = device.get_advertising();

        server.on_connect(|_server, desc| {
            log::info!("Client connected {:?}", desc);
        });

        server.on_disconnect(|desc, reason| {
            log::info!("Client {} disconnected: {:?}", desc.address(), reason);
        });

        server.on_authentication_complete(|desc, _res| {
            log::info!("Client authenticated {:?}", desc);
        });

        let mut hid = BLEHIDDevice::new(server);

        hid.manufacturer("anonymouse inc");
        hid.pnp(0x02, 0x5ac, 0x820a, 0x0210);
        hid.hid_info(0x00, 0x01);
        hid.set_battery_level(100);

        let descr = ReportDescriptorBuilder::new()
            .item(UsagePage::GenericDesktopControls)
            .item(GenericDesktopControlsUsage::Mouse)
            .item(Collection::Application)
            .item(GenericDesktopControlsUsage::Pointer)
            .item(Collection::Physical)
            .item(UsagePage::Button)
            .usage_minimum::<u16>(1)
            .usage_maximum::<u16>(3)
            .logical_minimum(0)
            .logical_maximum(1)
            .report_count(3)
            .report_size(1)
            .input(0x02) // Input (Data, Variable, Absolute)
            .report_count(1)
            .report_size(5)
            .input(0x03) // Input (Constant, Variable, Absolute)
            .item(UsagePage::GenericDesktopControls)
            .item(GenericDesktopControlsUsage::X)
            .item(GenericDesktopControlsUsage::Y)
            .item(GenericDesktopControlsUsage::Wheel)
            .logical_minimum(-127)
            .logical_maximum(127)
            .report_size(8)
            .report_count(3)
            .input(0x06) // Input (Data, Variable, Relative)
            .end_collection()
            .end_collection()
            .build()
            .bytes();

        hid.report_map(descr.as_slice());

        let input_mouse = hid.input_report(0);

        adv.lock()
            .scan_response(true)
            .set_data(
                BLEAdvertisementData::new()
                    .name("anonymouse")
                    .appearance(0x03c2) // BLE_APPEARANCE_HID_KEYBOARD
                    .add_service_uuid(hid.hid_service().lock().uuid()),
            )
            .unwrap();

        adv.lock().start().unwrap();

        Self {
            // server,
            input_mouse,
            // hid,
        }
    }

    fn shift(&self, x: i8, y: i8) {
        let report: [i8; 4] = [0, x, y, 0];
        self.input_mouse.lock().set_from(&report).notify();
    }
}
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let device = BLEDevice::take();
    device
        .security()
        .set_auth(AuthReq::all())
        .set_passkey(000000)
        .set_io_cap(SecurityIOCap::NoInputNoOutput)
        .resolve_rpa();

    let mouse = Mouse::new(device);

    let mut rng = thread_rng();

    loop {
        // Randomly jerk the mouse 20 times
        for _ in 0..20 {
            let x = rng.gen_range(-15i8..15i8);
            let y = rng.gen_range(-15i8..15i8);
            mouse.shift(x, y);
            hal::delay::FreeRtos::delay_ms(5);
            mouse.shift(-x, -y);
            hal::delay::FreeRtos::delay_ms(5);
        }

        // Sleep for 2 to 10 seconds
        hal::delay::FreeRtos::delay_ms(1000 * rng.gen_range(2..10))
    }
}
