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

// What if it was all a lie, and nothing is real and this is all a dream?
// What if we don't exist at all?

use cv_convert::TryIntoCv;
use gdnative::godot_print;
use image::RgbImage;
use opencv::core::{Scalar, Vector};
use opencv::dnn::blob_from_image;
use opencv::objdetect::CascadeClassifier;
use opencv::{
    core::{Mat, ToInputArray},
    dnn::{read_net_from_caffe, Net, NetTrait},
    imgcodecs::{imdecode, imwrite, ImreadModes},
    Error,
};
use std::collections::HashMap;

pub enum CurrentDetector<'a> {
    HaarCascades(CascadeClassifier, &'a str),
    LocalBinaryPatterns(),
}

pub struct FacialDetector {
    dnn: Net,
}

impl FacialDetector {
    pub fn new(proto_text_path: &str, dnn_path: &str) -> Result<Self, ()> {
        match read_net_from_caffe(proto_text_path, dnn_path) {
            Ok(n) => Ok(FacialDetector { dnn: n }),
            Err(_why) => Err(()),
        }
    }

    pub fn detect_face(&mut self, _img_height: u32, _img_width: u32, img_raw_data: &[u8])
    /*-> HashMap<FaceBox, f32>*/
    {
        let facebox_hashmap: HashMap<FaceBox, f32> = HashMap::new();
        let img_data_vec: opencv::core::Vector<u8> = Vector::from(img_raw_data.to_vec());
        let img_mat = match imdecode(&img_data_vec, ImreadModes::IMREAD_GRAYSCALE as i32) {
            Ok(i) => blob_from_image(
                &i,
                1.0,
                opencv::core::Size::new(300, 300),
                opencv::core::Scalar::default(),
                false,
                false,
                5,
            )
            .unwrap(),
            Err(why) => {
                panic!("{}", why.to_string())
            }
        };
    }
}

#[derive(Copy, Clone)]
pub struct FaceBox {
    x_left_bottom: i32,
    x_right_bottom: i32,
    y_left_top: i32,
    y_right_top: i32,
}
impl FaceBox {
    pub fn new(x_left_bottom: i32, x_right_bottom: i32, y_left_top: i32, y_right_top: i32) -> Self {
        FaceBox {
            x_left_bottom,
            x_right_bottom,
            y_left_top,
            y_right_top,
        }
    }
}
