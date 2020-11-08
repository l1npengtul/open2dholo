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
        let thread = Builder::new()
            .name(format!(
                "input-processor_{}-{}",
                bind_device.product_id, num_thread
            ))
            .spawn(move || input_process_func(to_thread_rx, from_thread_tx))
            .unwrap();
        let description = DeviceDesc::from_description(bind_device);
        to_thread_tx.send(MessageType::SET(description.clone()));
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

pub fn input_process_func(
    recv: Receiver<MessageType>,
    send: Sender<Processed>,
) -> u8 {
    std::thread::sleep(Duration::from_millis(100));
    let mut current_device: Option<DeviceDesc> = None;
    'main: loop {
        let incoming = match recv.try_recv() {
            Ok(msg) => {
                match msg {
                    MessageType::SET(dev) => {
                        current_device = Some(dev)
                    }
                    MessageType::CLOSE(code) => {

                    }
                    MessageType::DIE(exit) => {
                        return exit;
                    }
                }
            }
            Err(e) => {
                match e {
                    TryRecvError::Disconnected => {
                        return 0;
                    }
                    _ => {
                        // spooky the error goes into the ether
                    }
                }
            }
        }
    }
    0
}
