//     Open2DH - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
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

use crate::error::invalid_device_error::InvalidDeviceError;
use crate::util::camera::device_utils::{get_os_webcam_index, DeviceContact};
use crate::util::camera::{
    device_utils::{DeviceFormat, DeviceHolder, PathIndex, PossibleDevice, Resolution, StreamType},
    webcam::{Webcam, WebcamType},
};
use gdnative::prelude::*;
use opencv::core::{Mat, CV_8UC3};
use opencv::videoio::{VideoCapture, VideoCaptureProperties, VideoCaptureTrait, VideoWriter, CAP_MSMF, CAP_V4L2, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_PROP_FPS, CAP_PROP_FORMAT, CAP_PROP_FOURCC};
use pyo3::prelude::*;
use pyo3::types::*;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use usb_enumeration::{enumerate, Filters};
use uvc::{FormatDescriptor, FrameFormat};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::video::capture::Parameters;
use v4l::{
    format::Format, fraction::Fraction, framesize::FrameSizeEnum, video::traits::Capture, FourCC,
};
use crate::error::invalid_device_error::InvalidDeviceError::{CannotOpenStream, CannotGetDeviceInfo, CannotFindDevice, InvalidPlatform};

// USE set_format for v4l2 device
pub struct V4LinuxDevice {
    device_type: WebcamType,
    device_format: Cell<DeviceFormat>,
    device_path: PathIndex,
    pub inner: RefCell<v4l::Device>,
}

impl V4LinuxDevice {
    pub fn new(index: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let device = match v4l::Device::new(index) {
            Ok(dev) => dev,
            Err(why) => {
                return Err(Box::new(CannotFindDevice(format!(
                    "{}, idx: {}",
                    why.to_string(),
                    index
                ))));
            }
        };
        let device_type = WebcamType::V4linux2;
        let device_path = PathIndex::Index(index);
        Ok(V4LinuxDevice {
            device_type,
            device_format: Cell::new(DeviceFormat::MJPEG),
            device_path,
            inner: RefCell::new(device),
        })
    }
    pub fn new_path(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let device = match v4l::Device::with_path(path.to_string()) {
            Ok(dev) => dev,
            Err(why) => {
                return Err(Box::new(CannotFindDevice(format!(
                    "{}, path: {}",
                    why.to_string(),
                    path
                ))));
            }
        };
        let device_type = WebcamType::V4linux2;
        let device_path = PathIndex::Path(path);
        Ok(V4LinuxDevice {
            device_type,
            device_format: Cell::new(DeviceFormat::MJPEG),
            device_path,
            inner: RefCell::new(device),
        })
    }
    pub fn new_location(location: PathIndex) -> Result<Self, Box<dyn std::error::Error>> {
        match location {
            PathIndex::Path(p) => V4LinuxDevice::new_path(p),
            PathIndex::Index(i) => V4LinuxDevice::new(i.to_owned()),
        }
    }
}

