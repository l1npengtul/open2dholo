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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Error: {0}!")]
    General(String),
    #[error("Could not get the CNN Facial Detection Model at filepath: {0}!")]
    CNNModelNotFound(String),
    #[error("Could not get the Facial Landmark Detector at filepath: {0}!")]
    LandmarkPredictorNotFound(String),
    #[error("Expected 68 landmark points, only found {0}!")]
    AllPointsNotDetected(usize),
}

unsafe impl Send for ProcessingError {}
unsafe impl Sync for ProcessingError {}
