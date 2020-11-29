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

use crate::{error::processing_error::ProcessingError, util::{camera::{camera_device::{V4LinuxDevice, UVCameraDevice}, device_utils::{DeviceDesc, PossibleDevice}, webcam::Webcam}, packet::{MessageType, Processed, ProcessedPacket}}};
use dlib_face_recognition::{
    FaceDetector, FaceDetectorTrait, ImageMatrix, LandmarkPredictor, LandmarkPredictorTrait,
};
use flume::{Receiver, Sender, TryRecvError};
use std::{sync::{Arc, atomic::AtomicUsize}, thread::{Builder, JoinHandle}, time::Duration};
use downcast_rs::*;

pub struct InputProcessing {
    sender_p1: Sender<MessageType>,
    // To Thread
    reciever_p2: Receiver<ProcessedPacket>,
    // From Thread
    thread_handle: JoinHandle<Result<(), Box<ProcessingError>>>,
}

impl InputProcessing {
    pub fn new(
        num_thread: usize,
        device: Box<dyn Webcam + Sync + Send>,
    ) -> Result<Self, ()> {
        let (to_thread_tx, to_thread_rx) = flume::unbounded();
        let (from_thread_tx, from_thread_rx) = flume::unbounded();
        let thread = match Builder::new()
            .name(format!("input-processor_{}", num_thread))
            .spawn(move || {
                input_process_func(
                    to_thread_rx,
                    from_thread_tx,
                    device
                )
            }) {
                Ok(join) => {
                    join
                }
                Err(_why) => {
                    return Err(());
                }
        };
        Ok(
            InputProcessing{
                sender_p1: to_thread_tx,
                // To Thread
                reciever_p2: from_thread_rx,
                // From Thread
                thread_handle: thread,
            }
        )
    }
    pub fn fichange_device(&self, device: PossibleDevice) -> Result<(), ()> {
        match self.sender_p1.send(MessageType::Set(device)) {
            Ok(_v) => Ok(()),
            Err(_e) => Err(()),
        }
    }

    //pub fn get_output_handler
    pub fn kill(&mut self) {
        unimplemented!()
    }
}

impl Drop for InputProcessing {
    fn drop(&mut self) {
        unimplemented!()
    }
}

fn input_process_func(
    recv: Receiver<MessageType>,
    send: Sender<ProcessedPacket>,
    startup_dev: Box<dyn Webcam>,
) -> Result<(), Box<ProcessingError>> {
    std::thread::sleep(Duration::from_millis(100));
    
    match startup_dev.get_inner(){
        PossibleDevice::UVCAM { vendor_id, product_id, serial, res, fps, fmt } => {
            let mut uvc_device = match crate::UVC.find_device(vendor_id.map_or(None, |f| Some(f as i32)), product_id.map_or(None, |f| Some(f as i32)), serial.as_deref()) {
                Ok(d) => d,
                Err(why) => return Err(Box::new(ProcessingError::General(format!("Cannot open device: {}", why.to_string()))))
            };

            let mut resolution = res;
            let mut frame_rate = fps;
            let mut format = fmt;

            loop {
                // get messages we may need to respond to
                if let Ok(message) = recv.try_recv() {
                    match message {
                        MessageType::Set(device) => {
                            if let PossibleDevice::UVCAM{ vendor_id, product_id, serial, res, fps, fmt } = device {
                                match crate::UVC.find_device(vendor_id.map_or(None, |f| Some(f as i32)), product_id.map_or(None, |f| Some(f as i32)), serial.as_deref()) {
                                    Ok(d) => {
                                        uvc_device = d;
                                        resolution = res;
                                        frame_rate = fps;
                                        format = fmt;
                                    },
                                    Err(why) => return Err(Box::new(ProcessingError::General(format!("Cannot open device: {}", why.to_string()))))
                                }
                            }
                        }
                        _ => return Err(Box::new(ProcessingError::General(format!("Thread Close/End request."))))
                    }

                    // acutal input stream here
                    let device_handler = match uvc_device.open() {
                        Ok(h) => h,
                        Err(why) => return Err(Box::new(ProcessingError::General(format!("Cannot open device handler: {}", why.to_string()))))
                    };

                    let mut stream_handler = match device_handler.get_stream_handle_with_format_size_and_fps(format, resolution.x, resolution.y, frame_rate) {
                        Ok(h) => h,
                        Err(why) => return Err(Box::new(ProcessingError::General(format!("Cannot open device stream handler: {}", why.to_string()))))
                    };

                    let cnt = Arc::new(AtomicUsize::new(0));

                    let _device_stream = match stream_handler.start_stream(|_frame, count| {
                        // aaaa go crazy
                    }, cnt.clone()) {
                        Ok(active) => active,
                        Err(why) => return Err(Box::new(ProcessingError::General(format!("Cannot open device stream: {}", why.to_string()))))
                    };
                }

            }
        }
        PossibleDevice::V4L2 { location, res, fps, fmt } => {

        }
    };

    Ok(())
}

fn trick_or_channel(message: Result<MessageType, TryRecvError>) -> DeviceOrTrick {
    match message {
        Ok(msg) => match msg {
            MessageType::Set(dev) => DeviceOrTrick::Device(dev),
            MessageType::Close(code) => DeviceOrTrick::Exit("CLOSE_REQUEST".to_string()),
            MessageType::Die(code) => DeviceOrTrick::Exit("DIE".to_string()),
        },
        Err(e) => match e {
            TryRecvError::Disconnected => DeviceOrTrick::Exit("PIPE_DISCONNECT".to_string()),
            TryRecvError::Empty => DeviceOrTrick::None,
        },
    }
}

// Trick or treat with death and webcams. Fun for the whole family!
enum DeviceOrTrick {
    Device(PossibleDevice),
    Exit(String),
    None,
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
        ProcessedPacket::FacialLandmark(Processed::new(landmarks, None))
    } else {
        ProcessedPacket::MissingFacialPointsError(AtomicUsize::from(landmarks.len()))
    }
}
