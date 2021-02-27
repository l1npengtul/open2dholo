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

use crate::error::invalid_device_error::InvalidDeviceError::InvalidPlatform;
use crate::{
    error::invalid_device_error::InvalidDeviceError::{
        CannotFindDevice, CannotGetDeviceInfo, CannotOpenStream, CannotSetProperty,
    },
    make_dyn, ret_boxerr,
    util::camera::{
        device_utils::{
            get_os_webcam_index, DeviceContact, DeviceFormat, DeviceHolder, PathIndex,
            PossibleDevice, Resolution, StreamType,
        },
        webcam::{Webcam, WebcamType},
    },
};
use cv_convert::{TryFromCv, TryIntoCv};
use nalgebra::{DMatrix, MatrixMN};
use opencv::core::{Mat, CV_8UC1};
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, VideoCaptureProperties};
use opencv::{
    core::CV_8UC3,
    videoio::{
        VideoWriter, CAP_MSMF, CAP_PROP_FORMAT, CAP_PROP_FOURCC, CAP_PROP_FPS,
        CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_V4L2,
    },
};
use std::cell::{BorrowMutError, Cell, RefCell, RefMut};
use usb_enumeration::{enumerate, Filters};
use uvc::{FormatDescriptor, FrameFormat};
use v4l::{
    buffer::Type, format::Format, fraction::Fraction, framesize::FrameSizeEnum, io::mmap::Stream,
    video::capture::Parameters, video::traits::Capture, FourCC,
};

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
            Err(why) => Err(Box::new(CannotGetDeviceInfo {
                prop: "Supported Resolutions".to_string(),
                msg: why.to_string(),
            })),
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
            Err(why) => Err(Box::new(CannotGetDeviceInfo {
                prop: "Supported Format (FourCC)".to_string(),
                msg: why.to_string(),
            })),
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
            Err(why) => Err(Box::new(CannotGetDeviceInfo {
                prop: "Supported Framerates".to_string(),
                msg: why.to_string(),
            })),
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
            Err(why) => Err(Box::new(CannotOpenStream(why.to_string()))),
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
                    Err(why) => {
                        return Err(Box::new(CannotFindDevice(format!(
                            "{},{}:{} {}",
                            why.to_string(),
                            description.vendor_id,
                            description.product_id,
                            description.serial_number.unwrap_or_else(|| "".to_string())
                        ))));
                    }
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
        Err(Box::new(CannotFindDevice(format!(
            "i64-{}:i64-{} {}",
            vendor_id.unwrap_or(-1),
            product_id.unwrap_or(-1),
            serial_number.unwrap_or_else(|| "".to_string())
        ))))
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
                    Err(why) => {
                        return Err(Box::new(CannotFindDevice(format!(
                            "noaddr: {}",
                            why.to_string()
                        ))));
                    }
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
        Err(Box::new(CannotFindDevice("noaddr".to_string())))
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
            Err(why) => Err(Box::new(CannotGetDeviceInfo {
                prop: "Supported Resolutions".to_string(),
                msg: why.to_string(),
            })),
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
                        return Err(Box::new(CannotGetDeviceInfo {
                            prop: "Supported Resolutions".to_string(),
                            msg: "Could not get supported Framerate for UVC device!".to_string(),
                        }));
                    }
                } else {
                    return Err(Box::new(CannotGetDeviceInfo {
                        prop: "Supported Resolutions".to_string(),
                        msg: "Could not get supported Framerate for UVC device!".to_string(),
                    }));
                }

                Ok(framerates)
            }
            Err(why) => Err(Box::new(CannotGetDeviceInfo {
                prop: "Supported Resolutions".to_string(),
                msg: why.to_string(),
            })),
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
            Err(why) => Err(Box::new(CannotGetDeviceInfo {
                prop: "Supported Resolutions".to_string(),
                msg: why.to_string(),
            })),
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
                    Err(why) => return Err(Box::new(CannotOpenStream(why.to_string()))),
                };
            }
            (Some(_), None) => Err(Box::new(CannotOpenStream(
                "Missing required arguments Framerate".to_string(),
            ))),
            (None, Some(_)) => Err(Box::new(CannotOpenStream(
                "Missing required arguments Resolution".to_string(),
            ))),
            (None, None) => Err(Box::new(CannotOpenStream(
                "Missing required arguments Resolution, Framerate".to_string(),
            ))),
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

