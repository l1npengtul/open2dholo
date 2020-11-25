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

use crate::util::camera::device::{DeviceDesc, DeviceHolder};
use crate::util::camera::webcam::Webcam;
use crate::{
    nodes::editor_tabs::util::create_custom_editable_item,
    util::camera::{camera_device::V4LinuxDevice, device::Resolution},
};
use gdnative::{
    api::{popup_menu::PopupMenu, tree::Tree, tree_item::TreeItem},
    prelude::*,
    NativeClass,
};
use std::cell::RefCell;
use std::collections::HashMap;
use usb_enumeration::enumerate;

#[derive(NativeClass)]
#[inherit(Tree)]
pub struct WebcamInputEditor {
    device_list: RefCell<HashMap<String, Box<dyn Webcam>>>,
    device_selected: RefCell<Option<Box<dyn Webcam>>>,
    format_selected: RefCell<Option<Resolution>>,
    fps_selected: RefCell<Option<i32>>,
}

#[methods]
impl WebcamInputEditor {
    fn new(_owner: &Tree) -> Self {
        let dev_list = RefCell::new(WebcamInputEditor::get_device_list());
        WebcamInputEditor {
            device_list: dev_list,
            device_selected: RefCell::new(None),
            format_selected: RefCell::new(None),
            fps_selected: RefCell::new(None),
        }
    }
    #[export]
    fn _ready(&self, owner: TRef<Tree>) {
        let camera_popup = unsafe {
            owner
                .get_node("../CameraPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };

        let v4l2camera = V4LinuxDevice::new_path("/dev/video0".to_string()).unwrap();
        godot_print!("{}", v4l2camera.name());

        camera_popup.set_visible(false);
        if let Err(_why) = camera_popup.connect(
            "id_pressed",
            owner,
            "on_camera_popup_menu_clicked",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to initialise UI!");
        }

        let framerate_popup = unsafe {
            owner
                .get_node("../FrameratePopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        framerate_popup.set_visible(false);
        if let Err(_why) = framerate_popup.connect(
            "id_pressed",
            owner,
            "on_framerate_popup_menu_clicked",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to initialise UI!");
        }

        let video_popup = unsafe {
            owner
                .get_node("../VideoPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        video_popup.set_visible(false);
        if let Err(_why) = video_popup.connect(
            "id_pressed",
            owner,
            "on_video_popup_menu_clicked",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to initialise UI!");
        }

        let resolution_popup = unsafe {
            owner
                .get_node("../ResolutionPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        resolution_popup.set_visible(false);
        if let Err(_why) = resolution_popup.connect(
            "id_pressed",
            owner,
            "on_resolution_popup_menu_clicked",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to initialise UI!");
        }

        let root_item: &TreeItem = unsafe {
            &*owner
                .create_item(owner.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };

        owner.set_hide_root(true);
        owner.set_columns(2);

        // get ready for some UI spaghetti
        let webcam_video_input: &TreeItem = unsafe {
            &*owner
                .create_item(root_item.assume_shared(), 1)
                .unwrap()
                .assume_safe()
        };
        webcam_video_input.set_text(0, "Webcam Input Settings");
        webcam_video_input.set_text_align(0, TreeItem::ALIGN_LEFT);
        webcam_video_input.set_disable_folding(false);

        create_custom_editable_item(owner, root_item, "Input Webcam:", 2);
        create_custom_editable_item(owner, root_item, "Webcam Resolution:", 3);
        create_custom_editable_item(owner, root_item, "Webcam Frame Rate:", 4);
        create_custom_editable_item(owner, root_item, "Webcam Video Format:", 5);

        if let Err(_why) = owner.connect(
            "custom_popup_edited",
            owner,
            "on_item_clicked",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Could not initialise UI!");
        }

        let button = unsafe {
            owner
                .get_node("../StartButton")
                .unwrap()
                .assume_safe()
                .cast::<Button>()
                .unwrap()
        };
        if let Err(_why) = button.connect("pressed", owner, "", VariantArray::new_shared(), 0) {
            panic!("Failed to initialise UI");
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
            };
            let clicked_string = clicked_item.get_text(0).to_string().to_owned();
            match &clicked_string[..] {
                "Input Webcam:" => {
                    let camera_popup = unsafe {
                        owner
                            .get_node("../CameraPopup")
                            .unwrap()
                            .assume_safe()
                            .cast::<PopupMenu>()
                            .unwrap()
                    };
                    for item in 0..camera_popup.get_item_count() {
                        camera_popup.remove_item(item);
                    }
                    if camera_popup.is_visible() {
                        camera_popup.set_visible(false);
                    } else {
                        let rect = owner.get_custom_popup_rect();
                        let size = rect.size.to_vector();
                        let position = rect.origin.to_vector();

                        self.update_device_list();

                        let device_list = self.device_list.borrow().clone();
                        let mut id_cnt = 0;
                        for device in device_list.into_iter() {
                            camera_popup.add_item(device.description, id_cnt, -1);
                            id_cnt += 1;
                        }

                        camera_popup.set_position(position, true);
                        camera_popup.set_size(size, true);
                        camera_popup.set_visible(true);
                    }
                }
                "Webcam Resolution:" => {
                    if let Some(camera_device) = self.device_selected.borrow().clone() {
                        let device_serial = camera_device.serial.to_owned();
                        godot_print!(
                            "{},{},{}",
                            camera_device.vendor_id,
                            camera_device.product_id,
                            device_serial.clone().unwrap_or("default".to_string())
                        );
                        match crate::UVC.find_device(
                            Some(camera_device.vendor_id as i32),
                            Some(camera_device.product_id as i32),
                            device_serial.as_deref(),
                        ) {
                            Ok(dev) => {
                                let resolution_popup = unsafe {
                                    owner
                                        .get_node("../ResolutionPopup")
                                        .unwrap()
                                        .assume_safe()
                                        .cast::<PopupMenu>()
                                        .unwrap()
                                };
                                for item in 0..resolution_popup.get_item_count() {
                                    resolution_popup.remove_item(item);
                                }
                                if resolution_popup.is_visible() {
                                    resolution_popup.set_visible(false);
                                } else {
                                    let rect = owner.get_custom_popup_rect();
                                    let size = rect.size.to_vector();
                                    let position = rect.origin.to_vector();

                                    match dev.open() {
                                        Ok(handler) => {
                                            let mut counter = 0;
                                            let mut resolutions: Vec<String> = Vec::new();
                                            for format in handler.supported_formats() {
                                                for frame in format.supported_formats() {
                                                    let resolution_string = format!(
                                                        "{}x{}",
                                                        frame.width(),
                                                        frame.height()
                                                    );
                                                    if !resolutions.contains(&resolution_string) {
                                                        resolutions.push(resolution_string);
                                                    }
                                                }
                                            }
                                            for label in resolutions {
                                                resolution_popup.add_item(label, counter, 1);
                                                counter += 1;
                                            }
                                        }
                                        Err(why) => {
                                            godot_print!("{}", why);
                                        }
                                    }

                                    resolution_popup.set_size(size, true);
                                    resolution_popup.set_position(position, true);
                                }
                            }
                            Err(why) => {
                                godot_print!("{}", why);
                            }
                        }
                    }
                }
                "Webcam Frame Rate:" => {}
                "Webcam Video Format:" => {}
                _ => {
                    return;
                }
            }
        }
    }

    #[export]
    pub fn on_camera_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        let camera_popup = unsafe {
            owner
                .get_node("../CameraPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        let clicked_item = unsafe {
            owner
                .assume_shared()
                .assume_safe()
                .get_edited()
                .unwrap()
                .assume_safe()
        };
        let clicked_popup = camera_popup
            .get_item_text(camera_popup.get_item_index(id as i64))
            .to_string();
        for desc in self.device_list.borrow().clone().into_iter() {
            if desc.description == clicked_popup {
                let send_desc = desc.clone();
                clicked_item.set_text(1, desc.description);
                *self.device_selected.borrow_mut() = Some(send_desc);
            }
        }
    }

    #[export]
    pub fn on_framerate_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        let framerate_popup = unsafe {
            owner
                .get_node("../FrameratePopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        // TODO: Get USB device from thing and open device
        let clicked_item = unsafe {
            owner
                .assume_shared()
                .assume_safe()
                .get_edited()
                .unwrap()
                .assume_safe()
        };
        let clicked_popup = framerate_popup
            .get_item_text(framerate_popup.get_item_index(id as i64))
            .to_string();
    }

    #[export]
    pub fn on_video_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        let video_popup = unsafe {
            owner
                .get_node("../VideoPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        // TODO: Get USB device from thing and open device
        let clicked_item = unsafe {
            owner
                .assume_shared()
                .assume_safe()
                .get_edited()
                .unwrap()
                .assume_safe()
        };
        let clicked_popup = video_popup
            .get_item_text(video_popup.get_item_index(id as i64))
            .to_string();
    }

    #[export]
    pub fn on_resolution_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        let resolution_popup = unsafe {
            owner
                .get_node("../ResolutionPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        // TODO: Get USB device from thing and open device
        let clicked_item = unsafe {
            owner
                .assume_shared()
                .assume_safe()
                .get_edited()
                .unwrap()
                .assume_safe()
        };
        let clicked_popup = resolution_popup
            .get_item_text(resolution_popup.get_item_index(id as i64))
            .to_string();
    }

    #[export]
    pub fn on_start_button_pressed(&self, owner: TRef<Tree>) {
        // emit signal to viewport to update its camera if different
    }

    fn get_device_list() -> Vec<DeviceHolder> {
        let mut dev_list: Vec<DeviceHolder> = Vec::new();
        let uvc_device_list = match crate::UVC.devices() {
            Ok(dev) => dev,
            Err(_why) => {
                panic!("Could not get devices!");
                //TODO: Show error message when cannot get device list
            }
        };

        let usb_device_list = enumerate();
        for uvc_device in uvc_device_list {
            for usb_device in usb_device_list.clone() {
                let dev: DeviceHolder = match DeviceHolder::from_devices(&usb_device, &uvc_device) {
                    Ok(device) => device,
                    Err(_why) => {
                        continue;
                    }
                };
                if !dev_list.contains(&dev) {
                    dev_list.push(dev);
                }
            }
        }
        dev_list
    }

    fn update_device_list(&self) {
        let mut device_list = WebcamInputEditor::get_device_list().clone();
        self.device_list.borrow_mut().clear();
        self.device_list.borrow_mut().append(&mut device_list);
    }
}
