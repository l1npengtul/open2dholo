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
use crate::configuration::user_config::UserConfig;
use dlib_face_recognition::FaceDetector;
use gdnative::prelude::*;
use parking_lot::RwLock;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::{atomic::AtomicBool, Arc};
use uvc;

pub mod configuration;
pub mod error;
pub mod nodes;
pub mod processing;

#[macro_use]
extern crate lazy_static;

// Make it so we can get a webcam stream anywhere so we don't have to deal with 'static bullshit
lazy_static! {
    static ref UVC: uvc::Context<'static> = {
        let ctx = uvc::Context::new();
        ctx.expect("Could not get UVC Context! Aborting!")
    };
    // REPLACE WITH CONFIGURATION STRUCT
    static ref FACE_DETECTED: AtomicBool = AtomicBool::new(false);
    //static ref USER_CONFIG: RwLock<UserConfig> = {};
    static ref PROCESSING_POOL: ThreadPool = {
        let processing_pool = ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .expect("Could not build threadpool!");
        processing_pool
    };
}

fn init(handle: InitHandle) {
    handle.add_class::<self::nodes::main::open2dhctrl::Main>();
    handle.add_class::<self::nodes::editor_tabs::model_tree_edit::ModelTreeEditor>();
}

godot_init!(init);
