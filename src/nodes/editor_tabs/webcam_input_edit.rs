//     Copyright (C) 2020-2021l1npengtul
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>.

use gdnative::{
    api::{tree::Tree, tree_item::TreeItem, popup_menu::PopupMenu},
    prelude::*,
    NativeClass,
};
use std::cell::RefCell;
use usb_enumeration::{enumerate, USBDevice};
use crate::util::device::{DeviceDesc, DeviceHolder};

#[derive(NativeClass)]
#[inherit(Tree)]
pub struct WebcamInputEditor {
    device_list: RefCell<Vec<DeviceHolder>>
}

#[methods]
impl WebcamInputEditor {
    fn new(_owner: &Tree) -> Self {
        let dev_list: RefCell<Vec<DeviceHolder>> = RefCell::new(Vec::new());
        let uvc_device_list = match crate::UVC.devices() {
            Ok(dev) => {
                dev
            }
            Err(_why) => {
                panic!("Could not get devices!");
                //TODO: Show error message when cannot get device list
            }
        };

        let usb_device_list = enumerate();
        for uvc_device in uvc_device_list {
            if let Ok(uvc_desc) = uvc_device.description() {
                for usb_device in usb_device_list.clone() {
                    if uvc_desc.product_id == usb_device.product_id && uvc_desc.vendor_id == usb_device.vendor_id {
                        let mut description: String = String::from(format!("{}:{}", uvc_desc.vendor_id, uvc_desc.product_id));
                        let mut id: String = String::from("");
                        if let Some(descript) = usb_device.description {
                            description = descript;
                        }
                        let to_push: DeviceHolder = DeviceHolder {
                            id: usb_device.id,
                            vendor_id: uvc_desc.vendor_id,
                            product_id: uvc_desc.product_id,
                            description: description,
                        };

                        if !dev_list.borrow().contains(&to_push) {
                            godot_print!("{}", to_push.clone().description);
                            dev_list.borrow_mut().push(to_push);
                        }
                    }
                }
            }
        }

        WebcamInputEditor {
            device_list: dev_list
        }
    }
    #[export]
    fn _ready(&self, owner: TRef<Tree>) {
        let popup = unsafe { owner.get_node("../PopupMenu").unwrap().assume_safe().cast::<PopupMenu>().unwrap() };
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
        let popup = unsafe { owner.get_node("../PopupMenu").unwrap().assume_safe().cast::<PopupMenu>().unwrap() };
        for item in 0..popup.get_item_count() {
            popup.remove_item(item);
        }
        if arrow_clicked {
            let clicked_item = unsafe {
                owner
                    .assume_shared()
                    .assume_safe()
                    .get_edited()
                    .unwrap()
                    .assume_safe()
            };
            let clicked_string = clicked_item.get_text(0).to_string().to_owned();
            match &clicked_string[..] {
                "Input Webcam:" => {
                    if popup.is_visible() {
                        popup.set_visible(false);
                    } else {
                        let rect = owner.get_custom_popup_rect();
                        let size = rect.size.to_vector();
                        let position = rect.origin.to_vector();
                        /*
                        let device_list = match crate::UVC.devices() {
                            Ok(dev) => {
                                dev
                            }
                            Err(_why) => {
                                panic!("Could not get devices!");
                                //TODO: Show error message when cannot get device list
                            }
                        };
                        self.device_list.borrow_mut().clear();

        
                        for device in device_list {
                            if let Ok(dev_desc) = device.description() {
                                self.device_list.borrow_mut().push(dev_desc);
                            }
                        }


                        let mut cnt = 0;
                        
                        for device_desc in self.device_list.borrow().iter() {
                           
                            let dev_formatted = device_desc.
                            popup.add_item(label, cnt, 1);
                            cnt += 1;
                        
                        }
                        */
                        popup.set_position(position, true);
                        popup.set_size(size, true);
                        popup.set_visible(true);
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }

    #[export]
    pub fn on_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        let popup = unsafe { owner.get_node("../PopupMenu").unwrap().assume_safe().cast::<PopupMenu>().unwrap() };
        // TODO: Get USB device from thing and open device
    }

    //fn 
}

