use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidDeviceError {
    #[error("Device with description vendor id: {vendor}, product id: {prod}, serial number: {ser}, ERROR could not open/get device! Make sure it exists!")]
    InvalidDescription {
        vendor: String,
        prod: String,
        ser: String,
    },
}
