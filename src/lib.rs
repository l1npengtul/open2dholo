//     Open2DH - Open 2D Holo, a program to procedurally animate your face onto an anime girl.
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

#![deny(clippy::pedantic)]
#![warn(clippy::all)]
#![allow(clippy::clippy::module_name_repetitions)]

use crate::util::camera::device_utils::DeviceContact;
use gdnative::prelude::*;
use std::cell::RefCell;

pub mod configuration;
pub mod error;
pub mod nodes;
pub mod processing;
pub mod util;

#[macro_use]
extern crate lazy_static;
extern crate downcast_rs;

// Make it so we can get a webcam stream anywhere so we don't have to deal with 'static bullshit
lazy_static! {
    static ref UVC: uvc::Context<'static> = {
        let ctx = uvc::Context::new();
        ctx.expect("Could not get UVC Context! Aborting!")
    };
}

thread_local! {
    pub(crate) static CURRENT_DEVICE: RefCell<Option<DeviceContact>> = RefCell::new(None);
    pub(crate) static UVC_DEV_H: RefCell<Option<uvc::DeviceHandle<'static>>> = RefCell::new(None);
}

fn init(handle: InitHandle) {
    handle.add_class::<crate::nodes::main::open2dhctrl::Main>();
    handle.add_class::<crate::nodes::editor_tabs::model_tree_edit::ModelTreeEditor>();
    handle.add_class::<crate::nodes::editor_tabs::webcam_input_edit::WebcamInputEditor>();
    handle.add_class::<crate::nodes::viewports::viewport_holder::ViewportHolder>()
}

godot_init!(init);
