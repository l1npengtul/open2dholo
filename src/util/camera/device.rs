use crate::util::camera::webcam::*;
use v4l::capture::device::Device;

pub struct V4LinuxDevice {
    device_type: WebcamType,
    path: String,
    pub inner: v4l::device::Device,
}
impl V4LinuxDevice {
     pub fn new(index: usize) -> Result<Self, Box<dyn std:::error::Error>> {
        let device = Device::new(index)
     }
}
impl Webcam for V4LinuxDevice {
    fn name(&self) -> String {
        todo!()
    }
}