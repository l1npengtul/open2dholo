use std::sync::atomic::{AtomicBool, AtomicUsize};
use serde::{Deserialize, Serialize};
use gdnative::prelude::*;
use crate::configuration::processing_config::ProcessingConfig;
use crate::processing::device_description::DeviceDesc;

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    processing: ProcessingConfig,
}

impl UserConfig {
    pub fn from_default() -> Self {
        UserConfig {
            processing: ProcessingConfig {
                use_cnn: AtomicBool::new(false),
                max_threads: AtomicUsize::new(8),
                default_device: DeviceDesc::from_default()
            }
        }
    }

    pub fn new(filepath: String) -> Self {

    }
}

