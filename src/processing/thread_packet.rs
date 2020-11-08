use crate::processing::device_description::DeviceDesc;

// TODO: Change to acutal data format
#[derive(Clone)]
pub enum MessageType {
    DIE(u8),
    SET(DeviceDesc),
    CLOSE(u8),
}
