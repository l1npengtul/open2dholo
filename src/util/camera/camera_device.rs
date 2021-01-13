//     Copyright (C) 2020-2021l1npengtul
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::util::camera::{device_utils::*, webcam::*};
use std::ops::Deref;
use std::{
    any::Any,
    cell::{Cell, RefCell},
};
use usb_enumeration::{enumerate, Filters};
use uvc::{DeviceHandle, FormatDescriptor, FrameFormat};
use v4l::{
    capture::parameters::Parameters, format::Format, fraction::Fraction, framesize::FrameSizeEnum,
    prelude::*, FourCC,
};

// USE set_format for v4l2 device
pub struct V4LinuxDevice {
    device_type: WebcamType,
    device_format: Cell<DeviceFormat>,
    device_path: PathIndex,
    pub inner: RefCell<v4l::capture::Device>,
}
impl V4LinuxDevice {
    pub fn new(index: &usize) -> Result<Self, ()> {
        let device = match v4l::capture::Device::new(index.clone()) {
            Ok(dev) => dev,
            Err(_why) => {
                return Err(());
            }
        };
        let device_type = WebcamType::V4linux2;
        let device_path = PathIndex::Index(index.clone());
        Ok(V4LinuxDevice {
            device_type,
            device_format: Cell::new(DeviceFormat::MJPEG),
            device_path,
            inner: RefCell::new(device),
        })
    }
    pub fn new_path(path: &String) -> Result<Self, ()> {
        let device = match v4l::capture::Device::with_path(path.clone()) {
            Ok(dev) => dev,
            Err(_why) => {
                return Err(());
            }
        };
        let device_type = WebcamType::V4linux2;
        let device_path = PathIndex::Path(path.clone());
        Ok(V4LinuxDevice {
            device_type,
            device_format: Cell::new(DeviceFormat::MJPEG),
            device_path,
            inner: RefCell::new(device),
        })
    }

    pub fn new_location(location: &PathIndex) -> Result<Self, ()> {
        return match location {
            PathIndex::Path(p) => V4LinuxDevice::new_path(p),
            PathIndex::Index(i) => V4LinuxDevice::new(i),
        };
    }
}
impl Webcam for V4LinuxDevice {
    fn name(&self) -> String {
        let device_name = match self.inner.borrow().query_caps() {
            Ok(capability) => capability.card,
            Err(_why) => "".to_string(),
        };
        device_name
    }

    fn set_resolution(&self, res: &Resolution) -> Result<(), Box<dyn std::error::Error>> {
        let mut v4l2_format = FourCC::new(b"YUYV");
        match self.device_format.get() {
            DeviceFormat::YUYV => {}
            DeviceFormat::MJPEG => {
                v4l2_format = FourCC::new(b"MJPG");
            }
        }
        let fmt = Format::new(res.x, res.y, v4l2_format);
        self.inner.borrow_mut().set_format(&fmt)?;
        Ok(())
    }

    fn set_framerate(&self, fps: &u32) -> Result<(), Box<dyn std::error::Error>> {
        let parameter = Parameters::new(Fraction::new(*fps, 1));
        self.inner.borrow_mut().set_params(&parameter)?;
        Ok(())
    }

