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

use crate::{
    globalize_path,
    nodes::util::check_endswith_glb,
    show_error,
    util::misc::{MdlRefBuilder, ModelReference},
    wtf,
};
use dirs::home_dir;
use gdnative::{
    api::{MenuButton, PopupMenu, OS},
    methods,
    prelude::*,
    NativeClass,
};
use native_dialog::FileDialog as NativeFileDialog;
use std::{cell::RefCell, collections::HashMap, convert::TryInto};
use walkdir::WalkDir;

#[derive(NativeClass)]
#[inherit(MenuButton)]
#[register_with(Self::register_signals)]
pub struct FileMenuButton {
    previous_file_path: RefCell<String>,
    default_model_paths: RefCell<HashMap<String, ModelReference>>, // Name and Path
}

// TODO: signal to connect to Viewport and change model
#[methods]
impl FileMenuButton {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "new_model_load",
            args: &[SignalArgument {
                name: "model_path",
                default: Variant::from_str(""),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        builder.add_signal(Signal {
            name: "new_tscn_model_load",
            args: &[SignalArgument {
                name: "model_path",
                default: Variant::from_str(""),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }
    fn new(_owner: &MenuButton) -> Self {
        let home_dir = home_dir().map_or_else(
            || {
                let os = OS::godot_singleton();
                os.get_user_data_dir().to_string()
            },
            |h| {
                if let Ok(p) = h.into_os_string().into_string() {
                    p
                } else {
                    let os = OS::godot_singleton();
                    os.get_user_data_dir().to_string()
                }
            },
        );
        FileMenuButton {
            previous_file_path: RefCell::new(home_dir),
            default_model_paths: RefCell::new(HashMap::new()),
        }
    }
    #[export]
    fn _ready(&self, owner: TRef<MenuButton>) {
        let popupmenu = unsafe { &*owner.get_popup().unwrap().assume_safe() };
        popupmenu.add_item("Open Model From Filesystem", 0, -1);
        popupmenu.add_submenu_item("Open Default Model", "/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HBoxContainer/HBoxContainer/File/Default", 1);
        wtf!(popupmenu.connect(
            "id_pressed",
            owner,
            "on_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0
        ));

        let default_popupmenu = unsafe {
            &*owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HBoxContainer/HBoxContainer/File/Default").unwrap().assume_safe().cast::<PopupMenu>().unwrap()
        };

        // crawl the default model directory
        let default_model_global = globalize_path!("res://default_models");
        let mut file_hashmap: HashMap<String, ModelReference> = HashMap::new();
        for file in WalkDir::new(default_model_global)
            .min_depth(1)
            .into_iter()
            .flatten()
        {
            if check_endswith_glb(&file) {
                // get the filename without the ".glb" and raw path
                let filename = file
                    .file_name()
                    .to_str()
                    .unwrap_or("")
                    .strip_suffix(".glb")
                    .unwrap_or("")
                    .to_string();
                let full_path = file.path().as_os_str().to_str().unwrap_or("").to_string();
                let tscn_path = full_path.strip_suffix(".glb").unwrap().to_string() + ".tscn";
                let mut mdl = MdlRefBuilder::from_vrm_meta_json(full_path.clone())
                    .with_model_path(full_path)
                    .with_tscn_path(tscn_path);
                if mdl.check_empty_displayname() {
                    mdl = mdl.with_display_name(filename.to_string());
                }
                let model_ref = mdl.build();
                file_hashmap.insert(model_ref.display_name().clone(), model_ref);
            }
        }

        let mut names: Vec<&String> = file_hashmap
            .values()
            .into_iter()
            .map(ModelReference::display_name)
            .collect();
        names.sort();
        for (idx, name) in names.into_iter().enumerate() {
            default_popupmenu.add_item(name, idx.try_into().unwrap(), -1);
        }

        wtf!(default_popupmenu.connect(
            "id_pressed",
            owner,
            "on_default_model_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0
        ));

        popupmenu.add_separator("");
        popupmenu.add_item("Open Settigs", 2, -1);

        *self.default_model_paths.borrow_mut() = file_hashmap;
    }

    #[export]
    pub fn on_popupmenu_button_clicked(&self, owner: TRef<MenuButton>, id: i32) {
        match id {
            0 => {
                match NativeFileDialog::new()
                    .set_location(&*self.previous_file_path.borrow())
                    .add_filter("glTF Model", &["*.gltf", "*.glb"])
                    // .add_filter("VRM Model", &["*.vrm"]) // HAHA TFW GODOT NO DYNAMIC LOADING SUPPORT KEKW
                    // .add_filter("FBX Model", &["*.fbx"])
                    // .add_filter(~"Collada Model", &["*.dae"])
                    .show_open_single_file()
                {
                    Ok(path) => {
                        if let Some(p) = path {
                            match p.parent() {
                                Some(dir_path) => {
                                    let path_str =
                                        dir_path.as_os_str().to_os_string().into_string().unwrap();
                                    *self.previous_file_path.borrow_mut() = path_str.clone();
                                    owner.emit_signal(
                                        "new_model_load",
                                        &[Variant::from_str(path_str)],
                                    );
                                }
                                None => {
                                    let path_str = p.into_os_string().into_string().unwrap();
                                    *self.previous_file_path.borrow_mut() = path_str
                                }
                            }
                            // TODO: Loader emit signal
                        } else {
                            show_error!("Failed to open file", "File path doesn't exist!");
                        }
                    }
                    Err(why) => {
                        show_error!("Failed to open file", why);
                    }
                }
            }

            2 => {
                godot_print!("AAAA");
            }
            _ => {}
        }
    }

    #[export]
    pub fn on_default_model_popupmenu_button_clicked(&self, owner: TRef<MenuButton>, id: i32) {
        godot_print!("?");
        let default_popupmenu = unsafe {
            &*owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HBoxContainer/HBoxContainer/File/Default").unwrap().assume_safe().cast::<PopupMenu>().unwrap()
        };
        let selected_text = default_popupmenu.get_item_text(i64::from(id)).to_string();

        if let Some(mdl_ref) = self.default_model_paths.borrow().get(&selected_text) {
            owner.emit_signal(
                "new_tscn_model_load",
                &[Variant::from_str(mdl_ref.globalized_tscn_path())],
            );
        }
    }
}

#[derive(NativeClass)]
#[inherit(MenuButton)]
#[register_with(Self::register_signals)]
pub struct EditMenuButton;

#[methods]
impl EditMenuButton {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "settings_open",
            args: &[],
        })
    }
    fn new(_owner: &MenuButton) -> Self {
        EditMenuButton
    }
    #[export]
    fn _ready(&self, owner: TRef<MenuButton>) {
        let popupmenu = unsafe { &*owner.get_popup().unwrap().assume_safe() };
        popupmenu.add_item("Open Editor", 0, -1);

        wtf!(popupmenu.connect(
            "id_pressed",
            owner,
            "on_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0,
        ))
    }

    #[export]
    pub fn on_popupmenu_button_clicked(&self, _owner: TRef<MenuButton>, id: i32) {
        match id {
            // 0 => {}
            _ => {}
        }
    }
}

#[derive(NativeClass)]
#[inherit(MenuButton)]
pub struct HelpMenuButton;

// TODO: signal to connect to Viewport and change model
#[methods]
impl HelpMenuButton {
    fn new(_owner: &MenuButton) -> Self {
        HelpMenuButton
    }
    #[export]
    fn _ready(&self, owner: TRef<MenuButton>) {
        let popupmenu = unsafe {
            &*owner
                .get_popup()
                .unwrap()
                .assume_safe()
                .cast::<PopupMenu>()
                .unwrap()
        };
        popupmenu.add_item("Open Docs", 0, -1); // TODO: Fix Nonexistant Docs
        popupmenu.add_separator("");
        popupmenu.add_item("About", 1, -1);
        wtf!(popupmenu.connect(
            "id_pressed",
            owner,
            "on_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0,
        ))
    }

    #[export]
    pub fn on_popupmenu_button_clicked(&self, _owner: TRef<MenuButton>, _id: i32) {}
}
