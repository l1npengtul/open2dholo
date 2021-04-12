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

use gdnative::{
    api::{Tree, TreeItem},
    GodotObject, TRef,
};

pub fn create_editable_item(item: &TreeItem, field: &str) {
    item.set_text(0, field);
    item.set_text_align(0, TreeItem::ALIGN_LEFT);
    item.set_editable(1, true);
}

pub fn create_editable_range(item: &TreeItem, field: &str, min: f64, max: f64, step: f64) {
    item.set_text(0, field);
    item.set_text_align(0, TreeItem::ALIGN_LEFT);
    item.set_editable(1, true);
    item.set_range_config(1, min, max, step, false);
}

pub fn create_custom_editable_item(owner: TRef<Tree>, parent: &TreeItem, field: &str, idx: i64) {
    let webcam_format_resoultion: &TreeItem = unsafe {
        &*owner
            .create_item(parent.assume_shared(), idx)
            .unwrap()
            .assume_safe()
    };
    webcam_format_resoultion.set_text(0, field);
    webcam_format_resoultion.set_text_align(0, 0);
    webcam_format_resoultion.set_cell_mode(1, 4);
    webcam_format_resoultion.set_editable(1, true);
}
