//     Open2DH - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
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

use crate::configuration::processing_config::ProcessingConfig;
use crate::error::config_error::ConfigError;
use crate::util::camera::device_utils::DeviceDesc;
use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::{fs::File, path::Path};

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
                default_device: DeviceDesc::from_default(),
            },
        }
    }

    pub fn from_cfg() -> Result<Self, Box<dyn std::error::Error>> {
        let file_path: &Path = Path::new("config/settings.ron");
        let cfg_file = File::open(file_path);
        match cfg_file {
            Ok(file) => match from_reader(file) {
                Ok(cfg) => Ok(cfg),
                Err(_why) => Err(Box::new(ConfigError::InvalidConfiguration(String::from(
                    "config/settings.ron",
                )))),
            },
            Err(_why) => Err(Box::new(ConfigError::FileNotFound(String::from(
                "config/settings.ron",
            )))),
        }
    }

    pub fn from_custom_cfg(file_path: Box<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path_str = match file_path.to_path_buf().into_os_string().into_string() {
            Ok(p) => p,
            Err(_str) => return Err(Box::new(ConfigError::InvalidPath)),
        };
        let cfg_file = File::open(file_path);
        match cfg_file {
            Ok(file) => match from_reader(file) {
                Ok(cfg) => Ok(cfg),
                Err(_why) => Err(Box::new(ConfigError::InvalidConfiguration(file_path_str))),
            },
            Err(_why) => Err(Box::new(ConfigError::FileNotFound(file_path_str))),
        }
    }

    //pub fn write_current(&self) -> Result<(), Box<dyn std::error::Error>> {}
}
