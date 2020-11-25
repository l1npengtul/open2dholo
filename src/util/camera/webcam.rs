use crate::util::camera::device::{DeviceHolder, Resolution};

pub trait Webcam {
    fn name(&self) -> String;
    fn set_resolution(&mut self, res: Resolution) -> Result<(), Box<dyn std::error::Error>>;
    fn set_framerate(&mut self, fps: u32) -> Result<(), Box<dyn std::error::Error>>;
    fn get_supported_resolutions(&self) -> Result<Vec<Resolution>, Box<dyn std::error::Error>>;
    fn get_supported_framerate(
        &self,
        res: Resolution,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>>;
}

pub enum WebcamType {
    V4linux2(String),
    USBVideo(DeviceHolder),
}
