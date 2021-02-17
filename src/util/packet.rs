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

use crate::util::camera::device_utils::PossibleDevice;
use dlib_face_recognition::Point;
use opencv::core::Mat;
use std::sync::atomic::AtomicUsize;
use crate::processing::face_detector::facial_existance::FaceBox;
use crate::processing::face_detector::detectors::util::{Point2D, Point3D};

// TODO: Change to acutal data format
#[derive(Clone)]
pub enum MessageType {
    Die(u8),
    Set(PossibleDevice),
    Close(u8),
}

// TODO: Change to acutal data format

#[derive(Clone)]
pub struct Processed {
    landmarks: Vec<Point>,
}

impl Processed {
    pub fn new(data: Vec<Point>) -> Self {
        Processed { landmarks: data }
    }
}


#[derive(Clone)]
pub struct ProcessFaceDetectionPacket {
    pub(crate) img_data: Mat,
    pub(crate) img_height: u32,
    pub(crate) img_width: u32,
}


#[derive(Clone)]
pub enum RecieveProcessFaceLandmarkPacket {
    ItWorkedPog2D(Vec<Point2D>),
    ItWorkedPog3D(Vec<Point3D>),
    ItDidntWorkedBruh(Box<dyn std::error::Error>),
}