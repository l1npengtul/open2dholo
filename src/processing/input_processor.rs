use crate::{
    error::thread_send_message_error::ThreadSendMessageError,
    globalize_path, handle_boxerr, mat_init,
    util::{
        camera::{
            camera_device::{OpenCvCameraDevice, UVCameraDevice, V4LinuxDevice},
            device_utils::{DeviceConfig, DeviceContact, DeviceFormat, PossibleDevice, Resolution},
            webcam::Webcam,
        },
        misc::{BackendConfig, FullyCalculatedPacket, MessageType},
    },
    vector,
};
use cv_convert::TryFromCv;
use dlib_face_recognition::{
    FaceDetector, FaceDetectorTrait, ImageMatrix, LandmarkPredictor, LandmarkPredictorTrait,
};
use facial_processing::{
    error::FacialProcessingError,
    utils::{
        face::FaceLandmark,
        misc::{BoundingBox, EulerAngles, PnPArguments, Point2D},
    },
};
use flume::{Receiver, Sender};
use gdnative::godot_print;
use image::{ImageBuffer, Rgb};
use nalgebra::Matrix3;
use opencv::{
    calib3d::{
        rodrigues, rq_decomp3x3, solve_pnp, solve_pnp_ransac, SOLVEPNP_AP3P, SOLVEPNP_DLS,
        SOLVEPNP_EPNP, SOLVEPNP_IPPE, SOLVEPNP_IPPE_SQUARE, SOLVEPNP_ITERATIVE, SOLVEPNP_MAX_COUNT,
         SOLVEPNP_UPNP,
    },
    core::{
        Mat, MatExprTrait, Point2d, Point3d, ToInputArray, ToOutputArray, Vector,
         CV_64F,
    },
};
use std::{
    cell::{Cell, RefCell},
    line,
    thread::{Builder, JoinHandle},
};


pub struct InputProcesser {
    device: RefCell<PossibleDevice>,
    // bruh wtf
    backend_cfg: Cell<BackendConfig>,
    // face_detector: Arc<Mutex<Box<dyn DetectorTrait>>>,
    thread: JoinHandle<u8>,
    receiver_fromthread: Receiver<FullyCalculatedPacket>,
    sender_tothread: Sender<MessageType>,
}

