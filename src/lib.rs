#![deny(clippy::pedantic)]
use gdnative::prelude::*;
use uvc;

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
}

fn init(handle: InitHandle) {
    handle.add_class::<self::nodes::main::open2dhctrl::Main>();
    handle.add_class::<self::nodes::editor_tabs::model_tree_edit::ModelTreeEditor>();
}

godot_init!(init);
