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
    fn _ready(&self, owner: TRef<Tree>) {
        let root_item: &TreeItem = unsafe {
            &*owner
                .create_item(owner.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };

        owner.set_hide_root(true);
        owner.set_columns(2);

        let webcam_video_input: &TreeItem = unsafe {
            &*owner
                .create_item(root_item.assume_shared(), 1)
                .unwrap()
                .assume_safe()
        };
        webcam_video_input.set_text(0, "Webcam Input Settings");
        webcam_video_input.set_text_align(0, TreeItem::ALIGN_CENTER);

        let webcam_select_list: &TreeItem = unsafe {
            &*owner.create_item(root_item.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };

        owner.connect("custom_popup_edited", owner, "item_clicked", VariantArray::new_shared(), 0);
    }
    #[export]
    fn item_clicked(&self, owner: &Tree) {

    }
}