impl InputProcesser {
    pub fn new(
        device: PossibleDevice,
        config: BackendConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backend_cfg = Cell::new(config);
        let (sender_fromthread, receiver_fromthread) = flume::unbounded();
        let (sender_tothread, receiver_tothread) = flume::unbounded();
        let cfg2 = backend_cfg.get();
        let dev2 = device.clone();

        let thread = Builder::new()
            .name("input_processor".to_string())
            .stack_size(33_554_432) // 32 MiB
            .spawn(move || process_input(cfg2, dev2, sender_fromthread, receiver_tothread))
            .unwrap();

        Ok(InputProcesser {
            device: RefCell::new(device),
            backend_cfg,
            thread,
            receiver_fromthread,
            sender_tothread,
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
        if self
            .sender_tothread
            .send(MessageType::SetDevice {
                name: None,
                device: new_device,
            })
            .is_err()
        {
            return Err(Box::new(ThreadSendMessageError::CannotSend));
        }
        Ok(())
    }

    pub fn set_device_cfg(&self, dev_cfg: DeviceConfig) -> Result<(), Box<dyn std::error::Error>> {
        let current_possible = self.device.borrow().clone().change_config(dev_cfg);
        self.device.replace(current_possible);
        if self
            .sender_tothread
            .send(MessageType::ChangeDevice(dev_cfg))
            .is_err()
        {
            return Err(Box::new(ThreadSendMessageError::CannotSend));
        }
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

    /// Get a reference to the input processer's backend cfg.
    pub fn backend_cfg(&self) -> &Cell<BackendConfig> {
        &self.backend_cfg
    }

    /// Get a reference to the input processer's thread.
    pub fn thread(&self) -> &JoinHandle<u8> {
        &self.thread
    }
}

fn process_input(
    _cfg: BackendConfig,
    device: PossibleDevice,
    sender: Sender<FullyCalculatedPacket>,
    message: Receiver<MessageType>,
) -> u8 {
    let init_res = device.res();
    let init_fps = device.fps();
    let face_detector = FaceDetector::new();
    let mut device = match get_dyn_webcam(Some("".to_string()), device) {
        Ok(webcam) => webcam,
        Err(_) => return 255,
    };
    let ld_detector = LandmarkPredictor::new(globalize_path!(
        "res://models/facial-processing-rs-models/shape_predictor_68_face_landmarks.dat"
    ))
    .unwrap();

    match device.open_stream() {
        Ok(_) => {}
        Err(why) => {
            godot_print!("died {}, {}", line!(), why.to_string());
        }
    };

    loop {
        if let Ok(msg_recv) = message.try_recv() {
            match msg_recv {
                MessageType::Die(code) => {
                    return code;
                }
                MessageType::SetDevice {
                    name,
                    device: new_dev,
                } => {
                    device = match get_dyn_webcam(name, new_dev) {
                        Ok(webcam) => webcam,
                        Err(why) => {
                            godot_print!("died {}, {}", line!(), why.to_string());
                            return 255;
                        }
                    };

                    match device.open_stream() {
                        Ok(_) => {}
                        Err(why) => {
                            godot_print!("died {}, {}", line!(), why.to_string());
                        }
                    }
                }
                MessageType::ChangeDevice(new_cfg) => {
                    let new_res = new_cfg.res;
                    let new_fps = new_cfg.fps;
                    if new_res != init_res {
                        handle_boxerr!(device.set_resolution(new_res), 253);
                    }
                    if new_fps != init_fps {
                        handle_boxerr!(device.set_framerate(new_fps), 253);
                    }
                }
            }
        }

        // get frame
        let mut frame_data = match device.get_frame() {
            Ok(f) => {
                godot_print!("framelen: {}", f.len());
                f
            }
            Err(why) => {
                godot_print!("died {}, {}", line!(), why.to_string());
                return 255;
            }
        };
        let res = device.get_resolution().unwrap();
        frame_data.resize((res.x * res.y * 3) as usize, 0_u8);

        let framebuf = match ImageBuffer::from_raw(res.x, res.y, frame_data) {
            Some(v) => {
                let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> = v;
                ImageMatrix::from_image(&img_buf)
            }
            None => {
                godot_print!("no frame");
                continue;
            }
        };

        for rect in face_detector.face_locations(&framebuf).iter() {
            let landmarks = ld_detector.face_landmarks(&framebuf, rect);

            let mut pt_vec = vec![];
            let mut point_vec = vec![];
            for lm_point in landmarks.iter() {
                pt_vec.push(Point2D {
                    x: lm_point.x() as f64,
                    y: lm_point.y() as f64,
                });
                point_vec.push(*lm_point)
            }

            let facelandmark = FaceLandmark::from_dlib(BoundingBox::from(*rect), point_vec);

            let pnp = EulerAngles {
                x: 0_f64,
                y: 0_f64,
                z: 0_f64,
            };



            if sender
                .send(FullyCalculatedPacket {
                    landmarks: pt_vec,
                    euler: pnp,
                })
                .is_err()
            {
                godot_print!("died {}", line!());
                return 254;
            }
        }
    }
}

fn get_dyn_webcam<'a>(
    name: Option<String>,
    device: PossibleDevice,
) -> Result<Box<dyn Webcam<'a> + 'a>, Box<dyn std::error::Error>> {
    let device_held: Box<dyn Webcam<'a>> = match device {
        PossibleDevice::UniversalVideoCamera {
            vendor_id,
            product_id,
            serial,
            res,
            fps,
            fmt: _fmt,
        } => {
            let uvcam: UVCameraDevice<'a> = match UVCameraDevice::new_camera(
                vendor_id.map(i32::from),
                product_id.map(i32::from),
                serial,
            ) {
                Ok(camera) => camera,
                Err(why) => {
                    return Err(why);
                }
            };
            handle_boxerr!(uvcam.set_framerate(fps));
            handle_boxerr!(uvcam.set_resolution(res));
            Box::new(uvcam)
        }
        PossibleDevice::Video4Linux2 {
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
            handle_boxerr!(v4lcam.set_resolution(res));
            handle_boxerr!(v4lcam.set_framerate(fps));
            Box::new(v4lcam)
        }
        PossibleDevice::OpenComVision {
            index: _index,
            res,
            fps,
            fmt: _fmt,
        } => {
            let ocvcam = match OpenCvCameraDevice::from_possible_device(
                name.unwrap_or_else(|| "OpenCV Camera".to_string()),
                device,
            ) {
                Ok(device) => device,
                Err(why) => {
                    return Err(why);
                }
            };
            handle_boxerr!(ocvcam.set_resolution(res));
            handle_boxerr!(ocvcam.set_framerate(fps));
            Box::new(ocvcam)
        }
    };

