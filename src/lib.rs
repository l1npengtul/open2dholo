use gdnative::prelude::*;
use uvc::Device;

mod main;
fn init(handle: InitHandle) {
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);