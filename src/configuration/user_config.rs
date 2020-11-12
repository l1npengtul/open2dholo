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

