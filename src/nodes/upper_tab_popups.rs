//     Open2DH - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
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
    api::{FileDialog, MenuButton, PopupMenu},
    methods,
    prelude::*,
    NativeClass,
};
use std::cell::RefCell;

#[derive(NativeClass)]
#[inherit(MenuButton)]
pub struct FileMenuButton {
    previous_file_path: RefCell<String>,
}

// TODO: signal to connect to Viewport and change model
#[methods]
impl FileMenuButton {
    fn new(_owner: &MenuButton) -> Self {
        FileMenuButton {
            previous_file_path: RefCell::new("res://".to_string()),
        }
    }
    #[export]
    fn _ready(&self, owner: TRef<MenuButton>) {
        let popupmenu = unsafe { &*owner.get_popup().unwrap().assume_safe() };
        popupmenu.add_item("Open Model", 0, -1);
        popupmenu.connect(
            "id_pressed",
            self,
            "on_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0,
        );
        let filedialog = unsafe {
            &*owner
                .get_node("ModelFileDialog")
                .unwrap()
                .assume_safe()
                .cast::<FileDialog>()
                .unwrap()
        };
        filedialog.connect(
            "dir_selected",
            self,
            "on_directory_selected_filedialog",
            VariantArray::new_shared(),
            0,
        );
        filedialog.connect(
            "file_selected",
            self,
            "on_file_selected_filedialog",
            VariantArray::new_shared(),
            0,
        );
    }

    #[export]
    pub fn on_popupmenu_button_clicked(&self, owner: TRef<MenuButton>, id: i32) {
        match id {
            0 => {
                let filedialog = unsafe {
                    &*owner
                        .get_node("ModelFileDialog")
                        .unwrap()
                        .assume_safe()
                        .cast::<FileDialog>()
                        .unwrap()
                };
                filedialog.set_visible(true);
            }
            _ => {}
        }
    }

    #[export]
    pub fn on_directory_selected_filedialog(&self, _owner: TRef<MenuButton>, dir: String) {
        *self.previous_file_path.borrow_mut() = dir;
    }

    #[export]
    pub fn on_file_selected_filedialog(&self, owner: TRef<MenuButton>, file: String) {
        // TODO: Emit Signal

        let filedialog = unsafe {
            &*owner
                .get_node("ModelFileDialog")
                .unwrap()
                .assume_safe()
                .cast::<FileDialog>()
                .unwrap()
        };
        filedialog.set_visible(false);
    }
}

#[derive(NativeClass)]
#[inherit(MenuButton)]
pub struct EditMenuButton;

#[methods]
impl EditMenuButton {
    fn new(_owner: &MenuButton) -> Self {
        EditMenuButton
    }
    #[export]
    fn _ready(&self, owner: TRef<MenuButton>) {
        let popupmenu = unsafe { &*owner.get_popup().unwrap().assume_safe() };
        popupmenu.add_item("Open Settings", 0, -1);
        popupmenu.connect(
            "id_pressed",
            self,
            "on_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0,
        );
    }

    #[export]
    pub fn on_popupmenu_button_clicked(&self, owner: TRef<MenuButton>, id: i32) {
        match id {
            0 => {}
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
        let popupmenu = unsafe { &*owner.get_popup().unwrap().assume_safe() };
        popupmenu.add_item("Open Docs", 0, -1); // TODO: Fix Nonexistant Docs
        popupmenu.add_separator("");
        popupmenu.add_item("About", 1, -1);
        popupmenu.connect(
            "id_pressed",
            self,
            "on_popupmenu_button_clicked",
            VariantArray::new_shared(),
            0,
        );
    }

    #[export]
    pub fn on_popupmenu_button_clicked(&self, owner: TRef<MenuButton>, id: i32) {
        match id {
            0 => {}
            1 => {}
            _ => {}
        }
    }
}
