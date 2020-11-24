use crate::util::camera::{webcam::*, device::*};
use v4l::{framesize::FrameSizeEnum, prelude::*};


// USE set_format for v4l2 device
pub struct V4LinuxDevice {
    device_type: WebcamType,
    path: String,
    pub inner: v4l::capture::Device,
}
impl V4LinuxDevice {
     pub fn new(index: usize) -> Result<Self, ()> {
        let device = v4l::capture::Device::new(index);
        Err(())
     }
}
impl Webcam for V4LinuxDevice {
    fn name(&self) -> String {
        todo!()
    }

    fn set_resolution(&mut self, res: Resolution) {
        todo!()
    }

    fn set_framerate(&mut self, fps: i64) {
        todo!()
    }

    fn get_supported_resolutions(&self) -> Vec<Resolution> {
        return match self.inner.enum_framesizes(v4l::FourCC::new(b"YUYV")) {
            Ok(formats) => {
                let mut ret: Vec<Resolution> = Vec::new();
                for fs in formats {
                    let compat = match fs.size {
                        FrameSizeEnum::Stepwise(_step) => {
                            continue;
                        }
                        FrameSizeEnum::Discrete(dis) => {
                            Resolution {
                                x: dis.width,
                                y: dis.height,
                            }
                        }
                    };
                    ret.push(compat);
                }
                ret
            }
            Err(_why) => {
                let ret: Vec<Resolution> = Vec::new();
                ret
            }
        }

    }

    fn get_supported_framerate(&self, res: Resolution) -> Vec<i32> {
        return match self.inner.enum_frameintervals(v4l::FourCC::new(b"YUYV"), res.x, res.y) {
            Ok(inte) => {
                let mut ret: Vec<i32> = Vec::new();
                for frame in inte {
                    
                }
            }
            Err(_why) => {
                let ret: Vec<i32> = Vec::new();
                ret
            }
        }
    }
}