pub struct OpenCVCameraDevice {
    name: RefCell<String>,
    res: Cell<Resolution>,
    fps: Cell<u32>,
    index: Cell<u32>,
    video_capture: RefCell<VideoCapture>,
}

impl OpenCVCameraDevice {
    pub fn new(
        name: String,
        idx: u32,
        frame: u32,
        resolution: Resolution,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let video_capture = {
            // generate video capture with auto detect backend
            let mut v_cap = match VideoCapture::new(idx as i32, get_api_pref_int() as i32) {
                Ok(vc) => vc,
                Err(why) => ret_boxerr!(why),
            };

            set_property_init(&mut v_cap);
            if let Err(why) = set_property_res(&mut v_cap, resolution) {
                return Err(why);
            }
            if let Err(why) = set_property_fps(&mut v_cap, frame) {
                return Err(why);
            }

            RefCell::new(v_cap)
        };

        Ok(OpenCVCameraDevice {
            name: RefCell::new(name),
            res: Cell::new(resolution),
            fps: Cell::new(frame),
            index: Cell::new(idx),
            video_capture,
        })
    }

    pub fn from_possible_device(
        n: String,
        possible_device: PossibleDevice,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let name = RefCell::new(n);
        let res = Cell::new(possible_device.res());
        let fps = Cell::new(possible_device.fps());
        let idx = match get_os_webcam_index(possible_device) {
            Ok(i) => i,
            Err(why) => return Err(why),
        };

        let video_capture = {
            // generate video capture with auto detect backend
            let mut v_cap = match VideoCapture::new(idx as i32, get_api_pref_int() as i32) {
                Ok(vc) => vc,
                Err(why) => ret_boxerr!(why),
            };

            set_property_init(&mut v_cap);
            if let Err(why) = set_property_res(&mut v_cap, res.get()) {
                return Err(why);
            }
            if let Err(why) = set_property_fps(&mut v_cap, fps.get()) {
                return Err(why);
            }

            RefCell::new(v_cap)
        };

        Ok(OpenCVCameraDevice {
            name,
            res,
            fps,
            index: Cell::new(idx),
            video_capture,
        })
    }

    pub fn from_device_contact(
        n: String,
        device_contact: DeviceContact,
        resolution: Resolution,
        framerate: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
        let a = self
            .video_capture
            .borrow()
            .get(CAP_PROP_FRAME_HEIGHT)
            .unwrap();
        let b = self
            .video_capture
            .borrow()
            .get(CAP_PROP_FRAME_WIDTH)
            .unwrap();
        Resolution::new(a as u32, b as u32)
    }

    pub fn fps(&self) -> u32 {
        let fps = self.video_capture.borrow().get(CAP_PROP_FPS).unwrap();
        fps as u32
    }

    pub fn idx(&self) -> u32 {
        self.index.get()
    }

    pub fn set_res(&self, new_res: Resolution) -> Result<(), Box<dyn std::error::Error>> {
        {
            // make sure we drop the edit lock
            self.res.set(new_res);
        }
        return match self.video_capture.try_borrow_mut() {
            Ok(mut vc) => {
                let v_dev = &mut *vc;
                set_property_res(v_dev, self.res.get())
            }
            Err(why) => ret_boxerr!(why),
        };
    }

    pub fn set_fps(&self, frame: u32) -> Result<(), Box<dyn std::error::Error>> {
        {
            self.fps.set(frame);
        }
        return match self.video_capture.try_borrow_mut() {
            Ok(mut vc) => {
                let v_dev = &mut *vc;
                set_property_fps(v_dev, frame)
            }
            Err(why) => ret_boxerr!(why),
        };
    }

