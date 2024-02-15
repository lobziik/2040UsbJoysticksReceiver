#![no_std]

use serde::ser::{Serialize, SerializeTuple, Serializer};
use usbd_hid::descriptor::AsInputReport;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid_macros::gen_hid_descriptor;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (usage_page = GENERIC_DESKTOP,) = {
            (usage = X,) = {
                # [item_settings data, variable, relative] x = input;
            };
            (usage = Y,) = {
                # [item_settings data, variable, relative] y = input;
            };
        };
        (usage_page = BUTTON, collection = PHYSICAL, usage_min = 0x01, usage_max = 0x0C) = {
            #[item_settings data,array,absolute] buttons=input;
        };
    }
)]
#[allow(dead_code)]
pub struct JoystickReport {
    pub x: i8,
    pub y: i8,
    pub buttons: u8,
}
