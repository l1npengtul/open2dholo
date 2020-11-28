use crate::util::camera::device_utils::*;

pub trait Webcam {
    fn name(&self) -> String;
    fn set_resolution(&mut self, res: Resolution) -> Result<(), Box<dyn std::error::Error>>;
    fn set_framerate(&mut self, fps: u32) -> Result<(), Box<dyn std::error::Error>>;
    fn get_supported_resolutions(&self) -> Result<Vec<Resolution>, Box<dyn std::error::Error>>;
    fn get_supported_formats(
        &self,
        res: Resolution,
    ) -> Result<Vec<DeviceFormat>, Box<dyn std::error::Error>>;
    fn get_supported_framerate(
        &self,
        res: Resolution,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>>;
    fn get_camera_format(&self) -> DeviceFormat;
    fn set_camera_foramt(&self, format: DeviceFormat);
    fn get_camera_type(&self) -> WebcamType;
    fn open_stream(&mut self) -> Result<StreamType, Box<dyn std::error::Error>>;
}
#[derive(Copy, Clone)]
pub enum WebcamType {
    V4linux2,
    USBVideo,
}
