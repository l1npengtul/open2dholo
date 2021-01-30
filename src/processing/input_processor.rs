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

use crate::processing::face_detector::facial_existance::FacialDetector;
use crate::{
    error::processing_error::ProcessingError,
    util::{
        camera::device_utils::{PathIndex, PossibleDevice, Resolution},
        packet::{MessageType, Processed, ProcessedPacket},
    },
};
use dlib_face_recognition::{
    FaceDetector, FaceDetectorTrait, ImageMatrix, LandmarkPredictor, LandmarkPredictorTrait,
};
use flume::{Receiver, Sender};
use gdnative::godot_print;
use scheduled_thread_pool::ScheduledThreadPool;
use std::path::Path;
use std::{
    sync::{atomic::AtomicUsize, Arc},
    thread::{Builder, JoinHandle},
    time::Duration,
};
use uvc::Device as UVCDevice;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::{
    io::mmap::Stream,
    video::{capture::Parameters, traits::Capture},
    Device, Format, FourCC,
};

pub struct InputProcessing {
    // To Thread
    sender_p1: Sender<MessageType>,
    // From Thread
    reciever_p2: Receiver<Processed>,
    // thread
    _thread_handle: JoinHandle<Result<(), Box<ProcessingError>>>,
}

impl InputProcessing {
    pub fn new(device: PossibleDevice) -> Result<Self, ()> {
        let (to_thread_tx, to_thread_rx) = flume::unbounded();
        let (from_thread_tx, from_thread_rx) = flume::unbounded();
        godot_print!("a");
        let thread = match Builder::new()
            .name(format!("input-processor-senpai_{}", 1))
            .spawn(move || input_process_func(to_thread_rx, from_thread_tx, device))
        {
            Ok(join) => join,
            Err(_why) => {
                return Err(());
            }
        };
        Ok(InputProcessing {
            sender_p1: to_thread_tx,
            // To Thread
            reciever_p2: from_thread_rx,
            // From Thread
            _thread_handle: thread,
        })
    }

    //pub fn get_output_handler
    pub fn kill(&mut self) {
        if self.sender_p1.send(MessageType::Die(0)).is_err() {
            // /shrug if this fails to send we're fucked
        }
    }

    pub fn get_thread_output(&self) -> Receiver<Processed> {
        self.reciever_p2.clone()
    }
}

impl Drop for InputProcessing {
    fn drop(&mut self) {
        self.kill()
    }
}

