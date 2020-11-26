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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidDeviceError {
    #[error("Device with description vendor id: {vendor}, product id: {prod}, serial number: {ser}, ERROR could not open/get device! Make sure it exists!")]
    InvalidDescription {
        vendor: String,
        prod: String,
        ser: String,
    },
    #[error("Could not find and open the device!")]
    CannotFindDevice,
    #[error("Could not get device property \"{prop}\": {msg}")]
    CannotGetDeviceInfo { prop: String, msg: String },
}
