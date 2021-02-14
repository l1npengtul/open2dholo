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

use crate::processing::face_detector::detectors::dlib::dlib_detector::DLibDetector;
use crate::processing::face_detector::detectors::util::{
    DetectorHardware, DetectorTrait, DetectorType, Rect,
};
use crate::util::camera::camera_device::{UVCameraDevice, V4LinuxDevice};
use crate::util::camera::device_utils::{DeviceFormat, StreamType};
use crate::util::camera::webcam::{Webcam, WebcamType};
use crate::util::packet::ProcessPacket;
use crate::{
    error::processing_error::ProcessingError,
    util::{
        camera::device_utils::{PathIndex, PossibleDevice, Resolution},
        packet::{MessageType, Processed},
    },
};
use dlib_face_recognition::{
    FaceDetector, FaceDetectorTrait, ImageMatrix, LandmarkPredictor, LandmarkPredictorTrait,
};
use flume::{Receiver, Sender};
use gdnative::godot_print;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::path::Path;
use std::thread::Thread;
use std::{
    sync::{atomic::AtomicUsize, Arc},
    thread::{Builder, JoinHandle},
    time::Duration,
};
use suspend::{Listener, Notifier, Suspend};
use uvc::Device as UVCDevice;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::{
    io::mmap::Stream,
    video::{capture::Parameters, traits::Capture},
    Device, Format, FourCC,
};

// pub struct InputProcessing<'a> {
//     // To Thread
//     sender_p1: Sender<MessageType<'a>>,
//     // From Thread
//     reciever_p2: Receiver<Processed>,
//     // thread
//     _thread_handle: JoinHandle<Result<(), Box<ProcessingError>>>,
// }
//
// impl<'a> InputProcessing<'a> {
//     pub fn new(device: PossibleDevice) -> Result<Self, ()> {
//         let (to_thread_tx, to_thread_rx) = flume::unbounded();
//         let (from_thread_tx, from_thread_rx) = flume::unbounded();
//         godot_print!("a");
//         let thread = match Builder::new()
//             .name(format!("input-processor-senpai_{}", 1))
//             .spawn(move || input_process_func(to_thread_rx, from_thread_tx, device))
//         {
//             Ok(join) => join,
//             Err(_why) => {
//                 return Err(());
//             }
//         };
//         Ok(InputProcessing {
//             sender_p1: to_thread_tx,
//             // To Thread
//             reciever_p2: from_thread_rx,
//             // From Thread
//             _thread_handle: thread,
//         })
//     }
//
//     //pub fn get_output_handler
//     pub fn kill(&mut self) {
//         if self.sender_p1.send(MessageType::Die(0)).is_err() {
//             // ¯\_(ツ)_/¯ if this fails to send we're fucked
//         }
//     }
//
//     pub fn get_thread_output(&self) -> Receiver<Processed> {
//         self.reciever_p2.clone()
//     }
// }
//
// impl Drop for InputProcessing {
//     fn drop(&mut self) {
//         self.kill()
//     }
// }
//
// // Welcome to function hell. I hope you enjoy your stay.
// // l1npengtul is not responsible for brain damage, eye damage, or any other calamity/damage that may result from reading this funtion.
// // You have been warned. Sorry to whoever has to read this.
// fn input_process_func(
//     recv: Receiver<MessageType>,
//     send: Sender<Processed>,
//     startup_dev: PossibleDevice,
// ) -> Result<(), Box<ProcessingError>> {
//     std::thread::sleep(Duration::from_millis(10));
//
//     // start up facial detector
//     let face_detector = DLibDetector::new(false);
//
//     match startup_dev {
//         PossibleDevice::UVCAM {
//             vendor_id,
//             product_id,
//             serial,
//             res,
//             fps,
//             fmt,
//         } => {
//             let uvc_device = match make_uvc_device(vendor_id, product_id, serial) {
//                 Ok(d) => d,
//                 Err(why) => {
//                     return Err(Box::new(ProcessingError::General(format!(
//                         "Cannot open device: {}",
//                         why.to_string()
//                     ))))
//                 }
//             };
//
//             let device_handler = match uvc_device.open() {
//                 Ok(h) => h,
//                 Err(why) => {
//                     return Err(Box::new(ProcessingError::General(format!(
//                         "Cannot open device: {}",
//                         why.to_string()
//                     ))))
//                 }
//             };
//
//             let mut stream_handler = match device_handler
//                 .get_stream_handle_with_format_size_and_fps(fmt, res.x, res.y, fps)
//             {
//                 Ok(s) => s,
//                 Err(why) => {
//                     return Err(Box::new(ProcessingError::General(format!(
//                         "Cannot open device: {}",
//                         why.to_string()
//                     ))))
//                 }
//             };
//
//             let (img_send, img_recv) = flume::unbounded();
//             let cnt = Arc::new(AtomicUsize::new(0));
//
//             let stream = match stream_handler.start_stream(
//                 move |frame, count| {
//                     // aaaa go crazy
//                     let img_data = frame.to_rgb().unwrap().to_bytes().to_vec();
//                     img_send.send(img_data);
//                 },
//                 cnt,
//             ) {
//                 Ok(a) => a,
//                 Err(why) => {
//                     return Err(Box::new(ProcessingError::General(format!(
//                         "Cannot open device: {}",
//                         why.to_string()
//                     ))))
//                 }
//             };
//
//             loop {
//                 if let Ok(img) = img_recv.try_recv() {
//                     for f in face_detector.detect_face_rects(res.y, res.x, img.as_slice()) {
//                         godot_print!("{},{} {},{}", f.x1(), f.y1(), f.x2(), f.y2());
//                     }
//                 }
//                 if let Ok(message) = recv.try_recv() {
//                     match message {
//                         MessageType::Die(_) | MessageType::Close(_) => {
//                             stream.stop();
//                             return Ok(());
//                         }
//                         _ => continue,
//                     }
//                 }
//             }
//         }
//         PossibleDevice::V4L2 {
//             location,
//             res,
//             fps,
//             fmt,
//         } => {
//             let mut v4l_device = match make_v4l_device(&location, res, fps, fmt) {
//                 Ok(d) => {
//                     godot_print!("b");
//                     d
//                 }
//                 Err(why) => {
//                     return Err(Box::new(ProcessingError::General(format!(
//                         "Cannot open device: {}",
//                         why.to_string()
//                     ))))
//                 }
//             };
//
//             // godot_print!("c");
//
//             let mut stream = match Stream::with_buffers(&v4l_device, Type::VideoCapture, 4) {
//                 Ok(s) => s,
//                 Err(why) => {
//                     return Err(Box::new(ProcessingError::General(format!(
//                         "Cannot open device: {}",
//                         why.to_string()
//                     ))))
//                 }
//             };
//
//             // main loop with processing
//             loop {
//                 if let Ok(message) = recv.try_recv() {
//                     match message {
//                         MessageType::Set(possible) => {
//                             if let PossibleDevice::V4L2 {
//                                 location,
//                                 res,
//                                 fps: framerate,
//                                 fmt,
//                             } = possible
//                             {
//                                 match make_v4l_device(&location, res, framerate, fmt) {
//                                     Ok(d) => {
//                                         v4l_device = d;
//                                         stream = match Stream::with_buffers(
//                                             &v4l_device,
//                                             Type::VideoCapture,
//                                             4,
//                                         ) {
//                                             Ok(s) => s,
//                                             Err(why) => {
//                                                 return Err(Box::new(ProcessingError::General(
//                                                     format!(
//                                                         "Cannot open device: {}",
//                                                         why.to_string()
//                                                     ),
//                                                 )))
//                                             }
//                                         };
//                                     }
//                                     Err(why) => {
//                                         return Err(Box::new(ProcessingError::General(format!(
//                                             "Cannot open device: {}",
//                                             why.to_string()
//                                         ))))
//                                     }
//                                 }
//                             }
//                         }
//                         _ => return Ok(()),
//                     }
//                 }
//
//                 //let mut cnt = Arc::new(AtomicUsize::new(0));
//
//                 if let Ok((buffer, _meta)) = stream.next() {
//                     for f in face_detector.detect_face_rects(res.y, res.x, buffer) {
//                         godot_print!("{},{} {},{}", f.x1(), f.y1(), f.x2(), f.y2());
//                     }
//                 // aaa go crazy
//                 } else {
//                     return Err(Box::new(ProcessingError::General(
//                         "Error capturing V4L2 buffer".to_string(),
//                     )));
//                 }
//             }
//         }
//     };
// }