fn input_process_func(
    recv: Receiver<MessageType>,
    send: Sender<Processed>,
    startup_dev: PossibleDevice,
) -> Result<(), Box<ProcessingError>> {
    std::thread::sleep(Duration::from_millis(100));
    let mut face_detector = match FacialDetector::new(
        "deploy.prototxt",
        "res10_300x300_ssd_iter_140000.caffemodel",
    ) {
        Ok(face) => face,
        Err(_why) => {
            panic!("could not get DNN!")
        }
    };

    match startup_dev {
        PossibleDevice::UVCAM {
            vendor_id,
            product_id,
            serial,
            res,
            fps,
            fmt,
        } => {
            let uvc_device = match make_uvc_device(vendor_id, product_id, serial) {
                Ok(d) => d,
                Err(why) => {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Cannot open device: {}",
                        why.to_string()
                    ))))
                }
            };

            let device_handler = match uvc_device.open() {
                Ok(h) => h,
                Err(why) => {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Cannot open device: {}",
                        why.to_string()
                    ))))
                }
            };

            let mut stream_handler = match device_handler
                .get_stream_handle_with_format_size_and_fps(fmt, res.x, res.y, fps)
            {
                Ok(s) => s,
                Err(why) => {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Cannot open device: {}",
                        why.to_string()
                    ))))
                }
            };

            // face_detector.detect_face(
            //     frame.height() as i32,
            //     frame.width() as i32,
            //     frame.to_rgb().unwrap().to_bytes(),
            // );

            let (img_send, img_recv) = flume::unbounded();
            let cnt = Arc::new(AtomicUsize::new(0));

            let stream = match stream_handler.start_stream(
                move |frame, _count| {
                    // aaaa go crazy
                    let img_data = frame.to_rgb().unwrap().to_bytes().to_vec();
                    img_send.send(img_data);
                },
                cnt,
            ) {
                Ok(a) => a,
                Err(why) => {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Cannot open device: {}",
                        why.to_string()
                    ))))
                }
            };

            loop {
                if let Ok(img) = img_recv.try_recv() {
                    face_detector.detect_face(res.x, res.y, img.as_slice());
                }
                if let Ok(message) = recv.try_recv() {
                    match message {
                        MessageType::Die(_) | MessageType::Close(_) => {
                            stream.stop();
                            return Ok(());
                        }
                        _ => continue,
                    }
                }
            }
        }
        PossibleDevice::V4L2 {
            location,
            res,
            fps,
            fmt,
        } => {
            let mut v4l_device = match make_v4l_device(&location, res, fps, fmt) {
                Ok(d) => {
                    godot_print!("b");
                    d
                }
                Err(why) => {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Cannot open device: {}",
                        why.to_string()
                    ))))
                }
            };

            // godot_print!("c");

            let mut stream = match Stream::with_buffers(&v4l_device, Type::VideoCapture, 4) {
                Ok(s) => s,
                Err(why) => {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Cannot open device: {}",
                        why.to_string()
                    ))))
                }
            };

            // main loop with processing
            loop {
                if let Ok(message) = recv.try_recv() {
                    match message {
                        MessageType::Set(possible) => {
                            if let PossibleDevice::V4L2 {
                                location,
                                res,
                                fps: framerate,
                                fmt,
                            } = possible
                            {
                                match make_v4l_device(&location, res, framerate, fmt) {
                                    Ok(d) => {
                                        v4l_device = d;
                                        stream = match Stream::with_buffers(
                                            &v4l_device,
                                            Type::VideoCapture,
                                            4,
                                        ) {
                                            Ok(s) => s,
                                            Err(why) => {
                                                return Err(Box::new(ProcessingError::General(
                                                    format!(
                                                        "Cannot open device: {}",
                                                        why.to_string()
                                                    ),
                                                )))
                                            }
                                        };
                                    }
                                    Err(why) => {
                                        return Err(Box::new(ProcessingError::General(format!(
                                            "Cannot open device: {}",
                                            why.to_string()
                                        ))))
                                    }
                                }
                            }
                        }
                        _ => return Ok(()),
                    }
                }

                //let mut cnt = Arc::new(AtomicUsize::new(0));

                if let Ok((buffer, _meta)) = stream.next() {
                    face_detector.detect_face(res.x, res.y, buffer);
                // aaa go crazy
                } else {
                    return Err(Box::new(ProcessingError::General(
                        "Error capturing V4L2 buffer".to_string(),
                    )));
                }
            }
        }
    };
}

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

fn find_landmarks(img: &ImageMatrix) -> ProcessedPacket {
    let faces = FaceDetector::new().face_locations(img).to_vec();
    let faces2 = faces;
    let largest_rectangle = if let Some(rt) = faces2.get(0) {
        rt
    } else {
        godot_print!("no face!");
        return ProcessedPacket::None;
    };

    // Get facial landmarks
    let filename = "mmod_human_face_detector.dat";
    let landmark_getter = match LandmarkPredictor::new(filename) {
        Ok(detector) => detector,
        Err(_why) => {
            godot_print!("no file!");
            return ProcessedPacket::MissingFileError(String::from(filename));
        }
    };

    let landmarks = landmark_getter
        .face_landmarks(img, largest_rectangle)
        .to_vec();
    if landmarks.len() == 68 {
        ProcessedPacket::FacialLandmark(Processed::new(landmarks))
    } else {
        ProcessedPacket::MissingFacialPointsError(AtomicUsize::from(landmarks.len()))
    }
}

fn find_landmark_tf() {}
