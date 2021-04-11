//     Open2DHolo - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
//     Copyright (C) 2020-2021 l1npengtul
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

use std::cell::RefCell;
use std::collections::HashMap;

use gdnative::{
    api::{
        popup_menu::PopupMenu,
        tree::Tree,
        tree_item::{TreeCellMode, TreeItem},
    },
    prelude::*,
    NativeClass,
};

use crate::nodes::util::create_custom_editable_item;
use crate::util::camera::device_utils::DeviceContact;
use crate::util::camera::device_utils::{
    enumerate_cache_device, CachedDeviceList, DeviceFormat, PossibleDevice, Resolution,
};

#[derive(NativeClass)]
#[inherit(Tree)]
#[register_with(Self::register_signals)]
pub struct WebcamInputEditor {
    device_list: RefCell<HashMap<String, CachedDeviceList>>,
    device_selected: RefCell<Option<String>>,
    resolution_selected: RefCell<Option<Resolution>>,
    fps_selected: RefCell<Option<i32>>,
}

#[methods]
impl WebcamInputEditor {
    // register the signals to viewport we will need
    fn register_signals(builder: &ClassBuilder<Self>) {
        // we have to do this disgustingness becuase godot signals can only transport variants like why
        // TODO: deprecate both these signals and replace with our OpenCVCameraDevice abstraction
        builder.add_signal(Signal {
            name: "new_input_processer",
            args: &[
                // device name for pretty formatting
                SignalArgument {
                    name: "device_name",
                    default: Variant::from_str(""),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
                // resolution
                SignalArgument {
                    name: "resolution",
                    default: Variant::from_vector2(&Vector2::new(-1.0, -1.0)),
                    export_info: ExportInfo::new(VariantType::Vector2),
                    usage: PropertyUsage::DEFAULT,
                },
                // fps
                SignalArgument {
                    name: "framerate",
                    default: Variant::from_i64(-1),
                    export_info: ExportInfo::new(VariantType::I64),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        });

        // kill input processer sigbal
        builder.add_signal(Signal {
            name: "kill_input_process",
            args: &[],
        });
    }

    fn new(_owner: &Tree) -> Self {
        let dev_list = RefCell::new(enumerate_cache_device().unwrap_or_default());
        WebcamInputEditor {
            device_list: dev_list,
            device_selected: RefCell::new(None),
            resolution_selected: RefCell::new(None),
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

        let camera_settings_item: &TreeItem = unsafe {
            &*owner
                .create_item(root_item.assume_shared(), 6)
                .unwrap()
                .assume_safe()
        };

        camera_settings_item.set_disable_folding(false);
        camera_settings_item.set_collapsed(false);
        camera_settings_item.set_text(0, "Webcam Input Settings");
        camera_settings_item.set_text_align(0, TreeItem::ALIGN_CENTER);
        camera_settings_item.set_selectable(1, false);

        let face_detection_settings: &TreeItem = unsafe {
            &*owner
                .create_item(root_item.assume_shared(), 7)
                .unwrap()
                .assume_safe()
        };

        face_detection_settings.set_disable_folding(false);
        face_detection_settings.set_collapsed(true);
        face_detection_settings.set_text(0, "Face Detection Input Settigns");
        face_detection_settings.set_text_align(0, TreeItem::ALIGN_CENTER);
        face_detection_settings.set_selectable(1, false);

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

        create_custom_editable_item(owner, camera_settings_item, "Input Webcam:", 2);
        create_custom_editable_item(owner, camera_settings_item, "Webcam Resolution:", 4);
        create_custom_editable_item(owner, camera_settings_item, "Webcam Frame Rate:", 5);

        // 2: Where did 3 go?
        // 5: 4 8 3.
        // 4: you're next 2
        // 2: .
        // 5: ㅋㅋㅋㅋㅋㅋ

        create_custom_editable_item(owner, face_detection_settings, "Detector Hardware:", 8); // CPU, GPGPU(CUDA/ROCm)
        create_custom_editable_item(owner, face_detection_settings, "Detector Type:", 9); // DLIB_FHOG, DLIB_CNN, <insert other face reconizer here>

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
        if let Err(_why) = button.connect(
            "pressed",
            owner,
            "on_start_button_pressed",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to initialise UI");
        }
        button.set_disabled(true);
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
            let clicked_string = clicked_item.get_text(0).to_string();
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

                        for (id_cnt, (device_name, _device)) in
                            self.device_list.borrow_mut().iter().enumerate()
                        {
                            camera_popup.add_item(device_name, id_cnt as i64, -1);
                        }

                        camera_popup.set_position(position, true);
                        camera_popup.set_size(size, true);
                        camera_popup.set_visible(true);
                    }
                }
                "Webcam Resolution:" => match &*self.device_selected.borrow() {
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

                            let selected_cache_dev = match self.device_list.borrow().get(camera) {
                                Some(dev) => dev.to_owned(),
                                None => panic!("The device no longer exists!"),
                            };
                            let mut res_vec_sorted: Vec<Resolution> = Vec::new();
                            for res in selected_cache_dev.get_supported_mjpg().keys() {
                                res_vec_sorted.push(*res)
                            }
                            res_vec_sorted.sort();
                            for (id_cnt, res) in res_vec_sorted.into_iter().enumerate() {
                                resolution_popup.add_item(format!("{}", res), id_cnt as i64, -1);
                            }

                            // for selected_cache_dev.
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
                    Some(_camera) => {
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

                            if let Some(device_name) = &*self.device_selected.borrow() {
                                if let Some(device) = self.device_list.borrow().get(device_name) {
                                    if let Some(res) = *self.resolution_selected.borrow() {
                                        if let Some(framerate_list) =
                                            device.get_supported_mjpg().get(&res)
                                        {
                                            for (id_cnt, fps) in framerate_list.iter().enumerate() {
                                                fps_popup.add_item(
                                                    format!("{}", fps),
                                                    id_cnt as i64,
                                                    -1,
                                                );
                                            }
                                        }
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
                _ => (),
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
            .get_item_text(camera_popup.get_item_index(i64::from(id)))
            .to_string();
        // set selected device
        clicked_item.set_text(1, clicked_popup.clone());
        *self.device_selected.borrow_mut() = Some(clicked_popup);
        self.check_button_eligibility(owner);
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
            .get_item_text(resolution_popup.get_item_index(i64::from(id)))
            .to_string();
        let resolutin_string_vec: Vec<&str> = clicked_popup.split('x').into_iter().collect();
        let res = Resolution {
            x: resolutin_string_vec.get(0).unwrap().parse::<u32>().unwrap(),
            y: resolutin_string_vec.get(1).unwrap().parse::<u32>().unwrap(),
        };
        *self.resolution_selected.borrow_mut() = Some(res);
        clicked_item.set_text(1, clicked_popup);
        self.check_button_eligibility(owner);
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
            .get_item_text(framerate_popup.get_item_index(i64::from(id)))
            .to_string();
        let fps_int = clicked_popup.parse::<i32>().unwrap();
        *self.fps_selected.borrow_mut() = Some(fps_int);
        clicked_item.set_text(1, clicked_popup);
        self.check_button_eligibility(owner);
    }

    #[export]
    pub fn on_start_button_pressed(&self, owner: TRef<Tree>) {
        owner.emit_signal("kill_input_process", &[]);
        self.update_device_list();

        let dev = match self.device_list.borrow().get(
            &*self
                .device_selected
                .borrow()
                .as_ref()
                .unwrap_or(&"".to_string()),
        ) {
            Some(dev) => dev.to_owned(),
            None => return, // TODO: Global Error handler
        };

        let res = match &*self.resolution_selected.borrow() {
            Some(r) => *r,
            None => return,
        };

        let framerate = match &*self.fps_selected.borrow() {
            Some(r) => *r as u32,
            None => return,
        };

        let name = match &*self.device_selected.borrow() {
            Some(n) => n.clone(),
            None => return,
        };

        let possible = PossibleDevice::from_cached_device(dev, res, framerate, DeviceFormat::MJPEG);

        let resolution = Vector2::new(res.x as f32, res.y as f32);

        // crate::CURRENT_DEVICE.with(|device| *device.borrow_mut() = Some(device_contact));
        crate::CURRENT_DEVICE.with(|device| {
            *device.borrow_mut() = Some(DeviceContact::from_possible_device(&possible))
        });

        owner.emit_signal(
            "new_input_processer",
            &[
                Variant::from_str(name),
                Variant::from_vector2(&resolution),
                Variant::from_i64(i64::from(framerate)),
            ],
        );
    }

    #[export]
    pub fn check_button_eligibility(&self, owner: TRef<Tree>) {
        if self.device_selected.borrow_mut().is_some()
            && self.resolution_selected.borrow_mut().is_some()
            && self.fps_selected.borrow_mut().is_some()
        {
            let button = unsafe {
                owner
                    .get_node("../StartButton")
                    .unwrap()
                    .assume_safe()
                    .cast::<Button>()
                    .unwrap()
            };
            button.set_disabled(false);
        }
    }

    // updates the device list to look for new devices, etc
    fn update_device_list(&self) {
        self.device_list.borrow_mut().clear();
        if let Some(new_list) = enumerate_cache_device() {
            *self.device_list.borrow_mut() = new_list;
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
                    if child.get_text(0).to_string() != *"Input Webcam:"
                        && child.get_cell_mode(1) == TreeCellMode::CUSTOM
                        && clearable
                    {
                        child.set_text(1, "");
                    } else if child.get_text(0).to_string() == *"Input Webcam:" {
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
                    if child.get_text(0).to_string() != *"Webcam Resolution:"
                        && child.get_cell_mode(1) == TreeCellMode::CUSTOM
                        && clearable
                    {
                        child.set_text(1, "");
                    } else if child.get_text(0).to_string() == *"Webcam Resolution:" {
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
