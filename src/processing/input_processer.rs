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

//use flume::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};

use crate::util::device::DeviceDesc;
use crate::util::packet::{MessageType, Processed, ProcessedPacket};
use dlib_face_recognition::{
    FaceDetector, FaceDetectorTrait, ImageMatrix, LandmarkPredictor, LandmarkPredictorTrait,
};
use flume::{Receiver, Sender, TryRecvError};
use std::{
    sync::{atomic::AtomicUsize, Arc},
    thread::{Builder, JoinHandle},
    time::Duration,
};

pub struct InputProcessing {
    device: DeviceDesc,
    format: uvc::StreamFormat,
    sender_p1: Sender<MessageType>,
    // To Thread
    reciever_p2: Receiver<ProcessedPacket>,
    // From Thread
    thread_handle: JoinHandle<u8>,
}

impl InputProcessing {
    pub fn new(
        num_thread: usize,
        bind_device: uvc::DeviceDescription,
        stream_fmt: uvc::StreamFormat,
    ) -> Self {
        let (to_thread_tx, to_thread_rx) = flume::unbounded();
        let (from_thread_tx, from_thread_rx) = flume::unbounded();
        let description = DeviceDesc::from_description(bind_device);
        let description2 = description.clone();
        let thread = Builder::new()
            .name(format!("input-processor_{}", num_thread))
            .spawn(move || {
                input_process_func(
                    to_thread_rx,
                    from_thread_tx,
                    description,
                    stream_fmt.clone(),
                )
            })
            .unwrap();
        InputProcessing {
            device: description2,
            format: stream_fmt,
            sender_p1: to_thread_tx,
            reciever_p2: from_thread_rx,
            thread_handle: thread,
        }
    }
    pub fn fichange_device(&self, device: DeviceDesc) -> Result<(), ()> {
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

/*
 * Thread Return Codes - VERY
 *
 * 0 = Sucessful Exit
 * 1 = Device not found
 * 2 = General Error
 * 3 = Thread Communication Error
 * 4 = Threadpool Error
 */

fn input_process_func(
    _recv: Receiver<MessageType>,
    send: Sender<ProcessedPacket>,
    startup_desc: DeviceDesc,
    startup_format: uvc::StreamFormat,
) -> u8 {
    std::thread::sleep(Duration::from_millis(100));
    let device_serial = match startup_desc.ser {
        Some(serial) => Some(serial),
        None => None,
    };
    let current_format = startup_format;
    let current_device: uvc::Device = match crate::UVC.find_device(
        startup_desc.vid,
        startup_desc.pid,
        device_serial.as_deref(),
    ) {
        Ok(v) => v,
        Err(_why) => {
            return 1;
        }
    };
    //let threads = (1000 / current_format.fps) + 1;

    // The AtomicUsize limits us to around 136.1 years of webcam streaming on a 32-bit systems, or
    // 584542046090.6 years on a 64-bit systems. The queen of the UK will still be alive by the time the 
    // counter overflows and the program crashes.
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_send = send.clone();
    current_device
        .open()
        .unwrap()
        .get_stream_handle_with_format(current_format)
        .unwrap()
        .start_stream(
            move |frame, _count| {
                let img_matrix = unsafe {
                    ImageMatrix::new(
                        frame.width() as usize,
                        frame.height() as usize,
                        frame.to_rgb().unwrap().to_bytes().as_ptr(),
                    )
                };
                crate::PROCESSING_POOL
                    .install(|| cloned_send.send(find_landmarks(&img_matrix)).unwrap())
            },
            counter.clone(),
        )
        .expect("Could not start stream!");
    0
}

fn trick_or_channel(message: Result<MessageType, TryRecvError>) -> DeviceOrTrick {
    match message {
        Ok(msg) => match msg {
            MessageType::Set(dev) => DeviceOrTrick::Device(dev),
            MessageType::Close(code) => DeviceOrTrick::Exit(code),
            MessageType::Die(code) => DeviceOrTrick::Exit(code),
        },
        Err(e) => match e {
            TryRecvError::Disconnected => DeviceOrTrick::Exit(3),
            TryRecvError::Empty => DeviceOrTrick::None,
        },
    }
}

// Trick or treat with death and webcams. Fun for the whole family!
enum DeviceOrTrick {
    Device(DeviceDesc),
    Exit(u8),
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
