use gdnative::api::TreeItem;

pub fn create_editable_item(item: &TreeItem, field: &str) {
    item.set_text(0, field);
    item.set_text_align(0, TreeItem::ALIGN_LEFT);
    item.set_editable(1, true);
}
