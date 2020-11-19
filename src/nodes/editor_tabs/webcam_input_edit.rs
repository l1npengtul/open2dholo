use gdnative::{
    api::{tree::Tree, tree_item::TreeItem, popup_menu::PopupMenu},
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
        let popup = unsafe {owner.get_node("../PopupMenu").unwrap().assume_safe().cast::<PopupMenu>().unwrap()};
        popup.set_visible(false);
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

        if let Err(_why) = owner.connect(
            "custom_popup_edited",
            owner,
            "on_item_clicked",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Could not initialise UI!");
        }
    }

    #[export]
    pub fn on_item_clicked(&self, owner: TRef<Tree>, arrow_clicked: bool) {
        if arrow_clicked {
            let clicked_item = unsafe {
                owner
                    .assume_shared()
                    .assume_safe()
                    .get_edited()
                    .unwrap()
                    .assume_safe()
            }; // bruh what the fuck
            let clicked_string = clicked_item.get_text(0).to_string().to_owned();
            match &clicked_string[..] {
                "Input Webcam:" => {
                    let rect = owner.get_custom_popup_rect();
                    let size = rect.size.to_vector();
                    let position = rect.origin.to_vector();
                    
                    let devices = crate::UVC.devices().expect("Could not get devices!"); // TODO replace with match
                    let mut device_list: Vec<uvc::Device> = Vec::new();
                    let mut counter = 0;
                    let popup = unsafe {owner.get_node("../PopupMenu").unwrap().assume_safe().cast::<PopupMenu>().unwrap()};
                    
                    for device in devices {
                        popup.add_item(device.device_address().to_owned().to_string(), counter, 1 );
                        counter += 1;
                    }
                    popup.set_position(position, true);
                    popup.set_size(size, true);
                    popup.set_visible(true);
                },
                _ => {
                    return;
                }
            }
        }
    }

    #[export]
    pub fn on_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        let popup = unsafe {owner.get_node("../PopupMenu").unwrap().assume_safe().cast::<PopupMenu>().unwrap()};
        // TODO: Get USB device from thing and open device
    }
}