impl<'a> Webcam<'a> for V4LinuxDevice {
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
        return match self.inner.borrow().enum_framesizes(v4l2_format) {
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
                Ok(ret)
            }
            Err(why) => Err(Box::new(
                CannotGetDeviceInfo {
                    prop: "Supported Resolutions".to_string(),
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
                CannotGetDeviceInfo {
                    prop: "Supported Format (FourCC)".to_string(),
                    msg: why.to_string(),
                },
            )),
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
            Ok(interval) => {
                let mut re_t: Vec<u32> = Vec::new();
                for frame in interval {
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
                CannotGetDeviceInfo {
                    prop: "Supported Framerates".to_string(),
                    msg: why.to_string(),
                },
            )),
        };
    }
    fn get_camera_format(&self) -> DeviceFormat {
        self.device_format.get()
    }

    fn set_camera_format(&self, format: DeviceFormat) {
        self.device_format.set(format);
    }

    fn get_camera_type(&self) -> WebcamType {
        self.device_type
    }

    fn open_stream(&self) -> Result<StreamType, Box<dyn std::error::Error>> {
        return match Stream::with_buffers(&*self.inner.borrow_mut(), Type::VideoCapture, 4) {
            Ok(stream) => Ok(StreamType::V4L2Stream(stream)),
            Err(why) => Err(Box::new(
                CannotOpenStream(
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
}

pub struct UVCameraDevice<'a> {
    device_type: WebcamType,
    device_id: String,
    device_format: Cell<DeviceFormat>,
    device_resolution: Cell<Option<Resolution>>,
    device_framerate: Cell<Option<u32>>,
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
                    usb_dev
                        .description
                        .clone()
                        .unwrap_or_else(|| String::from(""))
                );
                let device_type = match DeviceHolder::from_devices(usb_dev, &inner) {
                    Ok(_dt) => WebcamType::USBVideo,
                    Err(why) => return Err(Box::new(
                        CannotFindDevice(
                            format!(
                                "{},{}:{} {}",
                                why.to_string(),
                                description.vendor_id,
                                description.product_id,
                                description.serial_number.unwrap_or_else(|| "".to_string())
                            ),
                        ),
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
            CannotFindDevice(format!(
                "i64-{}:i64-{} {}",
                vendor_id.unwrap_or(-1),
                product_id.unwrap_or(-1),
                serial_number.unwrap_or_else(|| "".to_string())
            )),
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
                    usb_dev
                        .description
                        .clone()
                        .unwrap_or_else(|| String::from(""))
                );
                let device_type = match DeviceHolder::from_devices(usb_dev, &inner) {
                    Ok(_dt) => WebcamType::USBVideo,
                    Err(why) => return Err(Box::new(
                        CannotFindDevice(
                            format!("noaddr: {}", why.to_string()),
                        ),
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
            CannotFindDevice(
                "noaddr".to_string(),
            ),
        ))
    }
}

unsafe impl Send for V4LinuxDevice {}

unsafe impl Sync for V4LinuxDevice {} // NEVER MUTATE BETWEEN THREADS!!! NEVER SEND A MUTABLE `V4LinuxDevice`!!!

impl<'a> Webcam<'a> for UVCameraDevice<'a> {
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
                            x: u32::from(frame.width()),
                            y: u32::from(frame.height()),
                        };
                        if !resolutions.contains(&resolution_string) {
                            resolutions.push(resolution_string);
                        }
                    }
                }
                Ok(resolutions)
            }
            Err(why) => Err(Box::new(
                CannotGetDeviceInfo {
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
                            *fps,
                        ) {
                            framerates.push(DeviceFormat::YUYV);
                        }
                        if let Ok(_stream) = handler.get_stream_handle_with_format_size_and_fps(
                            uvc::FrameFormat::MJPEG,
                            res.x,
                            res.y,
                            *fps,
                        ) {
                            framerates.push(DeviceFormat::MJPEG);
                        }
                    } else {
                        return Err(Box::new(
                            CannotGetDeviceInfo {
                                prop: "Supported Resolutions".to_string(),
                                msg: "Could not get supported Framerate for UVC device!".to_string(),
                            },
                        ));
                    }
                } else {
                    return Err(Box::new(
                        CannotGetDeviceInfo {
                            prop: "Supported Resolutions".to_string(),
                            msg: "Could not get supported Framerate for UVC device!".to_string(),
                        },
                    ));
                }

                Ok(framerates)
            }
            Err(why) => Err(Box::new(
                CannotGetDeviceInfo {
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
                    handler.supported_formats().into_iter().collect();
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
                CannotGetDeviceInfo {
                    prop: "Supported Resolutions".to_string(),
                    msg: why.to_string(),
                },
            )),
        }
    }

    fn get_camera_format(&self) -> DeviceFormat {
        self.device_format.get()
    }

    fn set_camera_format(&self, format: DeviceFormat) {
        self.device_format.set(format);
    }

    fn get_camera_type(&self) -> WebcamType {
        self.device_type
    }

    fn open_stream(&'a self) -> Result<StreamType, Box<dyn std::error::Error>> {
        return match (self.device_resolution.get(), self.device_framerate.get()) {
            (Some(_res), Some(_fps)) => {
                let _format = match self.device_format.get() {
                    DeviceFormat::YUYV => FrameFormat::YUYV,
                    DeviceFormat::MJPEG => FrameFormat::MJPEG,
                };
                return match self.inner.open() {
                    Ok(handler) => Ok(StreamType::UVCStream(handler)),
                    Err(why) => return Err(Box::new(
                        CannotOpenStream(
                            why.to_string(),
                        ),
                    )),
                };
            }
            (Some(_), None) => Err(Box::new(
                CannotOpenStream(
                    "Missing required arguments Framerate".to_string(),
                ),
            )),
            (None, Some(_)) => Err(Box::new(
                CannotOpenStream(
                    "Missing required arguments Resolution".to_string(),
                ),
            )),
            (None, None) => Err(Box::new(
                CannotOpenStream(
                    "Missing required arguments Resolution, Framerate".to_string(),
                ),
            )),
        };
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

        let fps = self.device_framerate.get().unwrap_or(5);

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

unsafe impl<'a> Send for UVCameraDevice<'a> {}

unsafe impl<'a> Sync for UVCameraDevice<'a> {} // NEVER MUTATE BETWEEN THREADS!!! NEVER SEND A MUTABLE `UVCameraDevice`!!!

pub struct OpenCVCameraDevice<'a> {
    name: RefCell<String>,
    res: Cell<Resolution>,
    fps: Cell<u32>,
    index: Cell<u32>,
    cv2: &'a PyModule,
    cv2_capture: &'a PyAny,
}

