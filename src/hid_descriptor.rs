use serde::ser::{Serialize, SerializeTuple, Serializer};
use usbd_hid::descriptor::AsInputReport;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid_macros::gen_hid_descriptor;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (collection = APPLICATION, usage = POINTER) = {
            (usage = X,) = {
                #[item_settings data, variable, absolute] x = input;
            };
            (usage = Y,) = {
                #[item_settings data, variable, absolute] y = input;
            };
        };
        (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = 0x0C, logical_min = 0) = {
            #[packed_bits 16] #[item_settings data, variable, absolute] buttons=input;
        };
    }
)]
#[allow(dead_code)]
pub struct JoystickReport {
    pub x: i8,
    pub y: i8,
    pub buttons: [u8; 2],
}

impl JoystickReport {
    pub fn set_zero(&mut self) {
        self.x = 0;
        self.y = 0;
        self.buttons = [0, 0];
    }
}
