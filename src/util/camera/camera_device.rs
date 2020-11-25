use crate::util::camera::{device::*, webcam::*};
use usb_enumeration::{enumerate, Filters};
use v4l::{
    capture::parameters::Parameters, format::Format, fraction::Fraction, framesize::FrameSizeEnum,
    prelude::*, FourCC,
};

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
        Ok(V4LinuxDevice {
            device_type,
            device_path,
            inner: device,
        })
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
        Ok(V4LinuxDevice {
            device_type,
            device_path,
            inner: device,
        })
    }
}
impl Webcam for V4LinuxDevice {
    fn name(&self) -> String {
        let device_name = match self.inner.query_caps() {
            Ok(capability) => capability.card,
            Err(_why) => "".to_string(),
        };
        if device_name == "".to_string() {
            let name = String::from(format!("{}", self.device_path));
            name
        } else {
            let name = String::from(format!("{} ({})", self.device_path, device_name));
            name
        }
    }

    fn set_resolution(&mut self, res: Resolution) -> Result<(), Box<dyn std::error::Error>> {
        let fmt = Format::new(res.x, res.y, FourCC::new(b"YUYV"));
        self.inner.set_format(&fmt)?;
        Ok(())
    }

    fn set_framerate(&mut self, fps: u32) -> Result<(), Box<dyn std::error::Error>> {
        let parameter = Parameters::new(Fraction::new(fps, 1));
        self.inner.set_params(&parameter)?;
        Ok(())
    }

    fn get_supported_resolutions(&self) -> Result<Vec<Resolution>, Box<dyn std::error::Error>> {
        return match self.inner.enum_framesizes(v4l::FourCC::new(b"YUYV")) {
            Ok(formats) => {
                let mut ret: Vec<Resolution> = Vec::new();
                for fs in formats {
                    let compat = match fs.size {
                        FrameSizeEnum::Stepwise(_step) => {
                            continue;
                        }
                        FrameSizeEnum::Discrete(dis) => Resolution {
                            x: dis.width,
                            y: dis.height,
                        },
                    };
                    ret.push(compat);
                }
                Ok(ret)
            }
            Err(_why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo(
                    "Supported Resolutions".to_string(),
                ),
            )),
        };
    }

    fn get_supported_framerate(
        &self,
        res: Resolution,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        return match self
            .inner
            .enum_frameintervals(v4l::FourCC::new(b"YUYV"), res.x, res.y)
        {
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
                Ok(ret)
            }
            Err(_why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo(
                    "Supported Framerate".to_string(),
                ),
            )),
        };
    }
}

pub struct UVCameraDevice<'a> {
    device_type: WebcamType,
    device_id: String,
    pub inner: uvc::Device<'a>,
}
impl<'a> UVCameraDevice<'a> {
    pub fn new(
        vendor_id: Option<i32>,
        product_id: Option<i32>,
        serial_number: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let inner = match crate::UVC.find_device(vendor_id, product_id, serial_number.as_deref()) {
            Ok(dev) => dev,
            Err(why) => return Err(Box::new(why)),
        };
        if let Ok(description) = inner.description() {
            let device = enumerate()
                .with_vendor_id(description.vendor_id)
                .with_product_id(description.product_id);
            if let Some(usb_dev) = device.get(0) {
                let device_name = format!(
                    "{}:{} {}",
                    description.vendor_id,
                    description.product_id,
                    usb_dev.description.clone().unwrap_or(String::from(""))
                );
                let device_type = match DeviceHolder::from_devices(usb_dev, &inner) {
                    Ok(dt) => WebcamType::USBVideo(dt),
                    Err(_why) => return Err(Box::new(
                        crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
                    )),
                };
                return Ok(UVCameraDevice {
                    device_type,
                    device_id: device_name,
                    inner,
                });
            }
        }
        Err(Box::new(
            crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
        ))
    }

    pub fn from_device(uvc_dev: uvc::Device<'a>) -> Result<Self, Box<dyn std::error::Error>> {
        let inner = uvc_dev;
        if let Ok(description) = inner.description() {
            let device = enumerate()
                .with_vendor_id(description.vendor_id)
                .with_product_id(description.product_id);
            if let Some(usb_dev) = device.get(0) {
                let device_name = format!(
                    "{}:{} {}",
                    description.vendor_id,
                    description.product_id,
                    usb_dev.description.clone().unwrap_or(String::from(""))
                );
                let device_type = match DeviceHolder::from_devices(usb_dev, &inner) {
                    Ok(dt) => WebcamType::USBVideo(dt),
                    Err(_why) => return Err(Box::new(
                        crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
                    )),
                };
                return Ok(UVCameraDevice {
                    device_type,
                    device_id: device_name,
                    inner,
                });
            }
        }
        Err(Box::new(
            crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
        ))
    }
}

impl<'a> Webcam for UVCameraDevice<'a> {
    fn name(&self) -> String {
        self.device_id.clone()
    }

    fn set_resolution(&mut self, res: Resolution) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!()
    }

    fn set_framerate(&mut self, fps: u32) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!()
    }

    fn get_supported_resolutions(&self) -> Result<Vec<Resolution>, Box<dyn std::error::Error>> {
        match self.inner.open() {
            Ok(handler) => {
                let mut resolutions: Vec<Resolution> = Vec::new();
                for format in handler.supported_formats() {
                    for frame in format.supported_formats() {
                        let resolution_string = Resolution {
                            x: frame.width() as u32,
                            y: frame.height() as u32,
                        };
                        if !resolutions.contains(&resolution_string) {
                            resolutions.push(resolution_string);
                        }
                    }
                }
                Ok(resolutions)
            }
            Err(_why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo(
                    "Supported Resolutions".to_string(),
                ),
            )),
        }
    }

    fn get_supported_framerate(
        &self,
        res: Resolution,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let supported_fps: Vec<u32> = vec![10, 25, 30, 60]; // im too lazy to acutally get the supported frame rates so here are some pretty universal ones
        Ok(supported_fps)
    }
}
