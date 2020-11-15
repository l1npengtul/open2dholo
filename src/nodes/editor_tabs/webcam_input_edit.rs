use crate::nodes::editor_tabs::util::create_editable_item;
use gdnative::{
    api::{tree::Tree, tree_item::*},
    prelude::*,
    NativeClass,
};

#[derive(NativeClass)]
#[inherit(Tree)]
pub struct WebcamInputEditor;

#[methods]
impl WebcamInputEditor {
    fn new(_owner: &Tree) -> Self {
        WebcamInputEditor
    }
    #[export]
    fn _ready(&self, owner: &Tree) {
        let root_item: &TreeItem = unsafe {
            &*owner
                .create_item(owner.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };

        // TODO: Less .unwrap() more error handle

        owner.set_hide_root(true);
        owner.set_columns(2);
    }
}
