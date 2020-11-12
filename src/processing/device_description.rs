use std::error::Error;
use std::os::raw::c_int;
use serde::{Serialize, Deserialize};

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
            ser: None
        }
    }
}
