use crate::util::{
    camera::{
        camera_device::{OpenCVCameraDevice, UVCameraDevice, V4LinuxDevice},
        device_utils::{DeviceContact, DeviceFormat, PossibleDevice, Resolution},
        webcam::Webcam,
    },
    misc::{BackendConfig, FullyCalculatedPacket},
};
use facial_processing::face_processor::FaceProcessorBuilder;
use flume::{Receiver, Sender, TryRecvError};
use rusty_pool::{JoinHandle, ThreadPool};
use std::{
    cell::{Cell, RefCell},
    time::Duration,
};
use std::sync::Arc;
use crate::util::misc::MessageType;
use std::error::Error;
use std::alloc::Global;
use image::{ImageBuffer, Rgb};

pub struct InputProcesser<'a> {
    // device: PossibleDevice,
    pub device_held: RefCell<Box<dyn Webcam<'a> + 'a>>,
    // bruh wtf
    backend_cfg: Cell<BackendConfig>,
    // face_detector: Arc<Mutex<Box<dyn DetectorTrait>>>,
    thread: JoinHandle<()>,
    sender_fromthread: Arc<Sender<FullyCalculatedPacket>>,
    receiver_fromthread: Receiver<FullyCalculatedPacket>,
    sender_tothread: Sender<MessageType>,
    receiver_tothread: Arc<Receiver<MessageType>>
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

        let (sender_fromthread, receiver_fromthread) = flume::unbounded();
        let (sender_tothread, receiver_tothread) = flume::unbounded();


        Ok(InputProcesser {
            device_held: RefCell::new(device_held),
            backend_cfg,
            thread: (),
            sender_fromthread: Arc::new(sender_fromthread),
            receiver_fromthread,
            sender_tothread,
            receiver_tothread: Arc::new(receiver_tothread)
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
        for point in self.receiver_fromthread.drain() {
            // lmao imagine using a blocking function in a loop that waits until everything has been dropped, couldn't be me
            point_vec.push(point);
        }
        point_vec
    }
}

fn process_input(cfg: BackendConfig, device: PossibleDevice,  sender: Arc<Sender<FullyCalculatedPacket>>, message: Arc<Receiver<MessageType>>) -> u8 {
    let processor = FaceProcessorBuilder::new()
        .with_backend(cfg.backend_as_facial())
        .with_input(cfg.res().x, cfg.res().y)
        .build()
        .unwrap();
    let init_res = device.res();
    let init_fps = device.fps();
    let mut device = match get_dyn_webcam(name, new_device) {
        Ok(webcam) => webcam,
        Err(_) => return -1
    };

    loop {
        if let Ok(msg_recv) = message.try_recv() {
            match msg_recv {
                MessageType::Die(code) => {
                    return code;
                }
                MessageType::SetDevice(new_dev) => {
                    device = match get_dyn_webcam(name, new_dev) {
                        Ok(webcam) => webcam,
                        Err(_) => return -1
                    };
                }
                MessageType::ChangeDevice(new_cfg) => {
                    let new_res = new_cfg.res;
                    let new_fps = new_cfg.fps;
                    if new_res != init_res {
                        device.set_resolution(new_res);
                    }
                    if new_fps != init_fps {
                        device.set_framerate(new_fps);
                    }
                }
            }
        }

        // get frame
        let frame = match  device.get_frame() {
            Ok(f) => f,
            Err(_) => {
                return -1;
            }
        };
        let res = device.get_resolution().unwrap();
        let image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(res.x, res.y, frame).unwrap();

        // detections
        let bbox = processor.calculate_face_bboxes(&image);
        if bbox.len() > 0 {
            let face_landmarks = processor.calculate_landmarks(&image, *bbox.get(0).unwrap());
        }

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
