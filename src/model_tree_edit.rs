use gdnative::{
    prelude::*,
    NativeClass, nativescript, methods, api::*
};

#[derive(NativeClass)]
#[inherit(Tree)]
pub struct ModelTreeEditor;

#[methods]
impl ModelTreeEditor {
    fn new(_owner: &Tree) -> Self {
        ModelTreeEditor
    }
    #[export]
    fn _ready(&self, _owner: &Tree) {
    }
}