    fn get_supported_resolutions(&self) -> Result<Vec<Resolution>, Box<dyn std::error::Error>> {
        let mut v4l2_format = FourCC::new(b"YUYV");
        match self.device_format.get() {
            DeviceFormat::YUYV => {}
            DeviceFormat::MJPEG => {
                v4l2_format = FourCC::new(b"MJPG");
            }
        }
        match self.inner.borrow().enum_framesizes(v4l2_format) {
            Ok(formats) => {
                let mut ret: Vec<Resolution> = Vec::new();
                for fs in formats {
                    let compat = match fs.size {
                        FrameSizeEnum::Stepwise(step) => Resolution {
                            x: step.min_width,
                            y: step.min_height,
                        },
                        FrameSizeEnum::Discrete(dis) => Resolution {
                            x: dis.width,
                            y: dis.height,
                        },
                    };
                    ret.push(compat);
                }
                return Ok(ret);
            }
            Err(why) => {
                return Err(Box::new(
                    crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                        prop: "Supported Resolutions".to_string(),
                        msg: why.to_string(),
                    },
                ))
            }
        };
    }

    fn get_supported_framerate(
        &self,
        res: Resolution,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let mut v4l2_format = FourCC::new(b"YUYV");
        match self.device_format.get() {
            DeviceFormat::YUYV => {}
            DeviceFormat::MJPEG => {
                v4l2_format = FourCC::new(b"MJPG");
            }
        }
        return match self
            .inner
            .borrow()
            .enum_frameintervals(v4l2_format, res.x, res.y)
        {
            Ok(inte) => {
                let mut re_t: Vec<u32> = Vec::new();
                for frame in inte {
                    match frame.interval {
                        v4l::frameinterval::FrameIntervalEnum::Discrete(dis) => {
                            re_t.push(dis.denominator);
                        }
                        v4l::frameinterval::FrameIntervalEnum::Stepwise(step) => {
                            re_t.push(step.min.denominator);
                            re_t.push(step.max.denominator);
                        }
                    }
                }
                Ok(re_t)
            }
            Err(why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                    prop: "Supported Framerates".to_string(),
                    msg: why.to_string(),
                },
            )),
        };
    }
    fn get_supported_formats(
        &self,
        _res: Resolution,
    ) -> Result<Vec<DeviceFormat>, Box<dyn std::error::Error>> {
        return match self.inner.borrow().enum_formats() {
            Ok(desc) => {
                let mut dev_format_list: Vec<DeviceFormat> = Vec::new();
                for fmt in desc {
                    // see if the format is either YUYV for MJPG
                    match &fmt.fourcc.to_string().to_ascii_lowercase().to_owned()[..] {
                        "yuyv" => {
                            dev_format_list.push(DeviceFormat::YUYV);
                        }
                        "mjpg" | "mjpeg" => {
                            dev_format_list.push(DeviceFormat::MJPEG);
                        }
                        _ => {
                            // do nothing
                        }
                    }
                }
                Ok(dev_format_list)
            }
            Err(why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                    prop: "Supported Format (FourCC)".to_string(),
                    msg: why.to_string(),
                },
            )),
        };
    }
    fn get_camera_format(&self) -> DeviceFormat {
        return self.device_format.get();
    }

    fn set_camera_format(&self, format: DeviceFormat) {
        self.device_format.set(format);
    }

    fn get_camera_type(&self) -> WebcamType {
        self.device_type
    }

    fn open_stream(&self) -> Result<StreamType, Box<dyn std::error::Error>> {
        return match MmapStream::with_buffers(&*self.inner.borrow_mut(), 4) {
            Ok(stream) => Ok(StreamType::V4L2Stream(stream)),
            Err(why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotOpenStream(
                    why.to_string(),
                ),
            )),
        };
    }

    fn get_inner(&self) -> PossibleDevice {
        let current_format = match self.inner.borrow().format() {
            Ok(format) => format,
            Err(_) => {
                Format::new(640, 480, FourCC::new(b"MJPG")) // TODO: proper error handling
            }
        };

        let res = Resolution {
            x: current_format.width,
            y: current_format.height,
        };

        let fps = match self.inner.borrow().params() {
            Ok(param) => param.interval.denominator as u32,
            Err(_) => 5,
        };

        let fmt = current_format.fourcc;

        PossibleDevice::V4L2 {
            location: self.device_path.clone(),
            res,
            fps,
            fmt,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct UVCameraDevice {
    device_type: WebcamType,
    device_id: String,
    device_format: Cell<DeviceFormat>,
    device_resolution: Cell<Option<Resolution>>,
    device_framerate: Cell<Option<u32>>,
    pub inner: uvc::Device<'static>,
}
impl UVCameraDevice {
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
                    Ok(_dt) => WebcamType::USBVideo,
                    Err(_why) => return Err(Box::new(
                        crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
                    )),
                };
                return Ok(UVCameraDevice {
                    device_type,
                    device_id: device_name,
                    device_format: Cell::new(DeviceFormat::MJPEG),
                    device_resolution: Cell::new(None),
                    device_framerate: Cell::new(None),
                    inner,
                });
            }
        }
        Err(Box::new(
            crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
        ))
    }

    pub fn from_device(uvc_dev: uvc::Device<'static>) -> Result<Self, Box<dyn std::error::Error>> {
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
                    Ok(_dt) => WebcamType::USBVideo,
                    Err(_why) => return Err(Box::new(
                        crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
                    )),
                };
                return Ok(UVCameraDevice {
                    device_type,
                    device_id: device_name,
                    device_format: Cell::new(DeviceFormat::YUYV),
                    device_resolution: Cell::new(None),
                    device_framerate: Cell::new(None),
                    inner,
                });
            }
        }
        Err(Box::new(
            crate::error::invalid_device_error::InvalidDeviceError::CannotFindDevice,
        ))
    }
}

unsafe impl Send for V4LinuxDevice {}
unsafe impl Sync for V4LinuxDevice {} // NEVER MUTATE BETWEEN THREADS!!! NEVER SEND A MUTABLE `V4LinuxDevice`!!!

impl Webcam for UVCameraDevice {
    fn name(&self) -> String {
        self.device_id.clone()
    }

    fn set_resolution(&self, res: &Resolution) -> Result<(), Box<dyn std::error::Error>> {
        self.device_resolution.set(Some(*res));
        Ok(())
    }

