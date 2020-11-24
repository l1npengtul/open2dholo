use std::collections::HashMap;
use crate::util::camera::{webcam::Webcam, camera_device::V4LinuxDevice};
use v4l::device::List;

pub fn enumerate() -> Option<HashMap<String, Box<dyn Webcam>>> {
    return match std::env::consts::OS {
        "linux" => {
            let mut known_devices: HashMap<String, Box<dyn Webcam>> = HashMap::new();
            for sys_device in List::new() {
                let v4l_device = match V4LinuxDevice::new(sys_device.index().unwrap_or(0)) {
                    Ok(dev) => Box::new(dev),
                    Err(_why) => continue,
                };
                if !known_devices.contains_key(&v4l_device.name()) {
                    known_devices.insert(v4l_device.name(), v4l_device);
                }
            } 
            Some(known_devices)
        }
        "macos" | "windows" => {
            let mut known_devices: HashMap<String, Box<dyn Webcam>> = HashMap::new();
            match crate::UVC.devices() {
                Ok(list) => {
                    for uvc_device in list {
                        
                    }
                }
                Err(_why) => {
                    return None;
                }
            }
            Some(known_devices)
        }
        _ => {
            None
        }
    } 
}