    pub fn open_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.video_capture.try_borrow_mut() {
            Ok(mut vc) => {
                let v_dev = &mut *vc;
                if v_dev.is_opened().unwrap_or(false) {
                    return Ok(());
                }
                match std::env::consts::OS {
                    "linux" => match v_dev.open(self.index.get() as i32, CAP_V4L2) {
                        Ok(o) => {
                            if o {
                                return Ok(());
                            }
                            ret_boxerr!(CannotOpenStream("Failed to open V4L2 Stream".to_string(),))
                        }
                        Err(why) => ret_boxerr!(why),
                    },
                    "windows" => match v_dev.open(self.index.get() as i32, CAP_MSMF) {
                        Ok(o) => {
                            if o {
                                return Ok(());
                            }
                            ret_boxerr!(CannotOpenStream(
                                "Failed to open MS Media Foundation Stream".to_string(),
                            ))
                        }
                        Err(why) => ret_boxerr!(why),
                    },
                    &_ => ret_boxerr!(InvalidPlatform("[\"linux\", \"windows\"]".to_string(),)),
                }
            }
            Err(why) => ret_boxerr!(why),
        };
    }

    // This function assumes that `open_stream()` has already been called.
    pub fn get_next_frame(&self) -> Result<DMatrix<u8>, Box<dyn std::error::Error>> {
        let res = self.res.get();
        let mut mat = match unsafe { Mat::new_rows_cols(res.y as i32, res.x as i32, CV_8UC3) } {
            Ok(m) => m,
            Err(why) => {
                ret_boxerr!(why)
            }
        };
        match self.video_capture.try_borrow_mut() {
            Ok(mut vc) => {
                let video = &mut *vc;
                video.read(&mut mat);
            }
            Err(why) => {
                ret_boxerr!(why);
            }
        }
        // TODO: convert Mat to array/slice/nalgebra matrix
    }

    // hide the body
    // dissolve it in lime,
    // all for a crime
    // of saying "pettan"
    pub fn dispose_of_body(self) {}
}
// fn set_property_init(vc: &mut VideoCapture) -> Result<(), Box<dyn std::error::Error>> {
//     // set the format to CV 8UC3
//     // match vc.set(CAP_PROP_FORMAT, f64::from(CV_8UC3)) {
//     //     Ok(r) => {
//     //         if r {
//     //             return match vc.set(
//     //                 CAP_PROP_FOURCC,
//     //                 f64::from(
//     //                     VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap(),
//     //                 ),
//     //             ) {
//     //                 Ok(r) => {
//     //                     if r {
//     //                         return Ok(());
//     //                     }
//     //                     ret_boxerr!(CannotSetProperty(
//     //                         "OpenCV returned `false` for CAP_PROP_FOURCC!".to_string()
//     //                     ))
//     //                 }
//     //                 Err(why) => Err(Box::new(why)),
//     //             };
//     //         }
//     //         ret_boxerr!(CannotSetProperty(
//     //             "OpenCV returned `false` for CAP_PROP_FORMAT!".to_string()
//     //         ))
//     //     }
//     //     Err(why) => ret_boxerr!(why),
//     // };
//     Ok(())
// }

fn set_property_res(
    vc: &mut VideoCapture,
    res: Resolution,
) -> Result<(), Box<dyn std::error::Error>> {
    match vc.set(
        VideoCaptureProperties::CAP_PROP_FRAME_HEIGHT as i32,
        f64::from(res.y),
    ) {
        Ok(r) => {
            if !r {
                ret_boxerr!(CannotSetProperty("CAP_PROP_FRAME_HEIGHT".to_string()))
            }
        }
        Err(why) => {
            return Err(Box::new(why));
        }
    }

    match vc.set(
        VideoCaptureProperties::CAP_PROP_FRAME_WIDTH as i32,
        f64::from(res.x),
    ) {
        Ok(r) => {
            if !r {
                ret_boxerr!(CannotSetProperty("CAP_PROP_FRAME_WIDTH".to_string()))
            }
        }
        Err(why) => {
            ret_boxerr!(why)
        }
    }

    Ok(())
}

fn set_property_fps(vc: &mut VideoCapture, fps: u32) -> Result<(), Box<dyn std::error::Error>> {
    match vc.set(VideoCaptureProperties::CAP_PROP_FPS as i32, f64::from(fps)) {
        Ok(r) => {
            if !r {
                ret_boxerr!(CannotSetProperty("CAP_PROP_FPS".to_string()))
            }
        }
        Err(why) => ret_boxerr!(why),
    }
    Ok(())
}

fn get_api_pref_int() -> u32 {
    match std::env::consts::OS {
        "linux" => CAP_V4L2 as u32,
        "windows" => CAP_MSMF as u32,
        &_ => 0,
    }
}
