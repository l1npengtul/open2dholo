use gdnative::{
    prelude::*,
    NativeClass, api::*
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
        match (_owner.create_item()) {
            
        };
        _owner.set_hide_root(true);
    
    }
}


