//     Open2DHolo - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
//     Copyright (C) 2020-2021 l1npengtul
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

use crate::{
    error::{
        conversion_error::ConversionError::{ConversionFromError, MatchFailedError},
        invalid_device_error::InvalidDeviceError::CannotFindDevice,
    },
    ret_boxerr,
    util::camera::{
        camera_device::{UVCameraDevice, V4LinuxDevice},
        webcam::QueryCamera,
    },
};
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering, collections::HashMap, convert::TryFrom, error::Error, fmt::Display,
    fmt::Formatter, os::raw::c_int,
};
use usb_enumeration::UsbDevice;
use uvc::{DeviceHandle, FrameFormat};
use v4l::{framesize::FrameSizeEnum, prelude::*, FourCC};

#[derive(Clone, Deserialize, Serialize)]
pub struct DeviceDesc {
    pub(crate) vid: Option<c_int>,
    pub(crate) pid: Option<c_int>,
    pub(crate) ser: Option<String>,
}

impl DeviceDesc {
    pub fn new(device: &uvc::Device) -> Result<Self, Box<dyn Error>> {
        let device_desc = device.description()?;
        Ok(DeviceDesc {
            vid: Some(c_int::from(device_desc.vendor_id)),
            pid: Some(c_int::from(device_desc.product_id)),
            ser: device_desc.serial_number,
        })
    }
    pub fn from_description(device: uvc::DeviceDescription) -> Self {
        DeviceDesc {
            vid: Some(c_int::from(device.vendor_id)),
            pid: Some(c_int::from(device.product_id)),
            ser: device.serial_number,
        }
    }
    pub fn from_default() -> Self {
        DeviceDesc {
            vid: None,
            pid: None,
            ser: None,
        }
    }
}

#[derive(Clone)]
pub struct DeviceHolder {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub description: String,
    pub serial: Option<String>,
}

impl DeviceHolder {
    pub fn new(
        id: String,
        vendor_id: u16,
        product_id: u16,
        description: String,
        serial: Option<String>,
    ) -> Self {
        DeviceHolder {
            id,
            vendor_id,
            product_id,
            description,
            serial,
        }
    }

    pub fn from_devices(
        usb: &UsbDevice,
        uvc: &uvc::Device,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(uvc_desc) = uvc.description() {
            if uvc_desc.vendor_id == usb.vendor_id && uvc_desc.product_id == usb.product_id {
                let mut description: String =
                    format!("{}:{}", uvc_desc.vendor_id, uvc_desc.product_id);
                let serial = uvc_desc.serial_number.clone();
                if let Some(descript) = usb.description.clone() {
                    description = format!("{} {}", description, descript);
                }
                let device: DeviceHolder = DeviceHolder::new(
                    usb.id.clone(),
                    uvc_desc.vendor_id,
                    uvc_desc.product_id,
                    description,
                    serial,
                );
                return Ok(device);
            }
        }
        Err(Box::new(CannotFindDevice("noaddr".to_string())))
    }
}

impl PartialEq for DeviceHolder {
    fn eq(&self, other: &Self) -> bool {
        if self.description == other.description
            && self.product_id == other.product_id
            && self.vendor_id == other.vendor_id
            && self.id == other.id
        {
            return false;
        }
        true
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolution {
    pub x: u32,
    pub y: u32,
}

impl Resolution {
    pub fn new(x: u32, y: u32) -> Self {
        Resolution { x, y }
    }

    pub fn from_variant(var: &Variant) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(v) = var.try_to_vector2() {
            return if v.x > 0.0 && v.y > 0.0 {
                let x = v.x as u32;
                let y = v.y as u32;
                Ok(Resolution { x, y })
            } else {
                Err(Box::new(ConversionFromError {
                    from: "Variant".to_string(),
                    to: "u32".to_string(),
                }))
            };
        }
        Err(Box::new(ConversionFromError {
            from: "Variant".to_string(),
            to: "Vector2".to_string(),
        }))
    }
}

impl TryFrom<v4l::framesize::FrameSize> for Resolution {
    type Error = String;

