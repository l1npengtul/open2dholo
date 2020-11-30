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

use crate::util::camera::{device_utils::DeviceFormat, webcam::Webcam};
use crate::{
    nodes::editor_tabs::util::create_custom_editable_item,
    util::camera::{device_utils::Resolution, enumerate::enumerate_devices},
};
use gdnative::{
    api::{
        popup_menu::PopupMenu,
        tree::Tree,
        tree_item::{TreeCellMode, TreeItem},
    },
    prelude::*,
    NativeClass,
};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(NativeClass)]
#[inherit(Tree)]
pub struct WebcamInputEditor {
    device_list: RefCell<HashMap<String, Box<dyn Webcam>>>,
    device_selected: RefCell<Option<Box<dyn Webcam>>>,
    resolution_selected: RefCell<Option<Resolution>>,
    format_selected: RefCell<Option<DeviceFormat>>,
    fps_selected: RefCell<Option<i32>>,
}

#[methods]
impl WebcamInputEditor {
    fn new(_owner: &Tree) -> Self {
        let dev_list = RefCell::new(enumerate_devices().unwrap_or(HashMap::new()));
        WebcamInputEditor {
            device_list: dev_list,
            device_selected: RefCell::new(None),
            resolution_selected: RefCell::new(None),
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

        let format_popup = unsafe {
            owner
                .get_node("../FormatPopup")
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        format_popup.set_visible(false);
        if let Err(_why) = format_popup.connect(
            "id_pressed",
            owner,
            "on_format_popup_menu_clicked",
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
        create_custom_editable_item(owner, root_item, "Webcam Video Format:", 3);
        create_custom_editable_item(owner, root_item, "Webcam Resolution:", 4);
        create_custom_editable_item(owner, root_item, "Webcam Frame Rate:", 5);

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
                    camera_popup.clear();
                    if camera_popup.is_visible() {
                        camera_popup.set_visible(false);
                    } else {
                        let rect = owner.get_custom_popup_rect();
                        let size = rect.size.to_vector();
                        let position = rect.origin.to_vector();

                        self.update_device_list();

                        let mut id_cnt = 0;
                        for (device_name, _device) in self.device_list.borrow_mut().iter() {
                            camera_popup.add_item(device_name, id_cnt, -1);
                            id_cnt += 1;
                        }

                        camera_popup.set_position(position, true);
                        camera_popup.set_size(size, true);
                        camera_popup.set_visible(true);
                    }
                }
                "Webcam Resolution:" => match self.device_selected.borrow().as_deref() {
                    Some(camera) => {
                        let resolution_popup = unsafe {
                            owner
                                .get_node("../ResolutionPopup")
                                .unwrap()
                                .assume_safe()
                                .cast::<PopupMenu>()
                                .unwrap()
                        };
                        resolution_popup.clear();
                        if resolution_popup.is_visible() {
                            resolution_popup.set_visible(false);
                        } else {
                            let rect = owner.get_custom_popup_rect();
                            let size = rect.size.to_vector();
                            let position = rect.origin.to_vector();
                            let mut counter = 0;

                            if let Some(fmt) = self.format_selected.borrow_mut().to_owned() {
                                camera.set_camera_foramt(fmt);
                            }
                            match camera.get_supported_resolutions() {
                                Ok(res_list) => {
                                    for res in res_list {
                                        resolution_popup.add_item(res.to_string(), counter, 1);
                                        counter += 1;
                                    }
                                }
                                Err(why) => {
                                    godot_print!("{}", why)
                                }
                            }
                            resolution_popup.set_size(size, true);
                            resolution_popup.set_position(position, true);
                            resolution_popup.set_visible(true);
                        }
                    }
                    None => {
                        godot_print!("No Camera!");
                    }
                },
                "Webcam Frame Rate:" => match self.device_selected.borrow().as_deref() {
                    Some(camera) => {
                        let fps_popup = unsafe {
                            owner
                                .get_node("../FrameratePopup")
                                .unwrap()
                                .assume_safe()
                                .cast::<PopupMenu>()
                                .unwrap()
                        };
                        fps_popup.clear();
                        if fps_popup.is_visible() {
                            fps_popup.set_visible(false);
                        } else {
                            let rect = owner.get_custom_popup_rect();
                            let size = rect.size.to_vector();
                            let position = rect.origin.to_vector();
                            let mut counter = 0;

                            if let Some(fmt) = self.format_selected.borrow_mut().to_owned() {
                                camera.set_camera_foramt(fmt);
                            }
                            if let Some(resolution) =
                                self.resolution_selected.borrow_mut().to_owned()
                            {
                                match camera.get_supported_framerate(resolution) {
                                    Ok(res_list) => {
                                        for res in res_list {
                                            fps_popup.add_item(res.to_string(), counter, 1);
                                            counter += 1;
                                        }
                                    }
                                    Err(why) => {
                                        godot_print!("{}", why)
                                    }
                                }
                            }

                            fps_popup.set_size(size, true);
                            fps_popup.set_position(position, true);
                            fps_popup.set_visible(true);
                        }
                    }
                    None => {
                        godot_print!("No Camera!");
                    }
                },
                "Webcam Video Format:" => match self.device_selected.borrow().as_deref() {
                    Some(device) => {
                        let format_popup = unsafe {
                            owner
                                .get_node("../FormatPopup")
                                .unwrap()
                                .assume_safe()
                                .cast::<PopupMenu>()
                                .unwrap()
                        };
                        format_popup.clear();
                        if format_popup.is_visible() {
                            format_popup.set_visible(false);
                        } else {
                            if let Ok(resolutions) = device.get_supported_resolutions() {
                                if let Some(res) = resolutions.get(0) {
                                    let rect = owner.get_custom_popup_rect();
                                    let size = rect.size.to_vector();
                                    let position = rect.origin.to_vector();
                                    let mut counter = 0;
                                    if let Ok(dev_fmt) = device.get_supported_formats(res.clone()) {
                                        for fourcc in dev_fmt {
                                            format_popup.add_item(fourcc.to_string(), counter, 1);
                                            counter += 1;
                                        }
                                    }

                                    format_popup.set_size(size, true);
                                    format_popup.set_position(position, true);
                                    format_popup.set_visible(true);
                                }
                            }
                        }
                    }
                    None => {
                        godot_print!("No Camera!");
                    }
                },
                _ => {
                    return;
                }
            }
        }
    }

