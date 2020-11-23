use crate::util::camera::webcam::*;
use v4l::capture::device::Device;


// USE set_format for v4l2 device
pub struct V4LinuxDevice {
    device_type: WebcamType,
    path: String,
    pub inner: Box<v4l::capture::Device>,
}
impl V4LinuxDevice {
     pub fn new(index: usize) -> Result<Self, ()> {
        let device = Device::new(index);
        Err(())
     }
}
impl Webcam for V4LinuxDevice {
    fn name(&self) -> String {
        todo!()
    }
}