    fn try_from(value: v4l::framesize::FrameSize) -> Result<Self, Self::Error> {
        Ok(match value.size {
            FrameSizeEnum::Stepwise(step) => Resolution {
                x: step.max_width,
                y: step.max_height,
            },
            FrameSizeEnum::Discrete(dis) => Resolution {
                x: dis.width,
                y: dis.height,
            },
        })
    }
}

// impl PartialEq for Resolution {
//     fn eq(&self, other: &Self) -> bool {
//         if self.x == other.x && self.y == other.y {
//             return false;
//         }
//         true
//     }
// }

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Resolution {
    // Flip around the order to make it seem the way the user would expect.
    // The user would expect a descending list of resolutions (aka highest -> lowest)
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x.cmp(&other.x) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.y.cmp(&other.y),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DeviceFormat {
    Yuyv,
    MJpeg,
}

impl DeviceFormat {
    pub fn from_variant(var: &Variant) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(st) = var.try_to_string() {
            return match &st.to_lowercase()[..] {
                "yuyv" => Ok(DeviceFormat::Yuyv),
                "mjpg" | "mjpeg" => Ok(DeviceFormat::MJpeg),
                _ => Err(Box::new(MatchFailedError(st))),
            };
        }
        Err(Box::new(ConversionFromError {
            from: "Variant".to_string(),
            to: "String".to_string(),
        }))
    }
}

impl Display for DeviceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceFormat::Yuyv => {
                write!(f, "YUYV")
            }
            DeviceFormat::MJpeg => {
                write!(f, "MJPG")
            }
        }
    }
}

pub enum StreamType<'a> {
    Video4Linux2Stream(MmapStream<'a>),
    UniversalVideoStream(DeviceHandle<'a>),
}

#[derive(Clone)]
pub enum PossibleDevice {
    UniversalVideoCamera {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        serial: Option<String>,
        res: Resolution,
        fps: u32,
        fmt: FrameFormat,
    },
    Video4Linux2 {
        location: PathIndex,
        res: Resolution,
        fps: u32,
        fmt: FourCC,
    },
    OpenComVision {
        index: u32,
        res: Resolution,
        fps: u32,
        fmt: FrameFormat,
    },
}

impl<'a> PossibleDevice {
    pub fn from_cached_device(
        cached: CachedDeviceList,
        res: Resolution,
        fps: u32,
        fmt: DeviceFormat,
    ) -> PossibleDevice {
        match &cached.device_location {
            DeviceContact::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
            } => {
                let dev_format = match fmt {
                    DeviceFormat::Yuyv => FrameFormat::YUYV,
                    DeviceFormat::MJpeg => FrameFormat::MJPEG,
                };

                PossibleDevice::UniversalVideoCamera {
                    vendor_id: vendor_id.to_owned(),
                    product_id: product_id.to_owned(),
                    serial: serial.clone(),
                    res,
                    fps,
                    fmt: dev_format,
                }
            }
            DeviceContact::Video4Linux2 { location } => {
                let dev_format = match fmt {
                    DeviceFormat::Yuyv => FourCC::new(b"MJPG"),
                    DeviceFormat::MJpeg => FourCC::new(b"YUYV"),
                };
                let lc: PathIndex = match location {
                    PathIndex::Path(p) => PathIndex::Path(p.clone()),
                    PathIndex::Index(i) => PathIndex::Index(*i),
                };
                PossibleDevice::Video4Linux2 {
                    location: lc,
                    res,
                    fps,
                    fmt: dev_format,
                }
            }
            DeviceContact::OpenComVision { index } => PossibleDevice::OpenComVision {
                index: *index,
                res,
                fps,
                fmt: FrameFormat::MJPEG,
            },
        }
    }

    pub fn from_device_contact(
        contact: DeviceContact,
        res: Resolution,
        fps: u32,
        fmt: DeviceFormat,
    ) -> Self {
        match contact {
            DeviceContact::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
            } => {
                let dev_format = match fmt {
                    DeviceFormat::Yuyv => FrameFormat::YUYV,
                    DeviceFormat::MJpeg => FrameFormat::MJPEG,
                };

                PossibleDevice::UniversalVideoCamera {
                    vendor_id: vendor_id.to_owned(),
                    product_id: product_id.to_owned(),
                    serial,
                    res,
                    fps,
                    fmt: dev_format,
                }
            }
            DeviceContact::Video4Linux2 { location } => {
                let dev_format = match fmt {
                    DeviceFormat::Yuyv => FourCC::new(b"MJPG"),
                    DeviceFormat::MJpeg => FourCC::new(b"YUYV"),
                };
                let lc: PathIndex = match location {
                    PathIndex::Path(p) => PathIndex::Path(p),
                    PathIndex::Index(i) => PathIndex::Index(i),
                };
                PossibleDevice::Video4Linux2 {
                    location: lc,
                    res,
                    fps,
                    fmt: dev_format,
                }
            }
            DeviceContact::OpenComVision { index } => PossibleDevice::OpenComVision {
                index,
                res,
                fps,
                fmt: FrameFormat::MJPEG,
            },
        }
    }

    pub fn to_device_contact(&self) -> DeviceContact {
        match self {
            PossibleDevice::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
                res: _res,
                fps: _fps,
                fmt: _fmt,
            } => DeviceContact::UniversalVideoCamera {
                vendor_id: *vendor_id,
                product_id: *product_id,
                serial: serial.clone(),
            },
            PossibleDevice::Video4Linux2 {
                location,
                res: _res,
                fps: _fps,
                fmt: _fmt,
            } => DeviceContact::Video4Linux2 {
                location: location.clone(),
            },
            PossibleDevice::OpenComVision {
                index,
                res: _res,
                fps: _fps,
                fmt: _fmt,
            } => DeviceContact::OpenComVision { index: *index },
        }
    }

    pub fn res(&self) -> Resolution {
        match self {
            PossibleDevice::UniversalVideoCamera {
                vendor_id: _vendor_id,
                product_id: _product_id,
                serial: _serial,
                res,
                fps: _fps,
                fmt: _fmt,
            } => *res,
            PossibleDevice::Video4Linux2 {
                location: _location,
                res,
                fps: _fps,
                fmt: _fmt,
            } => *res,
            PossibleDevice::OpenComVision {
                index: _index,
                res,
                fps: _fps,
                fmt: _fmt,
            } => *res,
        }
    }

    pub fn fps(&self) -> u32 {
        match self {
            PossibleDevice::UniversalVideoCamera {
                vendor_id: _vendor_id,
                product_id: _product_id,
                serial: _serial,
                res: _res,
                fps,
                fmt: _fmt,
            } => *fps,
            PossibleDevice::Video4Linux2 {
                location: _location,
                res: _res,
                fps,
                fmt: _fmt,
            } => *fps,
            PossibleDevice::OpenComVision {
                index: _index,
                res: _res,
                fps,
                fmt: _fmt,
            } => *fps,
        }
    }

    pub fn fmt(&self) -> DeviceFormat {
        DeviceFormat::MJpeg
    }

    pub fn change_config(self, dev_cfg: DeviceConfig) -> Self {
        match self {
            PossibleDevice::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
                res: _,
                fps: _,
                fmt,
            } => PossibleDevice::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
                res: dev_cfg.res,
                fps: dev_cfg.fps,
                fmt,
            },
            PossibleDevice::Video4Linux2 {
                location,
                res: _,
                fps: _,
                fmt,
            } => PossibleDevice::Video4Linux2 {
                location,
                res: dev_cfg.res,
                fps: dev_cfg.fps,
                fmt,
            },
            PossibleDevice::OpenComVision {
                index,
                res: _,
                fps: _,
                fmt,
            } => PossibleDevice::OpenComVision {
                index,
                res: dev_cfg.res,
                fps: dev_cfg.fps,
                fmt,
            },
        }
    }
}

