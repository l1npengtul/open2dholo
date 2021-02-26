//     Open2DH - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
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
// NO MATTER WHAT LINT COMES THROUGH THAT GATE
#![allow(clippy::clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unused_self)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use crate::util::camera::device_utils::DeviceContact;
use gdnative::prelude::*;
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;

pub mod configuration;
pub mod error;
pub mod nodes;
pub mod processing;
pub mod util;

#[macro_use]
extern crate lazy_static;

// Make it so we can get a webcam stream anywhere so we don't have to deal with 'static bullshit
lazy_static! {
    static ref UVC: Arc<uvc::Context<'static>> = {
        let ctx = uvc::Context::new();
        Arc::new(ctx.expect("Could not get UVC Context! Aborting!"))
    };
    static ref USER_DIR: Arc<String> = {
        Arc::new(
            gdnative::api::OS::godot_singleton()
                .get_user_data_dir()
                .to_string(),
        )
    };
}

thread_local! {
    pub(crate) static CURRENT_DEVICE: Rc<RefCell<Option<DeviceContact>>> = Rc::new(RefCell::new(None));
}

fn init(handle: InitHandle) {

    handle.add_class::<crate::nodes::main::open2dhctrl::Main>();
    handle.add_class::<crate::nodes::editor_tabs::model_tree_edit::ModelTreeEditor>();
    handle.add_class::<crate::nodes::editor_tabs::webcam_input_edit::WebcamInputEditor>();
    handle.add_class::<crate::nodes::viewports::viewport_holder::ViewportHolder>()
}

godot_init!(init);
