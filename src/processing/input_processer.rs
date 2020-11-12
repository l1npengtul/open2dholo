//use flume::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};

use crate::{
    error::thread_send_message_error::ThreadSendMessageError,
    processing::{
        device_description::DeviceDesc, process_packet::Processed, thread_packet::MessageType,
    },
};
use flume::{Receiver, SendError, Sender, TryRecvError};
use std::{
    convert::TryInto,
    error::Error,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, RwLock,
    },
    thread::{Builder, JoinHandle},
    time::Duration,
};
use rayon::{ThreadPoolBuilder, ThreadPool, ThreadPoolBuildError};

struct InputProcessing {
    device: DeviceDesc,
    format: uvc::StreamFormat,
    sender_p1: Sender<MessageType>,   // To Thread
    reciever_p2: Receiver<Processed>, // From Thread
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
    pub fn change_device(&self, device: DeviceDesc) -> Result<(), ()> {
        match self.sender_p1.send(MessageType::SET(device)) {
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
    recv: Receiver<MessageType>,
    send: Sender<Processed>,
    startup_desc: DeviceDesc,
    startup_format: uvc::StreamFormat,
) -> u8 {
    std::thread::sleep(Duration::from_millis(100));
    let device_serial = match startup_desc.ser {
        Some(serial) => Some(serial),
        None => None,
    };
    let mut current_format = startup_format;
    let mut current_device: uvc::Device = match crate::UVC.find_device(
        startup_desc.vid,
        startup_desc.pid,
        device_serial.as_deref(),
    ) {
        Ok(v) => v,
        Err(_why) => {
            return 1;
        }
    };
    let threads = (1000/current_format.fps) + 1;
    let processing_pool = match ThreadPoolBuilder::new().num_threads(threads as usize).build() {
        Ok(v) => {
            v
        }
        Err(_why) => {
            return 4;
        }
    };

    let counter = Arc::new(AtomicUsize::new(0));
    current_device
        .open()
        .unwrap()
        .get_stream_handle_with_format(current_format)
        .unwrap()
        .start_stream(|frame, count| {}, counter.clone())
        .expect("Could not start stream!");
    0
}

fn trick_or_channel(message: Result<MessageType, TryRecvError>) -> DeviceOrTrick {
    return match message {
        Ok(msg) => match msg {
            MessageType::SET(dev) => DeviceOrTrick::Device(dev),
            MessageType::CLOSE(code) => DeviceOrTrick::Exit(code),
            MessageType::DIE(code) => DeviceOrTrick::Exit(code),
        },
        Err(e) => match e {
            TryRecvError::Disconnected => DeviceOrTrick::Exit(3),
            TryRecvError::Empty => DeviceOrTrick::None,
        },
    };
}

// Trick or treat with death and webcams. Fun for the whole family!
enum DeviceOrTrick {
    Device(DeviceDesc),
    Exit(u8),
    None,
}
