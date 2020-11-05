use gdnative::prelude::*;

pub mod open2dhctrl;
pub mod model_tree_edit;
pub mod input_processer;

fn init(handle: InitHandle){
    handle.add_class::<self::open2dhctrl::Main>();
    handle.add_class::<self::model_tree_edit::ModelTreeEditor>()
}

godot_init!(init);
// Macro that creates the entry-points of the dynamic library.
