//use flume::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};
use gdnative::prelude::*;
use std::{sync::Arc, thread::JoinHandle};
use rayon::{prelude::*, ThreadPoolBuilder, ThreadPool};
use crate::{
    process_packet,
    thread_packet
};
use uvc;
use flume::{Sender, Receiver};


struct InputProcessing {
    processor: fn(uvc::Frame) -> process_packet::Processed,
    sender: Sender<process_packet::Processed>,
    reciever: Receiver<thread_packet::MessageType>,
    thread_handle: JoinHandle<_>
}

impl InputProcessing {
    pub fn new(num_thread: usize) -> Self {
        InputProcessing {

        }
    }
    pub fn send_message(msg: thread_packet::MessageType) -> Result<>
    pub fn kill(&self) {

    }
}

impl Drop for InputProcessing{
    fn drop(&mut self) {
        unimplemented!()
    }
}