    fn set_framerate(&self, fps: &u32) -> Result<(), Box<dyn std::error::Error>> {
        self.device_framerate.set(Some(*fps));
        Ok(())
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
            Err(why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                    prop: "Supported Resolutions".to_string(),
                    msg: why.to_string(),
                },
            )),
        }
    }

    fn get_supported_framerate(
        &self,
        _res: Resolution,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        match self.inner.open() {
            Ok(handler) => {
                let formats: Vec<FormatDescriptor> =
                    handler.supported_formats().into_iter().map(|f| f).collect();
                let mut framerates: Vec<u32> = Vec::new();
                for fmt_desc in formats {
                    for frame_desc in fmt_desc.supported_formats() {
                        let mut fps_from_arr: Vec<u32> = frame_desc
                            .intervals_duration()
                            .into_iter()
                            .map(|duration| (1000 / duration.as_millis()) as u32)
                            .collect();
                        framerates.append(&mut fps_from_arr);
                    }
                }
                Ok(framerates)
            }
            Err(why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                    prop: "Supported Resolutions".to_string(),
                    msg: why.to_string(),
                },
            )),
        }
    }

    fn get_supported_formats(
        &self,
        res: Resolution,
    ) -> Result<Vec<DeviceFormat>, Box<dyn std::error::Error>> {
        match self.inner.open() {
            Ok(handler) => {
                let mut framerates: Vec<DeviceFormat> = Vec::new();
                if let Ok(rates) = self.get_supported_framerate(res) {
                    if let Some(fps) = rates.get(0) {
                        if let Ok(_stream) = handler.get_stream_handle_with_format_size_and_fps(
                            uvc::FrameFormat::YUYV,
                            res.x,
                            res.y,
                            fps.clone(),
                        ) {
                            framerates.push(DeviceFormat::YUYV);
                        }
                        if let Ok(_stream) = handler.get_stream_handle_with_format_size_and_fps(
                            uvc::FrameFormat::MJPEG,
                            res.x,
                            res.y,
                            fps.clone(),
                        ) {
                            framerates.push(DeviceFormat::MJPEG);
                        }
                    } else {
                        return Err(Box::new(
                            crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                                prop: "Supported Resolutions".to_string(),
                                msg: "Could not get supported Framerate for UVC device!".to_string(),
                            },
                        ));
                    }
                } else {
                    return Err(Box::new(
                        crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                            prop: "Supported Resolutions".to_string(),
                            msg: "Could not get supported Framerate for UVC device!".to_string(),
                        },
                    ));
                }

                Ok(framerates)
            }
            Err(why) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotGetDeviceInfo {
                    prop: "Supported Resolutions".to_string(),
                    msg: why.to_string(),
                },
            )),
        }
    }

    fn get_camera_format(&self) -> DeviceFormat {
        return self.device_format.get();
    }

    fn set_camera_format(&self, format: DeviceFormat) {
        self.device_format.set(format);
    }

    fn get_camera_type(&self) -> WebcamType {
        self.device_type
    }

    fn open_stream(&'static self) -> Result<StreamType, Box<dyn std::error::Error>> {
        return match (self.device_resolution.get(), self.device_framerate.get()) {
            (Some(res), Some(fps)) => {
                let format = match self.device_format.get() {
                    DeviceFormat::YUYV => FrameFormat::YUYV,
                    DeviceFormat::MJPEG => FrameFormat::MJPEG,
                };
                return match self.inner.open() {
                    Ok(handler) => Ok(StreamType::UVCStream(handler)),
                    Err(why) => return Err(Box::new(
                        crate::error::invalid_device_error::InvalidDeviceError::CannotOpenStream(
                            why.to_string(),
                        ),
                    )),
                };
            }
            (Some(_), None) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotOpenStream(
                    "Missing required arguments Framerate".to_string(),
                ),
            )),
            (None, Some(_)) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotOpenStream(
                    "Missing required arguments Resolution".to_string(),
                ),
            )),
            (None, None) => Err(Box::new(
                crate::error::invalid_device_error::InvalidDeviceError::CannotOpenStream(
                    "Missing required arguments Resolution, Framerate".to_string(),
                ),
            )),
        };
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_inner(&self) -> PossibleDevice {
        let (vendor_id, product_id, serial) = match self.inner.description() {
            Ok(desc) => (
                Some(desc.vendor_id),
                Some(desc.product_id),
                desc.serial_number,
            ),
            Err(_) => (None, None, None),
        };

        let res = match self.device_resolution.get() {
            Some(r) => r,
            None => Resolution { x: 640, y: 480 },
        };

        let fps = match self.device_framerate.get() {
            Some(f) => f,
            None => 5,
        };

        let fmt = match self.device_format.get() {
            DeviceFormat::YUYV => FrameFormat::YUYV,
            DeviceFormat::MJPEG => FrameFormat::MJPEG,
        };

        PossibleDevice::UVCAM {
            vendor_id,
            product_id,
            serial,
            res,
            fps,
            fmt,
        }
    }
}

unsafe impl<'a> Send for UVCameraDevice {}
unsafe impl<'a> Sync for UVCameraDevice {} // NEVER MUTATE BETWEEN THREADS!!! NEVER SEND A MUTABLE `UVCameraDevice`!!!
