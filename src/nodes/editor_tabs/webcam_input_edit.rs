use crate::nodes::editor_tabs::util::create_editable_item;
use gdnative::{
    api::{tree::Tree, tree_item::*},
    core_types::Rect2,
    prelude::*,
    NativeClass,
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
            &*owner
                .create_item(root_item.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };
        webcam_select_list.set_text(0, "Input Webcam:");
        webcam_select_list.set_text_align(0, 0);
        webcam_select_list.set_cell_mode(1, 4);
        webcam_select_list.set_editable(1, true);
        //webcam_select_list.set_custom_draw(1, owner, "webcam_editor_clicked");
        webcam_select_list.set_text(1, "Select...");

        owner.connect(
            "custom_popup_edited",
            owner,
            "on_item_clicked",
            VariantArray::new_shared(),
            0,
        );
    }

    #[export]
    // The documentation on this is piss poor, so this will probably be wrong. Trial and error the function arguments until it works.
    // EDIT: I've been trying to debud this for over 2 hours only to realise i never attached the native script to the Tree.
    fn webcam_editor_clicked(&self, owner: TRef<Tree>, treeitem: Ref<TreeItem>, rect: Rect2) {
        godot_print!("a");
        let uvc_devices = match crate::UVC.devices() {
            Ok(dev) => {
                godot_print!("b");
                dev
            }
            Err(why) => {
                // show error
                return;
            }
        };
        // Change to directly filling menu
        let mut devlist: Vec<Device> = Vec::new();
        for i in uvc_devices.into_iter() {
            devlist.push(i);
        }
    }

    #[export]
    pub fn on_item_clicked(&self, owner: &Tree) {
        godot_print!("a");
    }
}