impl<'a> OpenCVCameraDevice<'a> {
    pub fn from_possible_device(
        n: String,
        possible_device: PossibleDevice,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // set the meta parameters
        let res = Cell::new(possible_device.res());
        let fps = Cell::new(possible_device.fps());
        let name = RefCell::new(n);
        let index = match get_os_webcam_index(possible_device) {
            Ok(idx) => Cell::new(idx),
            Err(why) => {
                return Err(why);
            }
        };

        // initialize the python module
        let cv2 = Python::with_gil(|gil| match gil.import("cv2") {
            Ok(gil) => gil,
            Err(why) => {
                panic!("Could not import cv2!: {}", why);
            }
        });

        let globals = PyDict::new(python);
        globals.set_item("cv2", cv2);
        let cv2_capture = match cv2.call("VideoCapture", (index.get(), ), None) {
            Ok(ca) => {
                ca
            }
            Err(why) => {
                panic!("{}", why.to_string());
            }
        };

        if let Err(why) = set_property_init(cv2_capture) {
            return Err(why);
        }

        if let Err(why) = set_property_res(cv2_capture, res.get()) {
            return Err(why);
        }

        if let Err(why) = set_property_fps(cv2_capture, fps.get()) {
            return Err(why);
        }

        Ok(
            OpenCVCameraDevice {
                name,
                res,
                fps,
                index,
                cv2,
                cv2_capture,
            }
        )
    }

    pub fn from_device_contact(
        n: String,
        device_contact: DeviceContact,
        resolution: Resolution,
        framerate: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        godot_print!("dev_cont");

        match device_contact {
            DeviceContact::UVCAM {
                vendor_id,
                product_id,
                serial,
            } => {
                let pd = PossibleDevice::UVCAM {
                    vendor_id,
                    product_id,
                    serial,
                    res: resolution,
                    fps: framerate,
                    fmt: FrameFormat::MJPEG,
                };
                OpenCVCameraDevice::from_possible_device(n, pd)
            }
            DeviceContact::V4L2 { location } => {
                let pd = PossibleDevice::V4L2 {
                    location,
                    res: resolution,
                    fps: framerate,
                    fmt: FourCC::new(b"MJPG"),
                };
                OpenCVCameraDevice::from_possible_device(n, pd)
            }
        }
    }

    pub fn res(&self) -> Resolution {
        let res_y: f64 = cv2_cam.call_method("get", (CAP_PROP_FRAME_HEIGHT, ), None).unwrap().extract().unwrap();
        let res_x: f64 = cv2_cam.call_method("get", (CAP_PROP_FRAME_WIDTH, ), None).unwrap().extract().unwrap();
        return Resolution::new(res_x as u32, res_y as u32);
    }

    pub fn fps(&self) -> u32 {
        let fps: f64 = cv2_cam.call_method("get", (CAP_PROP_FPS, ), None).unwrap().extract().unwrap();
        return fps as u32;
    }

    pub fn idx(&self) -> u32 {
        self.index.get()
    }

    pub fn set_res(&self, new_res: Resolution) -> Result<(), Box<dyn std::error::Error>> {
        return set_property_res(self.cv2_capture, new_res);
    }

    pub fn set_fps(&self, new_fps: u32) -> Result<(), Box<dyn std::error::Error>> {
        return set_property_fps(self.cv2_capture, new_fps);
    }

    pub fn open_stream(&self) -> Result<(), Box<dyn std::error::Error>> {}

    // This function assumes that `open_stream()` has already been called.
    pub fn get_next_frame(&self) -> Result<Mat, Box<dyn std::error::Error>> {}

    pub fn name(&self) -> String {
        (&*self.name.borrow()).clone()
    }

    // hide the livings
    // dissolve them in lime,
    // all for a crime
    // of saying "cutting-"
    pub fn dispose_of_body(self) {}
}