fn make_v4l_device(
    location: &PathIndex,
    res: Resolution,
    fps: u32,
    fmt: FourCC,
) -> Result<Device, Box<dyn std::error::Error>> {
    let device = match location {
        PathIndex::Path(path) => {
            let dev = match Device::with_path(Path::new(path)) {
                Ok(d) => d,
                Err(why) => return Err(Box::new(why)),
            };
            dev
        }
        PathIndex::Index(idx) => {
            let dev = match Device::new(*idx) {
                Ok(d) => d,
                Err(why) => return Err(Box::new(why)),
            };
            dev
        }
    };

    let fcc = fmt;

    let format = match device.format() {
        Ok(mut f) => {
            f.width = res.x;
            f.height = res.y;
            f.fourcc = fcc;
            f
        }
        Err(_) => Format::new(res.x, res.y, fcc),
    };

    let param = Parameters::with_fps(fps);

    if let Err(why) = device.set_format(&format) {
        return Err(Box::new(why));
    }
    if let Err(why) = device.set_params(&param) {
        return Err(Box::new(why));
    }

    Ok(device)
}

fn make_uvc_device<'a>(
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    serial: Option<String>,
) -> Result<UVCDevice<'a>, Box<dyn std::error::Error>> {
    let device = match crate::UVC.find_device(
        vendor_id.map(i32::from),
        product_id.map(i32::from),
        serial.as_deref(),
    ) {
        Ok(d) => d,
        Err(why) => return Err(Box::new(why)),
    };
    Ok(device)
}

