//     Open2DHolo - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
//     Copyright (C) 2020-2021 l1npengtul
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

use facial_processing::utils::{face::FaceLandmark, misc::{BackendProviders, EulerAngles, Point2D}};

use crate::util::camera::device_utils::{PossibleDevice, Resolution, DeviceConfig};

// TODO: Change to acutal data format
#[derive(Clone)]
pub enum MessageType {
    Die(u8),
    SetDevice(PossibleDevice),
    ChangeDevice(DeviceConfig)
}

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    Dlib,
}

#[derive(Clone, Copy, Debug)]
pub struct BackendConfig {
    backend: Backend,
    input_src_original: Resolution,
    input_src_scaled: Resolution,
    // DeviceConfig TODO: wait for TVM
}
impl BackendConfig {
    pub fn new(res: Resolution, backend: Backend) -> Self {
        // let mut scaled_res = Resolution::new(640, 480);
        // let mut scale_x = 1_f64;
        // let mut scale_y = 1_f64;
        // if res.x < 640 {

        // }
        BackendConfig {
            backend,
            input_src_original: res,
            input_src_scaled: res, // TODO: find min scale factor
        }
    }

    pub fn backend_as_facial(&self) -> BackendProviders {
        match self.backend {
            Backend::Dlib => {
                BackendProviders::DLib {
                    face_alignment_path: globalize_path!("res://models/facial-processing-rs-models/shape_predictor_68_face_landmarks.dat")
                }
            }
        }
    }

    pub fn res(&self) -> Resolution {
        self.input_src_original
    }
}

#[derive(Clone)]
pub struct FullyCalculatedPacket {
    pub landmarks: FaceLandmark,
    pub euler: EulerAngles,
    pub eye_positions: [Point2D; 2],
}
