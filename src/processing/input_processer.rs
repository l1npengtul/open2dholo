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
use std::{
    sync::{atomic::AtomicUsize, Arc},
    thread::{Builder, JoinHandle},
    time::Duration,
};
use uvc::Device;
use v4l::{
    buffer::Stream, capture::Parameters, prelude::CaptureDevice, prelude::MmapStream, Format,
    FourCC,
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
    pub fn kill(&self) {
        if let Err(_) = self.sender_p1.send(MessageType::Die(0)) {
            // /shrug if this fails to send we're fucked
        }
    }

    pub fn get_thread_output(&self) -> Receiver<Processed> {
        return self.reciever_p2.clone();
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
    // std::thread::sleep(Duration::from_millis(100));

    let thread_pool = ScheduledThreadPool::with_name("input_processer-{}", 2); // Use num_threads from processing
    godot_print!("thread");

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

            let cnt = Arc::new(AtomicUsize::new(0));

            let stream = match stream_handler.start_stream(
                move |frame, _count| {
                    // aaaa go crazy
                    let image = unsafe {
                        ImageMatrix::new(
                            frame.width() as usize,
                            frame.height() as usize,
                            frame.to_rgb().unwrap().to_bytes().as_ptr(),
                        )
                    };
                    let cloned_send = send.clone();

                    thread_pool.execute(move || {
                        match find_landmarks(&image) {
                            ProcessedPacket::FacialLandmark(landmarks) => {
                                // stonks
                                if let Err(_) = cloned_send.send(landmarks) {
                                    () // ooh yeah i care about errors
                                }
                            }
                            ProcessedPacket::None => {
                                std::thread::sleep(Duration::from_millis(50));
                            }
                            ProcessedPacket::GeneralError(_)
                            | ProcessedPacket::MissingFacialPointsError(_)
                            | ProcessedPacket::MissingFileError(_) => {}
                        }
                    });
                },
                cnt.clone(),
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
                if let Ok(message) = recv.try_recv() {
                    match message {
                        MessageType::Die(_) | MessageType::Close(_) => {
                            stream.stop();
                            return Ok(());
                        }
                        _ => continue,
                    }
                }
                std::thread::sleep(Duration::from_millis(50))
            }
        }
        PossibleDevice::V4L2 {
            location,
            res,
            fps,
            fmt,
        } => {
            let mut v4l_device = match make_v4l_device(&location, &res, &fps, &fmt) {
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

            let mut stream = match MmapStream::new(&v4l_device) {
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
                                fps,
                                fmt,
                            } = possible
                            {
                                match make_v4l_device(&location, &res, &fps, &fmt) {
                                    Ok(d) => {
                                        v4l_device = d;
                                        stream = match MmapStream::new(&v4l_device) {
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

                if let Ok(buffer) = stream.next() {
                    let image = unsafe {
                        ImageMatrix::new(res.x as usize, res.y as usize, buffer.as_ptr())
                    };
                    // aaa go crazy
                    let cloned_send = send.clone();
                    thread_pool.execute(move || {
                        match find_landmarks(&image) {
                            ProcessedPacket::FacialLandmark(landmarks) => {
                                // stonks
                                if let Err(_) = cloned_send.send(landmarks) {
                                    () // ooh yeah i care about errors
                                }
                            }
                            ProcessedPacket::None => {
                                std::thread::sleep(Duration::from_millis(50));
                            }
                            ProcessedPacket::GeneralError(_)
                            | ProcessedPacket::MissingFacialPointsError(_)
                            | ProcessedPacket::MissingFileError(_) => {}
                        }
                    });
                } else {
                    return Err(Box::new(ProcessingError::General(format!(
                        "Error capturing V4L2 buffer"
                    ))));
                }
            }
        }
    };
}

fn make_v4l_device(
    location: &PathIndex,
    res: &Resolution,
    fps: &u32,
    fmt: &FourCC,
) -> Result<CaptureDevice, Box<dyn std::error::Error>> {
    let mut device = match location {
        PathIndex::Path(path) => {
            let device = match CaptureDevice::with_path(path) {
                Ok(d) => d,
                Err(why) => return Err(Box::new(why)),
            };
            device
        }
        PathIndex::Index(idx) => {
            let device = match CaptureDevice::new(idx.clone()) {
                Ok(d) => d,
                Err(why) => return Err(Box::new(why)),
            };
            device
        }
    };

    let fcc = fmt.clone();

    let format = match device.format() {
        Ok(mut f) => {
            f.width = res.x;
            f.height = res.y;
            f.fourcc = fcc;
            f
        }
        Err(_) => Format::new(res.x, res.y, fcc),
    };

    let param = Parameters::with_fps(fps.clone());

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
) -> Result<Device<'a>, Box<dyn std::error::Error>> {
    let device = match crate::UVC.find_device(
        vendor_id.map_or(None, |f| Some(f as i32)),
        product_id.map_or(None, |f| Some(f as i32)),
        serial.as_deref(),
    ) {
        Ok(d) => d,
        Err(why) => return Err(Box::new(why)),
    };
    Ok(device)
}

fn find_landmarks(img: &ImageMatrix) -> ProcessedPacket {
    let faces = FaceDetector::new().face_locations(img).to_vec();
    let faces2 = faces.clone();
    let largest_rectangle = match faces2.get(0) {
        Some(rt) => rt,
        None => {
            return ProcessedPacket::None;
        }
    };

    // Get facial landmarks
    let filename = "mmod_human_face_detector.dat";
    let landmark_getter = match LandmarkPredictor::new(filename) {
        Ok(detector) => detector,
        Err(_why) => {
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