// some example code
//     println!("a");
//     let gil = Python::acquire_gil();
//     let python = gil.python();
//
//     let cv2 = python.import("cv2").unwrap();
//     let globals = PyDict::new(python);
//     globals.set_item("cv2", cv2);
//     let cv2_cam = match cv2.call("VideoCapture", (0, 200), None) {
//         Ok(ca) => {
//             ca
//         }
//         Err(why) => {
//             panic!("{}",why.to_string());
//         }
//     };
//     println!("{}", cv2_cam.get_type().to_string());
//     let args = PyDict::new(python);
//     let res: f64 = cv2_cam.call_method("get", (3, ), None).unwrap().extract().unwrap();
//     println!("{}", res);
//
//     let bo: bool = cv2_cam.call_method("set", (3, 1920), None).unwrap().extract().unwrap();
//     let bo1: bool = cv2_cam.call_method("set", (4, 1080), None).unwrap().extract().unwrap();
//     let res: f64 = cv2_cam.call_method("get", (3, ), None).unwrap().extract().unwrap();
//     let res1: f64 = cv2_cam.call_method("get", (4, ), None).unwrap().extract().unwrap();
//
//     println!("{}, {}", res, res1);

fn set_property_init(cv2_capture: &PyAny) -> Result<(), Box<dyn std::error::Error>> {
    return match cv2_capture.call_method("set", (CAP_PROP_FORMAT as u32, CV_8UC3), None) {
        Ok(r) => {
            let ret: bool = r.extract().unwrap_or(false);
            if ret {
                let fourcc = VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap();
                return match cv2_capture.call_method("set", (CAP_PROP_FOURCC as u32, fourcc), None) {
                    Ok(r) => {
                        let ret: bool = r.extract().unwrap_or(false);
                        if ret {
                            Ok(())
                        } else {
                            Err(Box::new(InvalidDeviceError::CannotSetProperty(format!("CAP_PROP_FOURCC: MJPG: {}", fourcc))))
                        }
                    }
                    Err(why) => {
                        Err(Box::new(why))
                    }
                };
            } else {
                Err(Box::new(InvalidDeviceError::CannotSetProperty(format!("CAP_PROP_FOURCC: MJPG: {}", fourcc))))
            }
        }
        Err(why) => {
            return Err(Box::new(why));
        }
    };
}

fn set_property_res(cv2_capture: &PyAny, res: Resolution) -> Result<(), Box<dyn std::error::Error>> {
    return match cv2_capture.call_method("set", (CAP_PROP_FRAME_WIDTH as u32, res.x), None) {
        Ok(r) => {
            let ret: bool = r.extract().unwrap_or(false);
            if ret {
                return match cv2_capture.call_method("set", (CAP_PROP_FRAME_HEIGHT as u32, res.y), None) {
                    Ok(r) => {
                        let ret: bool = r.extract().unwrap_or(false);
                        if ret {
                            Ok(())
                        } else {
                            Err(Box::new(InvalidDeviceError::CannotSetProperty(format!("CAP_PROP_FRAME_HEIGHT: {}", res.y))))
                        }
                    }
                    Err(why) => {
                        Err(Box::new(why))
                    }
                };
            } else {
                Err(Box::new(InvalidDeviceError::CannotSetProperty(format!("CAP_PROP_FRAME_WIDTH: {}", res.x))))
            }
        }
        Err(why) => {
            Err(Box::new(why))
        }
    };
}

fn set_property_fps(cv2_capture: &PyAny, fps: u32) -> Result<(), Box<dyn std::error::Error>> {
    return match cv2_capture.call_method("set", (CAP_PROP_FPS as u32, fps), None) {
        Ok(r) => {
            let ret: bool = r.extract().unwrap_or(false);
            if ret {
                Ok(())
            } else {
                Err(Box::new(InvalidDeviceError::CannotSetProperty(format!("CAP_PROP_FPS: {}", fps))))
            }
        }
        Err(why) => {
            Err(Box::new(why))
        }
    };
}

fn get_api_pref_int() -> Result<u32, Box<dyn std::error::Error>> {
    return match std::env::consts::OS {
        "linux" => {
            Ok(CAP_V4L2 as u32)
        }
        "windows" => {
            Ok(CAP_MSMF as u32)
        }
        &_ => {
            Err(Box::new(InvalidPlatform("Expected platform [\"linux\", \"windows\"]".to_string())))
        }
    };
}