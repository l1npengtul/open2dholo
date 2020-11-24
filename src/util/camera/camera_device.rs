use crate::util::camera::{webcam::*, device::*};
use v4l::{format::Format, FourCC, framesize::FrameSizeEnum, prelude::*, capture::parameters::Parameters, fraction::Fraction};
use usb_enumeration::{enumerate, Filters};

// USE set_format for v4l2 device
pub struct V4LinuxDevice {
    device_type: WebcamType,
    device_path: String,
    pub inner: v4l::capture::Device,
}
impl V4LinuxDevice {
     pub fn new(index: usize) -> Result<Self, ()> {
        let device = match v4l::capture::Device::new(index.clone()) {
            Ok(dev) => dev,
            Err(_why) => {
                return Err(());
            }
        };
        let device_type = WebcamType::V4linux2(index.to_string());
        let device_path = String::from(format!("/dev/video{}", index.to_string()));
        Ok(
            V4LinuxDevice {
                device_type,
                device_path, 
                inner: device,
            }
        )
     }
     pub fn new_path(path: String) -> Result<Self, ()> {
         let device = match v4l::capture::Device::with_path(path.clone()) {
             Ok(dev) => dev,
             Err(_why) => {
                 return Err(());
             }
         };
         let device_type = WebcamType::V4linux2(path.clone());
         let device_path = path;
         Ok(
             V4LinuxDevice {
                 device_type,
                 device_path,
                 inner: device,
             }
         )
     }
}
impl Webcam for V4LinuxDevice {
    fn name(&self) -> String {
        let device_name = match self.inner.query_caps() {
            Ok(capability) => {
                capability.card
            }
            Err(_why) => {
                "".to_string()
            }
        };
        if device_name == "".to_string() {
            let name = String::from(format!("{}", self.device_path));
            name
        }
        else {
            let name = String::from(format!("{} ({})", self.device_path, device_name));
            name
        }
    }

    fn set_resolution(&mut self, res: Resolution) {
        let fmt = Format::new(res.x, res.y, FourCC::new(b"YUYV"));
        self.inner.set_format(&fmt);
    }

    fn set_framerate(&mut self, fps: u32) {
        let parameter = Parameters::new(Fraction::new(fps, 1));
        self.inner.set_params(&parameter);
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

    fn get_supported_framerate(&self, res: Resolution) -> Vec<u32> {
        return match self.inner.enum_frameintervals(v4l::FourCC::new(b"YUYV"), res.x, res.y) {
            Ok(inte) => {
                let mut ret: Vec<u32> = Vec::new();
                for frame in inte {
                    match frame.interval {
                        v4l::frameinterval::FrameIntervalEnum::Discrete(dis) => {
                            if dis.numerator % dis.denominator == 0 {
                                ret.push(dis.numerator);
                            }
                        }
                        v4l::frameinterval::FrameIntervalEnum::Stepwise(_) => {}
                    }
                }
                ret
            }
            Err(_why) => {
                let ret: Vec<u32> = Vec::new();
                ret
            }
        }
    }
}


pub struct UVCameraDevice<'a> {
    device_type: WebcamType,
    device_id: String,
    pub inner: uvc::Device<'a>,
}
impl<'a> UVCameraDevice<'a> {
    pub fn new(vendor_id: Option<i32>, product_id: Option<i32>, serial_number: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let inner = match crate::UVC.find_device(vendor_id, product_id, serial_number.as_deref()) {
            Ok(dev) => dev,
            Err(why) => return Err(Box::new(why))
        };
        let device_name = format!("{}:{}", vendor_id.unwrap_or(0), product_id.unwrap_or(0));
        if let Ok(description) = inner.description() {
            let device = enumerate().with_vendor_id(description.vendor_id).with_product_id(description.product_id);
            if let Some(usb_dev) = device.get(0) {
                device_name = format!("{}:{} {}", description.vendor_id, description.product_id, usb_dev.description.unwrap_or(String::from("")));
            }
        }
        let device_type = WebcamType::USBVideo(
            
        );
        Err(())
    }
}