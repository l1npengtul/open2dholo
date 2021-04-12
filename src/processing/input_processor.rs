use crate::{
    util::misc::MessageType,
    util::{
        camera::{
            camera_device::{OpenCvCameraDevice, UVCameraDevice, V4LinuxDevice},
            device_utils::{DeviceContact, DeviceFormat, PossibleDevice, Resolution},
            webcam::Webcam,
        },
        misc::{BackendConfig, FullyCalculatedPacket},
    },
};
use facial_processing::face_processor::FaceProcessorBuilder;
use flume::{Receiver, Sender};
use image::{ImageBuffer, Rgb};
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    thread::{Builder, JoinHandle},
};

pub struct InputProcesser {
    device: RefCell<PossibleDevice>,
    // bruh wtf
    backend_cfg: Cell<BackendConfig>,
    // face_detector: Arc<Mutex<Box<dyn DetectorTrait>>>,
    thread: JoinHandle<u8>,
    sender_fromthread: Arc<Sender<FullyCalculatedPacket>>,
    receiver_fromthread: Receiver<FullyCalculatedPacket>,
    sender_tothread: Sender<MessageType>,
    receiver_tothread: Arc<Receiver<MessageType>>,
}

impl InputProcesser {
    pub fn new(
        device: PossibleDevice,
        config: BackendConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backend_cfg = Cell::new(config);
        let (sender_fromthread, receiver_fromthread) = flume::unbounded();
        let (sender_tothread, receiver_tothread) = flume::unbounded();

        let thread = Builder::new()
            .name("input_processor".to_string())
            .stack_size(33_554_432) // 32 MiB
            .spawn(move || {
                process_input(backend_cfg, device, sender_fromthread, receiver_tothread)
            })
            .unwrap();

        Ok(InputProcesser {
            device: RefCell::new(device),
            backend_cfg,
            thread,
            sender_fromthread: Arc::new(sender_fromthread),
            receiver_fromthread,
            sender_tothread,
            receiver_tothread: Arc::new(receiver_tothread),
        })
    }

    pub fn from_device_contact(
        device_contact: DeviceContact,
        res: Resolution,
        fps: u32,
        cfg: BackendConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let device =
            PossibleDevice::from_device_contact(device_contact, res, fps, DeviceFormat::MJpeg);
        InputProcesser::new(device, cfg)
    }

    pub fn change_device(
        &self,
        new_device: PossibleDevice,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.device.replace(new_device.clone());
        self.sender_tothread
            .send(MessageType::SetDevice{
                name: None,
                device: new_device,

            });
        Ok(())
    }

    pub fn query_gotten_results(&self) -> Vec<FullyCalculatedPacket> {
        let mut point_vec = Vec::new();
        for point in self.receiver_fromthread.drain() {
            // lmao imagine using a blocking function in a loop that waits until everything has been dropped, couldn't be me
            point_vec.push(point);
        }
        point_vec
    }
}

fn process_input(
    cfg: BackendConfig,
    device: PossibleDevice,
    sender: Arc<Sender<FullyCalculatedPacket>>,
    message: Arc<Receiver<MessageType>>,
) -> u8 {
    let processor = FaceProcessorBuilder::new()
        .with_backend(cfg.backend_as_facial())
        .with_input(cfg.res().x, cfg.res().y)
        .build()
        .unwrap();
    let init_res = device.res();
    let init_fps = device.fps();
    let mut device = match get_dyn_webcam(None, device) {
        Ok(webcam) => webcam,
        Err(_) => return -1,
    };

    loop {
        if let Ok(msg_recv) = message.try_recv() {
            match msg_recv {
                MessageType::Die(code) => {
                    return code;
                }
                MessageType::SetDevice(new_dev) => {
                    device = match get_dyn_webcam(None, new_dev) {
                        Ok(webcam) => webcam,
                        Err(_) => return -1,
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
        let frame = match device.get_frame() {
            Ok(f) => f,
            Err(_) => {
                return -1;
            }
        };
        let res = device.get_resolution().unwrap();
        let image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(res.x, res.y, frame).unwrap();

        // detections
        let bbox = processor.calculate_face_bboxes(&image);
        if bbox.len() <= 0 {
            continue;
        }
        let face_landmarks = processor.calculate_landmark(&image, *bbox.get(0).unwrap());
        let eyes = processor.calculate_eyes(face_landmarks.clone(), &image);
        let pnp = processor
            .calculate_pnp(&image, face_landmarks.clone())
            .unwrap();
        sender.send(FullyCalculatedPacket {
            landmarks: face_landmarks,
            euler: pnp,
            eye_positions: eyes,
        })
    }
}

fn get_dyn_webcam<'a>(
    name: Option<String>,
    device: PossibleDevice,
) -> Result<Box<dyn Webcam<'a> + 'a>, Box<dyn std::error::Error>> {
    let device_held: Box<dyn Webcam<'a>> = match device {
        PossibleDevice::UVCam {
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
        PossibleDevice::OpenCV {
            index: _index,
            res,
            fps,
            fmt: _fmt,
        } => {
            let ocvcam = match OpenCvCameraDevice::from_possible_device(
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
