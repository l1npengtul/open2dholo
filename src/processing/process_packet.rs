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

use gdnative::core_types::Vector2;
use parking_lot::RwLock;
use std::sync::Arc;

// TODO: Change to acutal data format

#[derive(Clone)]
pub struct Processed {
    data: Vec<Vector2>,
    frame_data: Option<Arc<RwLock<uvc::Frame>>>,
}
impl Processed {
    fn new(data: Vec<Vector2>, imgframe: Option<Arc<RwLock<uvc::Frame>>>) -> Self {
        Processed {
            data,
            frame_data: imgframe,
        }
    }
}
