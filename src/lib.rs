use gdnative::prelude::*;
use uvc;



pub mod input_processer;
pub mod model_tree_edit;
pub mod open2dhctrl;
pub mod process_packet;
pub mod thread_packet;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref UVC: uvc::Context = {
        let ctx = uvc::Context::new();
        ctx
    };
}

fn init(handle: InitHandle) {
    handle.add_class::<self::open2dhctrl::Main>();
    handle.add_class::<self::model_tree_edit::ModelTreeEditor>();
}

godot_init!(init);
// Macro that creates the entry-points of the dynamic library.
