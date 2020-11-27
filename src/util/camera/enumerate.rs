use crate::util::camera::{
    camera_device::{UVCameraDevice, V4LinuxDevice},
    webcam::Webcam,
};
use std::collections::HashMap;
use v4l::device::List;

pub fn enumerate_devices() -> Option<HashMap<String, Box<dyn Webcam>>> {
    return match std::env::consts::OS {
        "linux" => {
            let mut known_devices: HashMap<String, Box<dyn Webcam>> = HashMap::new();
            // get device list from v4l2
            for sys_device in List::new() {
                // get device from v4l2 using the index, getting /dev/video0 if it falis
                let v4l_device = match V4LinuxDevice::new(sys_device.index().unwrap_or(0)) {
                    Ok(dev) => Box::new(dev),
                    Err(_why) => continue,
                };
                // weed out the repeating
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
                        if let Ok(camera_device) = UVCameraDevice::from_device(uvc_device) {
                            let camera_name = camera_device.name();
                            if !known_devices.contains_key(&camera_name) {
                                known_devices.insert(camera_name, Box::new(camera_device));
                            }
                        }
                    }
                }
                Err(_why) => {
                    return None;
                }
            }
            Some(known_devices)
        }
        _ => None,
    };
}
