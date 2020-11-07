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
        ctx
    };
}

fn init(handle: InitHandle) {
    handle.add_class::<self::nodes::main::open2dhctrl>();
    handle.add_class::<self::nodes::editor_tabs::model_tree_edit>();
}

godot_init!(init);
// Macro that creates the entry-points of the dynamic library.
