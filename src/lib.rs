use gdnative::prelude::*;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);