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
use crate::util::camera::camera_device::{UVCameraDevice, V4LinuxDevice};
use crate::util::camera::device_utils::DeviceFormat;
use crate::util::camera::webcam::Webcam;
use crate::{
    processing::face_detector::detectors::util::{
        DetectorHardware, DetectorTrait, DetectorType, PointType,
    },
    util::camera::{
        camera_device::OpenCVCameraDevice,
        device_utils::{DeviceContact, PossibleDevice, Resolution},
    },
};
use flume::{Receiver, Sender};
use gdnative::godot_print;
use parking_lot::Mutex;
use rusty_pool::ThreadPool;
use std::error::Error;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Duration,
};

pub struct InputProcesser<'a> {
    // device: PossibleDevice,
    pub device_held: RefCell<Box<dyn Webcam<'a> + 'a>>,
    // bruh wtf
    detector_type: Cell<DetectorType>,
    detector_hw: Cell<DetectorHardware>,
    // face_detector: Arc<Mutex<Box<dyn DetectorTrait>>>,
    thread_pool: ThreadPool,
    int_sender_ft: Sender<PointType>,
    int_receiver_ft: Receiver<PointType>,
}

impl<'a> InputProcesser<'a> {
    pub fn new(
        name: Option<String>,
        device: PossibleDevice,
        detect_typ: DetectorType,
        detect_hw: DetectorHardware,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let device_held: Box<dyn Webcam<'a> + 'a> = match get_dyn_webcam(name, device) {
            Ok(webcam) => webcam,
            Err(why) => return Err(why),
        };
        device_held.open_stream();
        let detector_type = Cell::new(detect_typ);
        let detector_hw = Cell::new(detect_hw);

        // let face_detector: Arc<Mutex<Box<dyn DetectorTrait>>> =
        //     Arc::new(Mutex::new(Box::new(match detect_typ {
        //         DetectorType::DLibFHOG => DLibDetector::new(false),
        //         DetectorType::DLibCNN => DLibDetector::new(true),
        //     })));

        // TODO: Adjustable thread pool size
        let thread_pool = ThreadPool::new_named(
            "INPUT_PROCESSER".to_string(),
            4,
            8,
            Duration::from_millis(500),
        );

        let (int_sender_ft, int_receiver_ft) = flume::unbounded();

        Ok(InputProcesser {
            device_held: RefCell::new(device_held),
            detector_type,
            detector_hw,
            thread_pool,
            int_sender_ft,
            int_receiver_ft,
        })
    }

    pub fn from_device_contact(
        name: Option<String>,
        device_contact: DeviceContact,
        res: Resolution,
        fps: u32,
        detect_typ: DetectorType,
        detect_hw: DetectorHardware,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let device =
            PossibleDevice::from_device_contact(device_contact, res, fps, DeviceFormat::MJPEG);
        InputProcesser::new(name, device, detect_typ, detect_hw)
    }

    pub fn change_device(
        &self,
        name: Option<String>,
        new_device: PossibleDevice,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let device_held: Box<dyn Webcam<'a> + 'a> = match get_dyn_webcam(name, new_device) {
            Ok(webcam) => webcam,
            Err(why) => return Err(why),
        };

        device_held.open_stream();

        self.device_held.replace(device_held);
        Ok(())
    }

    pub fn add_workload(&self, _img_height: u32, _img_width: u32, _img_data: Vec<u8>) {
        // let send = self.int_sender_ft.clone();
        // let detector = self.face_detector.clone();
        //
        // self.thread_pool.execute(move || {
        //     let locked = detector.lock();
        //     let i_d = img_data.as_slice();
        //     let faces = locked.detect_face_rects(img_height, img_width, i_d);
        //     let result =
        //         locked.detect_landmarks(&faces.get(0).unwrap(), img_height, img_width, i_d);
        //     if let Err(why) = send.send(result) {
        //         godot_print!("Error: {}", why.to_string());
        //     }
        // })
    }

    pub fn capture_frame(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.device_held.borrow().get_frame()
    }

    pub fn capture_and_record(&self) -> Result<(), Box<dyn std::error::Error>> {
        let img_captured = match self.device_held.borrow().get_frame() {
            Ok(frame) => frame,
            Err(why) => return Err(why),
        };
        let res = { self.device_held.borrow().get_resolution().unwrap() };
        self.add_workload(res.y, res.x, img_captured);
        Ok(())
    }

    pub fn query_gotten_results(&self) -> Vec<PointType> {
        let mut point_vec = Vec::new();
        for point in self.int_receiver_ft.drain() {
            // lmao imagine using a blocking function in a loop that waits until everything has been dropped, couldn't be me
            point_vec.push(point);
        }
        point_vec
    }
}

fn get_dyn_webcam<'a>(
    name: Option<String>,
    device: PossibleDevice,
) -> Result<Box<dyn Webcam<'a> + 'a>, Box<dyn std::error::Error>> {
    let device_held: Box<dyn Webcam<'a>> = match device {
        PossibleDevice::UVCAM {
            vendor_id,
            product_id,
            serial,
            res,
            fps,
            fmt: _fmt,
        } => {
            let uvcam: UVCameraDevice<'a> = match UVCameraDevice::new_camera(
                vendor_id.map(|v| v as i32),
                product_id.map(|v| v as i32),
                serial,
            ) {
                Ok(camera) => camera,
                Err(why) => {
                    return Err(why);
                }
            };
            uvcam.set_framerate(&fps);
            uvcam.set_resolution(&res);
            Box::new(uvcam)
        }
        PossibleDevice::V4L2 {
            location,
            res,
            fps,
            fmt: _fmt,
        } => {
            let v4lcam = match V4LinuxDevice::new_location(location) {
                Ok(device) => device,
                Err(why) => {
                    return Err(why);
                }
            };
            v4lcam.set_resolution(&res);
            v4lcam.set_framerate(&fps);
            Box::new(v4lcam)
        }
        PossibleDevice::OPENCV {
            index: _index,
            res,
            fps,
            fmt: _fmt,
        } => {
            let ocvcam = match OpenCVCameraDevice::from_possible_device(
                name.unwrap_or("OpenCV Camera".to_string()),
                device,
            ) {
                Ok(device) => device,
                Err(why) => {
                    return Err(why);
                }
            };
            ocvcam.set_resolution(&res);
            ocvcam.set_framerate(&fps);
            Box::new(ocvcam)
        }
    };

    Ok(device_held)
}

// hack us election
// make trump president
// i get paid 19 ddollar fortnite card
//
// stonks
// pub fn _hack_us_election(hackable: bool) {
//     if hackable {
//         // joe_biden.make_not_president();
//         // wait if i make jo bidon lose he cant do femboy monetary compensation
//         // must make drump lose for femboy money i must
//         trump.lose_presidency();
//     }
// }
// suggestion added on recommendation of D.T., a russian/bi'ish person
// this is a fucking joke lmao
