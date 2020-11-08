#![deny(clippy::pedantic)]
use gdnative::prelude::*;
use uvc;

pub mod error;
pub mod nodes;
pub mod processing;

#[macro_use]
extern crate lazy_static;

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
// Macro that creates the entry-points of the dynamic library.
