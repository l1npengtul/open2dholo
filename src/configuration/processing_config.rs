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

use crate::processing::device_description::DeviceDesc;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize};

#[derive(Deserialize, Serialize)]
pub struct ProcessingConfig {
    pub(crate) use_cnn: AtomicBool,
    pub(crate) max_threads: AtomicUsize,
    pub(crate) default_device: DeviceDesc,
}