    Ok(device_held)
}

pub struct PnPSolver {
    face_3d: Vector<Point3d>,
    camera_res: Point2D,
    camera_distortion: Mat,
    camera_matrix: Mat,
    pnp_mode: i32,
    pnp_extrinsic: bool,
    pnp_args: PnPArguments,
}
impl PnPSolver {
    pub fn new(
        camera_res: Point2D,
        calc_mode: Option<i32>,
        pnp_args: PnPArguments,
    ) -> Result<Self, FacialProcessingError> {
        // Fake 3D Model definition
        let face_3d: Vector<Point3d> = vector![
            Point3d::new(0.0, 0.0, 0.0),          // Nose Tip
            Point3d::new(0.0, -330.0, -65.0),     // Chin
            Point3d::new(-225.0, 170.0, -135.0),  // Left corner left eye
            Point3d::new(225.0, 170.0, -135.0),   // Right corner right eye
            Point3d::new(-150.0, -150.0, -125.0), // Mouth Corner left
            Point3d::new(150.0, -150.0, -125.0)   // Mouth Corner right
        ];

        let focal_len = camera_res.x;
        let center = Point2D::new(camera_res.x / 2_f64, camera_res.y / 2_f64);
        let camera_matrix_na: Matrix3<f64> = Matrix3::from_row_slice(&[
            focal_len, 0.0, center.x, 0.0, focal_len, center.y, 0.0, 0.0, 1.0,
        ]);
        let camera_matrix = match Mat::try_from_cv(camera_matrix_na) {
            Ok(m) => m,
            Err(why) => {
                return Err(FacialProcessingError::InitializeError(why.to_string()));
            }
        };

        let camera_distortion = match Mat::zeros(4, 1, CV_64F) {
            Ok(mut m) => m.a(),
            Err(why) => {
                return Err(FacialProcessingError::InitializeError(why.to_string()));
            }
        };

        let pnp_mode = match calc_mode {
            Some(mode) => match mode {
                SOLVEPNP_AP3P | SOLVEPNP_DLS | SOLVEPNP_ITERATIVE | SOLVEPNP_IPPE
                | SOLVEPNP_IPPE_SQUARE | SOLVEPNP_MAX_COUNT | SOLVEPNP_EPNP | SOLVEPNP_UPNP => mode,
                _ => {
                    return Err(FacialProcessingError::InitializeError(format!(
                        "{} is not a valid PNP setting!",
                        mode
                    )))
                }
            },
            None => SOLVEPNP_EPNP,
        };

        Ok(PnPSolver {
            face_3d,
            camera_res,
            camera_distortion,
            camera_matrix,
            pnp_mode,
            pnp_extrinsic: false,
            pnp_args,
        })
    }

