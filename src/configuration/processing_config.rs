use std::sync::atomic::{AtomicBool, AtomicUsize};
use serde::{Deserialize, Serialize};
use crate::processing::device_description::DeviceDesc;

#[derive(Deserialize, Serialize)]
pub struct ProcessingConfig {
    pub(crate) use_cnn: AtomicBool,
    pub(crate) max_threads: AtomicUsize,
    pub(crate) default_device: DeviceDesc,
}

