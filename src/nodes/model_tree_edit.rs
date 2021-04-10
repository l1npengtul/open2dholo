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

use crate::nodes::util::create_editable_item;
use gdnative::{
    api::{tree::Tree, tree_item::TreeItem},
    prelude::*,
    NativeClass,
};

// imagine if l1npengtul was a real thing in real life
// would be scary TBH
// ~~that's why i am an illusion created by the south korean government running on JaVM - Jar Virtualisation Module~~
// l1npengtul exists inside a jar virtual machine, he is lie

#[derive(NativeClass)]
#[inherit(Tree)]
pub struct ModelTreeEditor;

#[methods]
impl ModelTreeEditor {
    fn new(_owner: &Tree) -> Self {
        ModelTreeEditor
    }
    #[export]
    fn _ready(&self, owner: &Tree) {
        let root_item: &TreeItem = unsafe {
            &*owner
                .create_item(owner.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };

        // TODO: Less .unwrap() more error handle

        owner.set_hide_root(true);
        owner.set_columns(2); // 2 Columns - One for the name, other for the editable value

        // Tree node for the X,Y,Z offset of the model until i can implement a better system like a scene editor
        // TODO
        let model_offset_editor: &TreeItem = unsafe {
            &*owner
                .create_item(root_item.assume_shared(), 1)
                .unwrap()
                .assume_safe()
        }; // god this is ugly
        model_offset_editor.set_text(0, "Model Offset");
        model_offset_editor.set_text_align(0, TreeItem::ALIGN_CENTER);
        // X Modifier
        let model_offset_editor_x: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };
        create_editable_item(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_x),
            "X Offset",
        );
        // Y Modifier
        let model_offset_editor_y: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 3)
                .unwrap()
                .assume_safe()
        };
        create_editable_item(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_y),
            "Y Offset",
        );
        // Z Modifier
        let model_offset_editor_z: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 4)
                .unwrap()
                .assume_safe()
        };
        create_editable_item(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_z),
            "Z Offset",
        );
    }
}
