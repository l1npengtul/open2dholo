use crate::util::{
    camera::{
        camera_device::{OpenCVCameraDevice, UVCameraDevice, V4LinuxDevice},
        device_utils::{DeviceContact, DeviceFormat, PossibleDevice, Resolution},
        webcam::Webcam,
    },
    misc::{BackendConfig, FullyCalculatedPacket},
};
use facial_processing::face_processor::FaceProcessorBuilder;
use flume::{Receiver, Sender};
use rusty_pool::{JoinHandle, ThreadPool};
use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

pub struct InputProcesser<'a> {
    // device: PossibleDevice,
    pub device_held: RefCell<Box<dyn Webcam<'a> + 'a>>,
    // bruh wtf
    backend_cfg: Cell<BackendConfig>,
    // face_detector: Arc<Mutex<Box<dyn DetectorTrait>>>,
    thread: JoinHandle<(), ()>,
    int_sender_ft: Sender<FullyCalculatedPacket>,
    int_receiver_ft: Receiver<FullyCalculatedPacket>,
}

impl<'a> InputProcesser<'a> {
    pub fn new(
        name: Option<String>,
        device: PossibleDevice,
        config: BackendConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let device_held: Box<dyn Webcam<'a> + 'a> = match get_dyn_webcam(name, device) {
            Ok(webcam) => webcam,
            Err(why) => return Err(why),
        };
        device_held.open_stream();
        let backend_cfg = Cell::new(config);

        // let face_detector: Arc<Mutex<Box<dyn DetectorTrait>>> =
        //     Arc::new(Mutex::new(Box::new(match detect_typ {
        //         DetectorType::DLibFHOG => DLibDetector::new(false),
        //         DetectorType::DLibCNN => DLibDetector::new(true),
        //     })));

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

fn process_input(cfg: BackendConfig, sender: Sender<FullyCalculatedPacket>) {
    let processor = FaceProcessorBuilder::new().with_backend();
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
                vendor_id.map(|v| i32::from(v)),
                product_id.map(|v| i32::from(v)),
                serial,
            ) {
                Ok(camera) => camera,
                Err(why) => {
                    return Err(why);
                }
            };
            uvcam.set_framerate(fps);
            uvcam.set_resolution(res);
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
            v4lcam.set_resolution(res);
            v4lcam.set_framerate(fps);
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
            ocvcam.set_resolution(res);
            ocvcam.set_framerate(fps);
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
