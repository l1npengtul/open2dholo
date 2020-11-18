use crate::nodes::editor_tabs::util::create_editable_item;
use gdnative::{
    api::{tree::Tree, tree_item::{TreeItem}},
    prelude::*,
    NativeClass,
    core_types::Rect2,
};
use uvc::Device;

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
        webcam_video_input.set_text_align(0, TreeItem::ALIGN_LEFT);

        let webcam_select_list: &TreeItem = unsafe {
            &*owner.create_item(root_item.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };
        webcam_select_list.set_text(0, "Input Webcam:");
        webcam_select_list.set_text_align(0, 0);
        webcam_select_list.set_cell_mode(1, 4);
        webcam_select_list.set_editable(1, true);

        if let Err(why) = owner.connect("custom_popup_edited", owner, "on_item_clicked", VariantArray::new_shared(), 0) {
            //panic!("Could not initialise UI!");
        }
    }

    #[export]
    pub fn on_item_clicked(&self, owner: TRef<Tree>, arrow_clicked: bool) {
        if arrow_clicked {
            let clicked_item = unsafe {owner.assume_shared().assume_safe().get_edited().unwrap().assume_safe().get_text(0).to_string()}.to_owned(); // bruh what the fuck
            match &clicked_item[..] {
                "Input Webcam:" => {
                    godot_print!("input webcam")
                }
                _ => {
                    return;
                }
            }
        }
    }
}