pub struct InputProcessingThreadless<'a> {
    // device: PossibleDevice,
    pub device_held: RefCell<Box<dyn Webcam<'a> + 'a>>, // bruh wtf
    detector_type: Cell<DetectorType>,
    detector_hw: Cell<DetectorHardware>,
    face_detector: RefCell<Box<dyn DetectorTrait>>,
}

impl<'a> InputProcessingThreadless<'a> {
    pub fn new(
        device: PossibleDevice,
        detect_typ: DetectorType,
        detect_hw: DetectorHardware,
    ) -> Self {
        let device_held: RefCell<Box<dyn Webcam>> = match device {
            PossibleDevice::UVCAM {
                vendor_id,
                product_id,
                serial,
                res,
                fps,
                fmt: _fmt,
            } => {
                let dev: UVCameraDevice<'a> = match make_uvc_device(vendor_id, product_id, serial) {
                    Ok(d) => {
                        let uvc_dev: UVCameraDevice<'a> = match UVCameraDevice::from_device(d) {
                            Ok(u) => u,
                            Err(why) => {
                                panic!("{}", why.to_string())
                            }
                        };
                        uvc_dev.set_camera_format(DeviceFormat::MJPEG);
                        uvc_dev.set_framerate(&fps);
                        uvc_dev.set_resolution(&res);
                        uvc_dev
                    }
                    Err(why) => {
                        panic!("{}", why.to_string());
                    }
                };
                RefCell::new(Box::new(dev))
            }
            PossibleDevice::V4L2 {
                location,
                res,
                fps,
                fmt: _fmt,
            } => {
                let dev = match V4LinuxDevice::new_location(location) {
                    Ok(d) => d,
                    Err(_why) => {
                        panic!("Could not get V4L Device!");
                    }
                };
                dev.set_resolution(&res);
                dev.set_framerate(&fps);
                dev.set_camera_format(DeviceFormat::MJPEG);
                RefCell::new(Box::new(dev))
            }
        };

        let detector_type = Cell::new(detect_typ);
        let detector_hw = Cell::new(detect_hw);

        let face_detector = RefCell::new(Box::new(match detect_typ {
            DetectorType::DLibFHOG => DLibDetector::new(false),
            DetectorType::DLibCNN => DLibDetector::new(true),
        }));

        InputProcessingThreadless {
            device_held,
            detector_type,
            detector_hw,
            face_detector,
        }
    }

    pub fn change_device(&self, new_device: PossibleDevice) {
        let device_held: Box<dyn Webcam> = match new_device {
            PossibleDevice::UVCAM {
                vendor_id,
                product_id,
                serial,
                res,
                fps,
                fmt: _fmt,
            } => {
                let mut dev: UVCameraDevice<'a> =
                    match make_uvc_device(vendor_id, product_id, serial) {
                        Ok(d) => {
                            let uvc_dev: UVCameraDevice<'a> = match UVCameraDevice::from_device(d) {
                                Ok(u) => u,
                                Err(why) => {
                                    panic!("{}", why.to_string())
                                }
                            };
                            uvc_dev.set_camera_format(DeviceFormat::MJPEG);
                            uvc_dev.set_framerate(&fps);
                            uvc_dev.set_resolution(&res);
                            uvc_dev
                        }
                        Err(why) => {
                            panic!("{}", why.to_string());
                        }
                    };
                Box::new(dev)
            }
            PossibleDevice::V4L2 {
                location,
                res,
                fps,
                fmt: _fmt,
            } => {
                let dev = match V4LinuxDevice::new_location(location) {
                    Ok(d) => d,
                    Err(_why) => {
                        panic!("Could not get V4L Device!");
                    }
                };
                dev.set_resolution(&res);
                dev.set_framerate(&fps);
                dev.set_camera_format(DeviceFormat::MJPEG);
                Box::new(dev)
            }
        };

        self.device_held.replace(device_held);
    }

    pub fn detect_faces(&self, img_height: u32, img_width: u32, img_data: &[u8]) -> Vec<Rect> {
        let a = &*self.face_detector.borrow();
        a.detect_face_rects(img_height, img_width, img_data)
    }
}

pub struct ThreadedWorker<EMILIA, MAJITENSHI> {
    // degenerate generic tag go brrrrrr
    // readability go *adios*
    thread_handle: JoinHandle<_>,
    func: Box<dyn Fn(Sender<EMILIA>, Receiver<MAJITENSHI>, Listener) + Sync + Send>,
    recv: Receiver<MAJITENSHI>,
    int_sender: Sender<EMILIA>,
    int_recv: Receiver<EMILIA>,
    suspend: Suspend,
    notiy: Notifier,
}

impl ThreadedWorker<EMILIA, MAJITENSHI> {
    pub fn new(
        func: Box<dyn Fn(Sender<EMILIA>, Receiver<MAJITENSHI>) + Sync + Send>,
        recv: Receiver<MAJITENSHI>,
        int_send: Sender<EMILIA>,
        int_recv: Receiver<EMILIA>,
        thread_name: String,
    ) -> Self {
        let thread_handle = match Builder::new().name(thread_name).spawn(func) {};
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
