use crate::wtf;
use gdnative::{
    api::{Resource, Viewport},
    prelude::*,
    NativeClass,
};
use std::cell::RefCell;
// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Viewport)]
pub struct PreviewViewport {
    loaded_model: RefCell<Option<Ref<Resource>>>,
}

#[methods]
impl PreviewViewport {
    fn new(_owner: &Viewport) -> Self {
        PreviewViewport {
            loaded_model: RefCell::new(None),
        }
    }

    #[export]
    fn _ready(&self, owner: TRef<Viewport>) {
        let model_load_origin = unsafe {
            &mut owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HSplitContainer/VSplitContainer/HSplitContainer2").unwrap().assume_safe()
        };

        wtf!(model_load_origin.connect(
            "model_load_start",
            owner,
            "on_model_load_start",
            VariantArray::new_shared(),
            0
        ));
    }

    #[export]
    fn on_model_load_start(&self, owner: TRef<Viewport>, path: Variant) {
        godot_print!("?");
        let path_string = path.to_string();
        let loader = ResourceLoader::godot_singleton();
        match loader.load(path_string, "", false) {
            // What does `type_hint` do?
            Some(mdl) => {
                godot_print!("?");
                *self.loaded_model.borrow_mut() = Some(mdl);
                self.start_track_model(owner)
            }
            None => {
                godot_print!("failed to load model!");
            }
        }
    }

    #[export]
    fn start_track_model(&self, owner: TRef<Viewport>) {
        match &mut *self.loaded_model.borrow_mut() {
            Some(mdl) => {
                let node = unsafe {
                    mdl.assume_safe()
                        .cast::<PackedScene>()
                        .unwrap()
                        .instance(0)
                        .unwrap()
                        .assume_safe()
                };
                owner.add_child(node, true);
                for child_id in 0..owner.get_child_count() {
                    let node_name = unsafe {owner.get_child(child_id).unwrap().assume_safe() }.name();
                    godot_print!("{}", node_name);
                }
            }
            None => {}
        }
    }
}
