//use flume::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};

use crate::{
    error::thread_send_message_error::ThreadSendMessageError,
    processing::{
        process_packet::Processed,
        thread_packet::{MessageType, ThreadMessage},
        device_description::DeviceDesc,
    },
};
use flume::{Receiver, SendError, Sender, TryRecvError};
use std::error::Error;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use std::{
    sync::Arc,
    thread::{Builder, JoinHandle},
};
use std::sync::RwLock;
use gdnative::api::VisualScriptSubCall;
use std::convert::TryInto;

struct InputProcessing {
    device: DeviceDesc,
    sender_p1: Sender<MessageType>, // To Thread
    reciever_p2: Receiver<Processed>, // From Thread
    thread_handle: JoinHandle<u8>,
}

impl InputProcessing {
    pub fn new(
        num_thread: usize,
        bind_device: uvc::DeviceDescription,
    ) -> Self {
        let (to_thread_tx, to_thread_rx) = flume::unbounded();
        let (from_thread_tx, from_thread_rx) = flume::unbounded();
        let description = DeviceDesc::from_description(bind_device);
        let thread = Builder::new()
            .name(format!(
                "input-processor_{}-{}",
                bind_device.product_id, num_thread
            ))
            .spawn(move || input_process_func(to_thread_rx, from_thread_tx, description.clone()))
            .unwrap();
        InputProcessing {
            device: description,
            sender_p1: to_thread_tx,
            reciever_p2: from_thread_rx,
            thread_handle: thread,
        }
    }
    pub fn change_device(&self, device: DeviceDesc) -> Result<(), ()>{
        match self.sender_p1.send(msg) {
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
 * -1 = Thread Communication Error
 * 0 = Sucessful Exit
 * 1 = Device not found
 * 2 = General Error
 */

fn input_process_func(
    recv: Receiver<MessageType>,
    send: Sender<Processed>,
    startup_desc: DeviceDesc,
    startup_format:
) -> u8 {
    std::thread::sleep(Duration::from_millis(100));
    let device_serial = match startup_desc.ser {
        Some(serial) => Some(&serial.to_owned()[..]),
        None => None
    };
    let mut current_device: uvc::Device = match crate::UVC.find_device(startup_desc.vid, startup_desc.pid, device_serial) {
        Ok(v) => v,
        Err(_why) => {
            return 1;
        }
    };
    current_device.open().unwrap().get_stream_handle_with_format()
    0
}

fn trick_or_channel(message: Result<MessageType, TryRecvError>) -> DeviceOrTrick {
    return match message {
        Ok(msg) => {
            match msg {
                MessageType::SET(dev) => {
                    DeviceOrTrick::Device(dev)
                }
                MessageType::CLOSE(code) => {
                    DeviceOrTrick::Exit(code)
                }
                MessageType::DIE(exit) => {
                    DeviceOrTrick::Exit(code)
                }
            }
        }
        Err(e) => {
            match e {
                TryRecvError::Disconnected => {
                    DeviceOrTrick::Exit(-1)
                }
                TryRecvError::Empty => {
                    DeviceOrTrick::None
                }
            }
        }
    }
}

// Trick or treat with death and webcams. Fun for the whole family!
enum DeviceOrTrick {
    Device(DeviceDesc),
    Exit(u8),
    None
}