    pub fn raw_forward(
        &self,
        data: FaceLandmark,
    ) -> Result<(Vector<f64>, Vector<f64>), FacialProcessingError> {
        match &self.pnp_args {
            PnPArguments::NoRandsc => {
                let mut rvec = Vector::new();
                let mut tvec = Vector::new();
                godot_print!("3");

                let mut fp: Vector<Point2d> = Vector::new();
                for pt in data.pnp_landmarks().to_vec() {
                    fp.push(Point2D::into(pt))
                }
                godot_print!("3");

                godot_print!(
                    "{}: {:#?}",
                    &self.face_3d.as_slice().len(),
                    &self.face_3d.as_slice()
                );
                godot_print!("{}: {:#?}", &fp.as_slice().len(), &fp.as_slice());
                godot_print!("{:#?}", &self.camera_matrix);
                godot_print!("{:#?}", &self.camera_distortion);

                match solve_pnp(
                    &self.face_3d.input_array().unwrap(),
                    &fp.input_array().unwrap(),
                    &self.camera_matrix.input_array().unwrap(),
                    &self.camera_distortion.input_array().unwrap(),
                    &mut rvec.output_array().unwrap(),
                    &mut tvec.output_array().unwrap(),
                    self.pnp_extrinsic,
                    self.pnp_mode,
                ) {
                    Ok(b) => {
                        godot_print!("3");

                        if b {
                            return Ok((rvec, tvec));
                        }
                        Err(FacialProcessingError::InternalError(
                            "PnP Calculation failed".to_string(),
                        ))
                    }
                    Err(why) => Err(FacialProcessingError::InternalError(why.to_string())),
                }
            }
            PnPArguments::Randsc {
                iter,
                reproj,
                conf,
                inliner: _inliner,
            } => {
                let mut rvec = Vector::new();
                let mut tvec = Vector::new();
                let mut fp: Vector<Point2d> = Vector::new();
                for pt in data.pnp_landmarks().to_vec() {
                    fp.push(Point2D::into(pt))
                }
                let mut il = opencv::core::no_array().unwrap();
                match solve_pnp_ransac(
                    &self.face_3d.input_array().unwrap(),
                    &fp.input_array().unwrap(),
                    &self.camera_matrix.input_array().unwrap(),
                    &self.camera_distortion.input_array().unwrap(),
                    &mut rvec.output_array().unwrap(),
                    &mut tvec.output_array().unwrap(),
                    self.pnp_extrinsic,
                    *iter,
                    *reproj,
                    *conf,
                    &mut il.output_array().unwrap(),
                    SOLVEPNP_EPNP,
                ) {
                    Ok(b) => {
                        if b {
                            return Ok((rvec, tvec));
                        }
                        Err(FacialProcessingError::InternalError(
                            "PnP Calculation failed".to_string(),
                        ))
                    }
                    Err(why) => Err(FacialProcessingError::InternalError(why.to_string())),
                }
            }
        }
    }

    pub fn forward(&self, data: FaceLandmark) -> Result<EulerAngles, FacialProcessingError> {
        godot_print!("2");
        match self.raw_forward(data) {
            Ok((rvec, _tvec)) => {
                godot_print!("2");
                let mut dest = mat_init!();
                let mut jackobin = mat_init!();
                godot_print!("2");
                if let Err(why) = rodrigues(
                    &rvec.input_array().unwrap(),
                    &mut dest.output_array().unwrap(),
                    &mut jackobin.output_array().unwrap(),
                ) {
                    return Err(FacialProcessingError::InternalError(format!(
                        "Failed to calculate rodrigues: {}",
                        why.to_string()
                    )));
                }
                godot_print!("2");

                let mut mtx_r = mat_init!();
                let mut mtx_q = mat_init!();
                let mut qx = mat_init!();
                let mut qy = mat_init!();
                let mut qz = mat_init!();
                godot_print!("2");
                match rq_decomp3x3(
                    &dest.input_array().unwrap(),
                    &mut mtx_r.output_array().unwrap(),
                    &mut mtx_q.output_array().unwrap(),
                    &mut qx.output_array().unwrap(),
                    &mut qy.output_array().unwrap(),
                    &mut qz.output_array().unwrap(),
                ) {
                    Ok(rots) => Ok(EulerAngles::from(rots)),
                    Err(why) => Err(FacialProcessingError::InternalError(why.to_string())),
                }
            }
            Err(f) => Err(f),
        }
    }

    /// Get a reference to the pn p solver's camera res.
    pub fn camera_res(&self) -> &Point2D {
        &self.camera_res
    }

    /// Set the pn p solver's camera res.
    pub fn set_camera_res(&mut self, camera_res: Point2D) {
        self.camera_res = camera_res;
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