#[derive(Clone)]
pub enum PathIndex {
    Path(String),
    Index(usize),
}

impl Display for PathIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathIndex::Path(p) => {
                write!(f, "{}", p)
            }
            PathIndex::Index(i) => {
                write!(f, "{}", i)
            }
        }
    }
}

#[derive(Clone)]
pub enum DeviceContact {
    UniversalVideoCamera {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        serial: Option<String>,
    },
    Video4Linux2 {
        location: PathIndex,
    },
    OpenComVision {
        index: u32,
    },
}

impl DeviceContact {
    pub fn from_possible_device(dev: &PossibleDevice) -> Self {
        match dev.clone() {
            PossibleDevice::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
                res: _res,
                fps: _fps,
                fmt: _fmt,
            } => DeviceContact::UniversalVideoCamera {
                vendor_id,
                product_id,
                serial,
            },
            PossibleDevice::Video4Linux2 {
                location,
                res: _res,
                fps: _fps,
                fmt: _fmt,
            } => DeviceContact::Video4Linux2 { location },
            PossibleDevice::OpenComVision {
                index,
                res: _res,
                fps: _fps,
                fmt: _fmt,
            } => DeviceContact::OpenComVision { index },
        }
    }
}

impl From<PossibleDevice> for DeviceContact {
    fn from(value: PossibleDevice) -> Self {
        DeviceContact::from_possible_device(&value)
    }
}

#[derive(Clone)]
pub struct CachedDeviceList {
    device_name: String,
    device_location: DeviceContact,
    device_format_mjpg: Box<HashMap<Resolution, Vec<u32>>>,
    device_format_yuyv: Box<HashMap<Resolution, Vec<u32>>>,
}

impl CachedDeviceList {
    // DO NOT REMOVE THE `&`
    pub fn from_webcam(camera: &dyn QueryCamera) -> Result<Self, Box<dyn std::error::Error>> {
        let device_name = camera.name();
        let device_location = camera.get_location();
        let mut resolutions = match camera.get_supported_resolutions() {
            Ok(res) => res,
            Err(why) => {
                return Err(why);
            }
        };

        resolutions.sort();

        let mut fmt_res_mjpg: HashMap<Resolution, Vec<u32>> = HashMap::new();

        for res in resolutions {
            if let Ok(framerates) = camera.get_supported_framerate(res) {
                fmt_res_mjpg.insert(res, framerates.clone());
            }
        }
        Ok(Self {
            device_name,
            device_location,
            device_format_yuyv: Box::new(fmt_res_mjpg.clone()),
            device_format_mjpg: Box::new(fmt_res_mjpg),
        })
    }

