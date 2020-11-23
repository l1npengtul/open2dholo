use crate::util::camera::device::DeviceHolder;

pub trait Webcam {
    fn name(&self) -> String;
}

pub enum WebcamType {
    V4linux2(String),
    USBVideo(DeviceHolder),
}