    #[export]
    pub fn on_camera_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        self.clear_other_fields(owner, "camera");
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
        // set selected device
        match self.device_list.borrow_mut().remove(&clicked_popup) {
            Some(device) => {
                *self.device_selected.borrow_mut() = Some(device);
                clicked_item.set_text(1, clicked_popup);
            }
            None => {
                godot_print!("Error!");
            }
        }
    }

    #[export]
    pub fn on_resolution_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        self.clear_other_fields(owner, "res");

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
        let resolutin_string_vec: Vec<&str> = clicked_popup.split("x").into_iter().collect();
        let res = Resolution {
            x: resolutin_string_vec.get(0).unwrap().parse::<u32>().unwrap(),
            y: resolutin_string_vec.get(1).unwrap().parse::<u32>().unwrap(),
        };
        *self.resolution_selected.borrow_mut() = Some(res);
        clicked_item.set_text(1, clicked_popup);
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
        let fps_int = clicked_popup.parse::<i32>().unwrap();
        *self.fps_selected.borrow_mut() = Some(fps_int);
        clicked_item.set_text(1, clicked_popup);
    }

    #[export]
    pub fn on_format_popup_menu_clicked(&self, owner: TRef<Tree>, id: i32) {
        self.clear_other_fields(owner, "format");
        let format_popup = unsafe {
            owner
                .get_node("../FormatPopup")
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
        let clicked_popup = format_popup
            .get_item_text(format_popup.get_item_index(id as i64))
            .to_string();
        match &clicked_popup.clone().to_ascii_lowercase().to_owned()[..] {
            "yuyv" => {
                *self.format_selected.borrow_mut() = Some(DeviceFormat::YUYV);
                clicked_item.set_text(1, clicked_popup);
            }
            "mjpg" | "mjpeg" => {
                *self.format_selected.borrow_mut() = Some(DeviceFormat::MJPEG);
                clicked_item.set_text(1, clicked_popup);
            }
            _ => {}
        }
    }

    #[export]
    pub fn on_start_button_pressed(&self, _owner: TRef<Tree>) {
        self.update_device_list();
    }

    // updates the device list to look for new devices, etc
    fn update_device_list(&self) {
        self.device_list.borrow_mut().clear();
        match enumerate_devices() {
            Some(new_list) => {
                *self.device_list.borrow_mut() = new_list;
            }
            None => {
                // do nothing
            }
        };
    }

    // Clears fields below it and its associated value
    // Camera => Update Dev List, Clears FMT, RES, FPS
    // Format => Clears Res, FPS
    // Res => Clears FPS
    // FPS => N/A
    // Note that `item` refers to the thing to NOT clear
    fn clear_other_fields(&self, owner: TRef<Tree>, item: &str) {
        match item {
            "camera" => {
                self.update_device_list();
                *self.device_selected.borrow_mut() = None;
                *self.format_selected.borrow_mut() = None;
                *self.resolution_selected.borrow_mut() = None;
                *self.fps_selected.borrow_mut() = None;
                let mut child = unsafe {
                    owner
                        .get_root()
                        .unwrap()
                        .assume_safe()
                        .get_children()
                        .unwrap()
                        .assume_safe()
                };
                let mut clearable = false;
                loop {
                    // see if the child is a custom tree item
                    if child.get_text(0).to_string() != "Input Webcam:".to_string()
                        && child.get_cell_mode(1) == TreeCellMode::CUSTOM
                        && clearable
                    {
                        child.set_text(1, "");
                    } else if child.get_text(0).to_string() == "Input Webcam:".to_string() {
                        clearable = true;
                    }
                    if let Some(a) = child.get_next() {
                        child = unsafe { a.assume_safe() };
                    } else {
                        break;
                    }
                }
            }
            "format" => {
                self.update_device_list();
                *self.resolution_selected.borrow_mut() = None;
                *self.fps_selected.borrow_mut() = None;
                let mut child = unsafe {
                    owner
                        .get_root()
                        .unwrap()
                        .assume_safe()
                        .get_children()
                        .unwrap()
                        .assume_safe()
                };
                let mut clearable = false;
                loop {
                    // see if the child is a custom tree item
                    if child.get_text(0).to_string() != "Webcam Video Format:".to_string()
                        && child.get_cell_mode(1) == TreeCellMode::CUSTOM
                        && clearable
                    {
                        child.set_text(1, "");
                    } else if child.get_text(0).to_string() == "Webcam Video Format:".to_string() {
                        clearable = true;
                    }
                    if let Some(a) = child.get_next() {
                        child = unsafe { a.assume_safe() };
                    } else {
                        break;
                    }
                }
            }
            "res" => {
                self.update_device_list();
                *self.fps_selected.borrow_mut() = None;
                let mut child = unsafe {
                    owner
                        .get_root()
                        .unwrap()
                        .assume_safe()
                        .get_children()
                        .unwrap()
                        .assume_safe()
                };
                let mut clearable = false;
                loop {
                    // see if the child is a custom tree item
                    if child.get_text(0).to_string() != "Webcam Resolution:".to_string()
                        && child.get_cell_mode(1) == TreeCellMode::CUSTOM
                        && clearable
                    {
                        child.set_text(1, "");
                    } else if child.get_text(0).to_string() == "Webcam Resolution:".to_string() {
                        clearable = true;
                    }
                    if let Some(a) = child.get_next() {
                        child = unsafe { a.assume_safe() };
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
