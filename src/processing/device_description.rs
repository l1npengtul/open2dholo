use std::os::raw::c_int;

#[derive(Clone)]
pub struct DeviceDesc{
    pub(crate) vid: Option<c_int>,
    pub(crate) pid: Option<c_int>,
    pub(crate) ser: Option<String>,
}
impl DeviceDesc {
    pub fn new(device: uvc::Device) -> Result<Self, ()> {
        let device_desc = device.description()?;
        Ok(DeviceDesc{
            vid: Some(c_int::from(device_desc.vendor_id)),
            pid: Some(c_int::from(device_desc.product_id)),
            ser: device_desc.serial_number,
        })
    }
    pub fn from_description(device: uvc::DeviceDescription) -> Self {
        DeviceDesc{
            vid: Some(c_int::from(device_desc.vendor_id)),
            pid: Some(c_int::from(device_desc.product_id)),
            ser: device_desc.serial_number,
        }
    }
}