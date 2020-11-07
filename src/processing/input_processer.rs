//use flume::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};

use crate::processing::{process_packet::Processed, thread_packet::MessageType};
use flume::{Receiver, SendError, Sender};
use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use std::{
    process::Ex,
    sync::Arc,
    thread::{Builder, JoinHandle},
};
use uvc::Result;

struct InputProcessing {
    processor: fn(Arc<uvc::StreamHandle<'static>>, Receiver<MessageType>, Sender<Processed>) -> u8,
    sender_p1: Sender<MessageType>,   // To Thread
    reciever_p2: Receiver<Processed>, // From Thread
    thread_handle: JoinHandle<u8>,
}

impl InputProcessing {
    pub fn new(
        num_thread: usize,
        bind_device: uvc::DeviceDescription,
        stream: Arc<uvc::StreamHandle<'static>>,
    ) -> Self {
        let (to_thread_tx, to_thread_rx) = flume::unbounded();
        let (from_thread_tx, from_thread_rx) = flume::unbounded();
        let thread = Builder::new()
            .name(format!(
                "input-processor_{}-{}",
                bind_device.product_id, num_thread
            ))
            .spawn(move || input_process_func(stream, to_thread_rx, from_thread_tx))
            .unwrap();
        InputProcessing {
            processor: input_process_func,
            sender_p1: to_thread_tx,
            reciever_p2: from_thread_rx,
            thread_handle: thread,
        }
    }
    pub fn send_message(
        &self,
        msg: MessageType,
    ) -> &std::result::Result<(), flume::SendError<MessageType>> {
        &self.sender_p1.send(msg)
    }
    //pub fn get_output_handler
    pub fn kill(&self) {}
}

impl Drop for InputProcessing {
    fn drop(&mut self) {
        unimplemented!()
    }
}

pub fn input_process_func(
    mut stream_handler: Arc<uvc::StreamHandle<'static>>,
    recv: Receiver<MessageType>,
    send: Sender<Processed>,
) -> u8 {
    std::thread::sleep(Duration::from_millis(100));
    // Open the stream
    let counter = Arc::new(AtomicUsize::new(0));
    let stream = stream_handler
        .start_stream(
            |frame, count| {
                // do computer vision magic trickery bullshit here yayyyyyyyyy
                // TODO: Make opencv not cry like a godamn baby everytime i try and compile it (opencv4.so in build directory anyone? no?)
            },
            counter.clone(),
        )
        .expect("Could not start stream!");
    0
}
