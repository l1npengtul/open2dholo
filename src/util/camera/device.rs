//     Copyright (C) 2020-2021l1npengtul
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::error::invalid_device_error;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::os::raw::c_int;
use usb_enumeration::USBDevice;

#[derive(Clone, Deserialize, Serialize)]
pub struct DeviceDesc {
    pub(crate) vid: Option<c_int>,
    pub(crate) pid: Option<c_int>,
    pub(crate) ser: Option<String>,
}
impl DeviceDesc {
    pub fn new(device: uvc::Device) -> Result<Self, Box<dyn Error>> {
        let device_desc = device.description()?;
        Ok(DeviceDesc {
            vid: Some(c_int::from(device_desc.vendor_id)),
            pid: Some(c_int::from(device_desc.product_id)),
            ser: device_desc.serial_number,
        })
    }
    pub fn from_description(device: uvc::DeviceDescription) -> Self {
        DeviceDesc {
            vid: Some(c_int::from(device.vendor_id)),
            pid: Some(c_int::from(device.product_id)),
            ser: device.serial_number,
        }
    }
    pub fn from_default() -> Self {
        DeviceDesc {
            vid: None,
            pid: None,
            ser: None,
        }
    }
}

#[derive(Clone)]
pub struct DeviceHolder {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub description: String,
    pub serial: Option<String>,
}
impl DeviceHolder {
    pub fn new(
        id: String,
        vendor_id: u16,
        product_id: u16,
        description: String,
        serial: Option<String>,
    ) -> Self {
        DeviceHolder {
            id,
            vendor_id,
            product_id,
            description,
            serial,
        }
    }

    pub fn from_devices(
        usb: &USBDevice,
        uvc: &uvc::Device,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(uvc_desc) = uvc.description() {
            if uvc_desc.vendor_id == usb.vendor_id && uvc_desc.product_id == usb.product_id {
                let mut description: String =
                    String::from(format!("{}:{}", uvc_desc.vendor_id, uvc_desc.product_id));
                let serial = uvc_desc.serial_number.clone();
                if let Some(descript) = usb.description.clone() {
                    description = String::from(format!("{} {}", description, descript));
                }
                let device: DeviceHolder = DeviceHolder::new(
                    usb.id.clone(),
                    uvc_desc.vendor_id,
                    uvc_desc.product_id,
                    description,
                    serial,
                );
                return Ok(device);
            }
        }
        return Err(Box::new(
            invalid_device_error::InvalidDeviceError::CannotFindDevice,
        ));
    }
}

impl PartialEq for DeviceHolder {
    fn eq(&self, other: &Self) -> bool {
        if self.description == other.description
            && self.product_id == other.product_id
            && self.vendor_id == other.vendor_id
            && self.id == other.id
        {
            return false;
        }
        true
    }
}

#[derive(Copy, Clone)]
pub struct Resolution {
    x: i64,
    y: i64,
}

