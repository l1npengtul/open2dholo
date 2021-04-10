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

use gdnative::{
    api::{Control, Panel, VBoxContainer, OS},
    methods,
    prelude::*,
    NativeClass,
};

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Open2DHoloCtrl;

#[methods]
impl Open2DHoloCtrl {
    fn new(_owner: &Control) -> Self {
        Open2DHoloCtrl
    }

    #[export]
    fn _ready(&self, owner: TRef<Control>) {
        let root_viewport: &Viewport = unsafe {
            &*owner
                .get_parent()
                .unwrap()
                .assume_safe()
                .cast::<Viewport>()
                .unwrap()
        };
        if let Err(_why) = root_viewport.connect(
            "size_changed",
            owner,
            "on_size_change",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to initialise UI!");
        }

        // set the size at ready to avoid weird UI scaling on first boot
        self.on_size_change(owner);
    }
    #[export]
    pub fn on_size_change(&self, owner: TRef<Control>) {
        let root_viewport_size = OS::godot_singleton().window_size();
        let colorrect = unsafe {
            &*owner
                .get_node("ColorRect")
                .unwrap()
                .assume_safe()
                .cast::<ColorRect>()
                .unwrap()
        };
        let main_ui = unsafe {
            &*owner
                .get_node("Open2DHoloMainUINode")
                .unwrap()
                .assume_safe()
                .cast::<Control>()
                .unwrap()
        };
        let panel = unsafe {
            &*owner
                .get_node("Open2DHoloMainUINode/Panel")
                .unwrap()
                .assume_safe()
                .cast::<Panel>()
                .unwrap()
        };
        let vbox = unsafe {
            &*owner
                .get_node("Open2DHoloMainUINode/Panel/VBoxContainer")
                .unwrap()
                .assume_safe()
                .cast::<VBoxContainer>()
                .unwrap()
        };

        let vbox_size = Vector2::new(
            root_viewport_size.x - (vbox.position().x * 2.0),
            root_viewport_size.y - (vbox.position().y * 2.0),
        );

        owner.set_size(root_viewport_size, true);
        colorrect.set_size(root_viewport_size, true);
        main_ui.set_size(root_viewport_size, true);
        panel.set_size(root_viewport_size, true);
        vbox.set_size(vbox_size, true);
    }
}
