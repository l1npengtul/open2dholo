use gdnative::{GodotObject, TRef, api::{Tree, TreeItem}};

pub fn create_editable_item(item: &TreeItem, field: &str) {
    item.set_text(0, field);
    item.set_text_align(0, TreeItem::ALIGN_LEFT);
    item.set_editable(1, true);
}

pub fn create_custom_editable_item(owner: TRef<Tree>, parent: &TreeItem, field: &str, idx: i64) {
    let webcam_format_resoultion: &TreeItem = unsafe {
        &*owner
            .create_item(parent.assume_shared(), idx)
            .unwrap()
            .assume_safe()
    };
    webcam_format_resoultion.set_text(0, field);
    webcam_format_resoultion.set_text_align(0, 0);
    webcam_format_resoultion.set_cell_mode(1, 4);
    webcam_format_resoultion.set_editable(1, true);
}