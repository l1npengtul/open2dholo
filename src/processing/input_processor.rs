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
use crate::{
    processing::face_detector::detectors::{
        dlib::dlib_detector::DLibDetector,
        util::{DetectorHardware, DetectorTrait, DetectorType, PointType},
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
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Duration,
};

//
// fn make_v4l_device(
//     location: &PathIndex,
//     res: Resolution,
//     fps: u32,
//     fmt: FourCC,
// ) -> Result<Device, Box<dyn std::error::Error>> {
//     let device = match location {
//         PathIndex::Path(path) => {
//             let dev = match Device::with_path(Path::new(path)) {
//                 Ok(d) => d,
//                 Err(why) => return Err(Box::new(why)),
//             };
//             dev
//         }
//         PathIndex::Index(idx) => {
//             let dev = match Device::new(*idx) {
//                 Ok(d) => d,
//                 Err(why) => return Err(Box::new(why)),
//             };
//             dev
//         }
//     };
//
//     let fcc = fmt;
//
//     let format = match device.format() {
//         Ok(mut f) => {
//             f.width = res.x;
//             f.height = res.y;
//             f.fourcc = fcc;
//             f
//         }
//         Err(_) => Format::new(res.x, res.y, fcc),
//     };
//
//     let param = Parameters::with_fps(fps);
//
//     if let Err(why) = device.set_format(&format) {
//         return Err(Box::new(why));
//     }
//     if let Err(why) = device.set_params(&param) {
//         return Err(Box::new(why));
//     }
//
//     Ok(device)
// }
//
// fn make_uvc_device<'a>(
//     vendor_id: Option<u16>,
//     product_id: Option<u16>,
//     serial: Option<String>,
// ) -> Result<UVCDevice<'a>, Box<dyn std::error::Error>> {
//     let device = match crate::UVC.find_device(
//         vendor_id.map(i32::from),
//         product_id.map(i32::from),
//         serial.as_deref(),
//     ) {
//         Ok(d) => d,
//         Err(why) => return Err(Box::new(why)),
//     };
//     Ok(device)
// }

pub struct InputProcessingThreadless {
    // device: PossibleDevice,
    pub device_held: RefCell<OpenCVCameraDevice>,
    // bruh wtf
    detector_type: Cell<DetectorType>,
    detector_hw: Cell<DetectorHardware>,
    face_detector: Arc<Mutex<Box<dyn DetectorTrait>>>,
    thread_pool: ThreadPool,
    int_sender_ft: Sender<PointType>,
    int_receiver_ft: Receiver<PointType>,
}

impl InputProcessingThreadless {
    pub fn new(
        name: Option<String>,
        device: PossibleDevice,
        detect_typ: DetectorType,
        detect_hw: DetectorHardware,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let device_held = match OpenCVCameraDevice::from_possible_device(
            name.unwrap_or_else(|| "".to_string()),
            device,
        ) {
            Ok(ocv) => RefCell::new(ocv),
            Err(why) => return Err(why),
        };

        let detector_type = Cell::new(detect_typ);
        let detector_hw = Cell::new(detect_hw);

        let face_detector: Arc<Mutex<Box<dyn DetectorTrait>>> =
            Arc::new(Mutex::new(Box::new(match detect_typ {
                DetectorType::DLibFHOG => DLibDetector::new(false),
                DetectorType::DLibCNN => DLibDetector::new(true),
            })));

        // TODO: Adjustable thread pool size
        let thread_pool = ThreadPool::new_named(
            "INPUT_PROCESSER".to_string(),
            4,
            8,
            Duration::from_millis(500),
        );

        let (int_sender_ft, int_receiver_ft) = flume::unbounded();

        Ok(InputProcessingThreadless {
            device_held,
            detector_type,
            detector_hw,
            face_detector,
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
        let device_held = match OpenCVCameraDevice::from_device_contact(
            name.unwrap_or_else(|| "".to_string()),
            device_contact,
            res,
            fps,
        ) {
            Ok(ocv) => RefCell::new(ocv),
            Err(why) => return Err(why),
        };

        let detector_type = Cell::new(detect_typ);
        let detector_hw = Cell::new(detect_hw);
        godot_print!("detect");

        let face_detector: Arc<Mutex<Box<dyn DetectorTrait>>> =
            Arc::new(Mutex::new(Box::new(match detect_typ {
                DetectorType::DLibFHOG => DLibDetector::new(false),
                DetectorType::DLibCNN => DLibDetector::new(true),
            })));

        // TODO: Adjustable thread pool size
        godot_print!("thread");

        let thread_pool = ThreadPool::new_named(
            "INPUT_PROCESSER".to_string(),
            4,
            8,
            Duration::from_millis(500),
        );

        let (int_sender_ft, int_receiver_ft) = flume::unbounded();
        godot_print!("input_process_ret");

        Ok(InputProcessingThreadless {
            device_held,
            detector_type,
            detector_hw,
            face_detector,
            thread_pool,
            int_sender_ft,
            int_receiver_ft,
        })
    }

    pub fn change_device(
        &self,
        name: Option<String>,
        new_device: PossibleDevice,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let device_held = match OpenCVCameraDevice::from_possible_device(
            name.unwrap_or_else(|| "".to_string()),
            new_device,
        ) {
            Ok(h) => h,
            Err(why) => return Err(why),
        };

        self.device_held.replace(device_held);
        Ok(())
    }

    pub fn add_workload(&self, img_height: u32, img_width: u32, img_data: Vec<u8>) {
        let send = self.int_sender_ft.clone();
        let detector = self.face_detector.clone();

        self.thread_pool.execute(move || {
            let locked = detector.lock();
            let i_d = img_data.as_slice();
            let faces = locked.detect_face_rects(img_height, img_width, i_d);
            let result =
                locked.detect_landmarks(&faces.get(0).unwrap(), img_height, img_width, i_d);
            if let Err(why) = send.send(result) {
                godot_print!("Error: {}", why.to_string());
            }
        })
    }

    pub fn capture_and_record(&self) -> Result<(), Box<dyn std::error::Error>> {
        let img_captured = match self.device_held.borrow().get_next_frame() {
            Ok(frame) => frame,
            Err(why) => return Err(why),
        };
        let res = { self.device_held.borrow().res() };
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