    pub fn set_custom_cached_idx(&mut self, idx: u32) {
        self.device_location = DeviceContact::OpenComVision { index: idx };
    }

    pub fn get_name(&self) -> String {
        self.device_name.clone()
    }

    pub fn get_location(&self) -> DeviceContact {
        self.device_location.clone()
    }

    pub fn get_supported_yuyv(&self) -> Box<HashMap<Resolution, Vec<u32>>> {
        self.device_format_yuyv.clone()
    }

    pub fn get_supported_mjpg(&self) -> Box<HashMap<Resolution, Vec<u32>>> {
        self.device_format_mjpg.clone()
    }
}

impl PartialEq for CachedDeviceList {
    fn eq(&self, other: &Self) -> bool {
        if self.device_name == other.device_name {
            return true;
        }
        false
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct DeviceConfig {
    pub res: Resolution,
    pub fps: u32,
}
impl From<PossibleDevice> for DeviceConfig {
    fn from(val: PossibleDevice) -> Self {
        DeviceConfig {
            res: val.res(),
            fps: val.fps(),
        }
    }
}
pub fn enumerate_cache_device() -> Option<HashMap<String, CachedDeviceList>> {
    let mut known_devices: HashMap<String, CachedDeviceList> = HashMap::new();
    // get device list from v4l2
    match std::env::consts::OS {
        "linux" => {
            for dev in v4l::context::enum_devices() {
                if let Ok(v4l_dev) = V4LinuxDevice::new(dev.index()) {
                    let b: Box<dyn QueryCamera> = Box::new(v4l_dev);
                    if let Ok(c_dev) = CachedDeviceList::from_webcam(b.as_ref()) {
                        known_devices.insert(
                            dev.name()
                                .unwrap_or(format!("/dev/video{}", dev.index()))
                                .to_string(),
                            c_dev,
                        );
                    }
                }
            }
        }
        "windows" | "macos" => {
            // assume libuvc list == opencv list
            match crate::UVC.devices() {
                Ok(list) => {
                    for (idx, uvc_device) in list.enumerate() {
                        if let Ok(mut camera_device) = {
                            let b: Box<dyn QueryCamera> =
                                Box::new(UVCameraDevice::from_device(uvc_device).unwrap());
                            CachedDeviceList::from_webcam(b.as_ref())
                        } {
                            let dev_name = camera_device.get_name();
                            camera_device.set_custom_cached_idx(idx as u32);
                            // weed out the repeating
                            known_devices.entry(dev_name).or_insert(camera_device);
                        }
                    }
                }
                Err(_why) => {
                    return None;
                }
            }
        }
        &_ => {
            return None;
        }
    }
    Some(known_devices)
}

pub fn get_os_webcam_index(device: PossibleDevice) -> Result<u32, Box<dyn std::error::Error>> {
    match device {
        PossibleDevice::UniversalVideoCamera {
            vendor_id,
            product_id,
            serial,
            res: _res,
            fps: _fps,
            fmt: _fmt,
        } => {
            match crate::UVC.devices() {
                Ok(list) => {
                    for (idx, uvc_device) in list.enumerate() {
                        match uvc_device.description() {
                            Ok(desc) => {
                                if vendor_id == Some(desc.vendor_id)
                                    && product_id == Some(desc.product_id)
                                    && serial == desc.serial_number
                                {
                                    return Ok(idx as u32);
                                }
                                ret_boxerr!(CannotFindDevice("Index not found!".to_string()))
                            }
                            Err(why) => ret_boxerr!(why),
                        }
                    }
                }
                Err(why) => {
                    ret_boxerr!(why)
                }
            }
            Ok(0)
        }
        PossibleDevice::Video4Linux2 {
            location,
            res: _res,
            fps: _fps,
            fmt: _fmt,
        } => match location {
            PathIndex::Path(p) => {
                // let mut idx = 0_u32;
                let mut p_owned = p;
                for ch in 0..10 {
                    // /dev/video = 10
                    p_owned.remove(ch);
                }
                match p_owned.parse::<u32>() {
                    Ok(i) => Ok(i),
                    Err(why) => Err(Box::new(CannotFindDevice(why.to_string()))),
                }
            }
            PathIndex::Index(i) => Ok(i as u32),
        },
        PossibleDevice::OpenComVision {
            index,
            res: _res,
            fps: _fps,
            fmt: _fmt,
        } => Ok(index),
    }
}
