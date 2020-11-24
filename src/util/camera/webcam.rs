use crate::util::camera::device::{DeviceHolder, Resolution};

pub trait Webcam {
    fn name(&self) -> String;
    fn set_resolution(&mut self, res: Resolution);
    fn set_framerate(&mut self, fps: i64);
    fn get_supported_resolutions(&self) -> Vec<Resolution>;
    fn get_supported_framerate(&self, res: Resolution) -> Vec<i32>;
}

pub enum WebcamType {
    V4linux2(String),
    USBVideo(DeviceHolder),
}
