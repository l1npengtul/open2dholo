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

use nalgebra::{Matrix2, Matrix2x4, Matrix4, RowVector2, RowVector4};
use opencv::{
    core::{Mat, CV_32F},
    prelude::*,
    video::KalmanFilter,
};
use std::error::Error;

// pub struct KalmanStabilize {
//     dynam: i32,
//     measure: i32,
//     filter: KalmanFilter,
//     measurement: Mat,
//     state: Mat,
//     predict: Mat,
// }
//
// impl KalmanStabilize {
//     pub fn new(dynam: i32, measure: i32) -> Result<Self, Box<dyn Error>> {
//         let mut filter = match KalmanFilter::new(dynam, measure, 0, CV_32F) {
//             Ok(f) => f,
//             Err(e) => {}
//         };
//
//         // set up point stabilizers
//         filter.set_transition_matrix(Matrix4::from_rows(&[
//             RowVector4::new(1.0, 0.0, 1.0, 0.0),
//             RowVector4::new(0.0, 1.0, 0.0, 1.0),
//             RowVector4::new(0.0, 0.0, 1.0, 0.0),
//             RowVector4::new(0.0, 0.0, 0.0, 1.0),
//         ]));
//
//         filter.set_measurement_matrix(Matrix2x4::from_rows(&[
//             RowVector4::new(1.0, 0.0, 0.0, 0.0),
//             RowVector4::new(0.0, 1.0, 0.0, 0.0),
//         ]));
//
//         filter.set_process_noise_cov(
//             Matrix4::from_rows(&[
//                 RowVector4::new(1.0, 0.0, 0.0, 0.0),
//                 RowVector4::new(0.0, 1.0, 0.0, 0.0),
//                 RowVector4::new(0.0, 0.0, 1.0, 0.0),
//                 RowVector4::new(0.0, 0.0, 0.0, 1.0),
//             ]) * 0.1,
//         );
//
//         filter.set_measurement_noise_cov(
//             Matrix2::from_rows(&[RowVector2::new(1.0, 0.0), RowVector2::new(0.0, 1.0)]) * 0.1,
//         );
//
//         Ok(KalmanStabilize {
//             dynam,
//             measure,
//             filter,
//             measurement: (),
//             state: (),
//             predict: (),
//         })
//     }
// }
