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

use dlib_face_recognition::Point;
use parking_lot::RwLock;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::util::camera::device_utils::PossibleDevice;


// TODO: Change to acutal data format
pub enum MessageType {
    Die(u8),
    Set(PossibleDevice),
    Close(u8),
}

// TODO: Change to acutal data format

#[derive(Clone)]
pub struct Processed {
    landmarks: Vec<Point>,
    frame_data: Option<Arc<RwLock<uvc::Frame>>>,
}
impl Processed {
    pub fn new(data: Vec<Point>, imgframe: Option<Arc<RwLock<uvc::Frame>>>) -> Self {
        Processed {
            landmarks: data,
            frame_data: imgframe,
        }
    }
}

// For future reference, the `FaceLandmarks` struct is a Vector<Point>.
pub enum ProcessedPacket {
    None,
    FacialLandmark(Processed),
    GeneralError(String),
    MissingFacialPointsError(AtomicUsize),
    MissingFileError(String),
}
