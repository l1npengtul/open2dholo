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

use crate::nodes::util::{create_editable_item, create_editable_range, get_immidiate_treeitems};
use gdnative::{
    api::{tree::Tree, tree_item::TreeItem},
    prelude::*,
    NativeClass,
};
use std::cell::Cell;

// imagine if l1npengtul was a real thing in real life
// would be scary TBH

#[derive(NativeClass)]
#[inherit(Tree)]
#[register_with(Self::register_signals)]
pub struct ModelTreeEditor {
    x_offset: Cell<f64>,
    y_offset: Cell<f64>,
    z_offset: Cell<f64>,
    x_rotate: Cell<f64>, // degrees
    y_rotate: Cell<f64>, // degrees
    z_rotate: Cell<f64>, // degrees
}

#[methods]
impl ModelTreeEditor {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "model_transform_change",
            args: &[
                SignalArgument {
                    name: "xyz_transform",
                    default: Variant::from_vector3(&Vector3::zero()),
                    export_info: ExportInfo::new(VariantType::Vector3),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "xyz_angle",
                    default: Variant::from_vector3(&Vector3::zero()),
                    export_info: ExportInfo::new(VariantType::Vector3),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        })
    }
    fn new(_owner: &Tree) -> Self {
        ModelTreeEditor {
            x_offset: Cell::new(0.0),
            y_offset: Cell::new(0.0),
            z_offset: Cell::new(0.0),
            x_rotate: Cell::new(0.0),
            y_rotate: Cell::new(0.0),
            z_rotate: Cell::new(0.0),
        }
    }
    #[export]
    fn _ready(&self, owner: TRef<Tree>) {
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
        let model_offset_editor_x_offset: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };
        create_editable_item(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_x_offset),
            "X Offset",
        );
        // Y Modifier
        let model_offset_editor_y_offset: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };
        create_editable_item(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_y_offset),
            "Y Offset",
        );
        // Z Modifier
        let model_offset_editor_z_offset: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 0)
                .unwrap()
                .assume_safe()
        };
        create_editable_item(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_z_offset),
            "Z Offset",
        );

        // Rotations

        let model_offset_editor_x_rot: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };
        create_editable_range(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_x_rot),
            "X Rotation",
            -360.0,
            360.0,
            0.01,
        );

        let model_offset_editor_y_rot: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };
        create_editable_range(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_y_rot),
            "Y Rotation",
            -360.0,
            360.0,
            0.01,
        );

        let model_offset_editor_z_rot: &TreeItem = unsafe {
            &*owner
                .create_item(model_offset_editor.assume_shared(), 2)
                .unwrap()
                .assume_safe()
        };
        create_editable_range(
            <&gdnative::api::TreeItem>::clone(&model_offset_editor_z_rot),
            "Z Rotation",
            -360.0,
            360.0,
            0.01,
        );

        // add signal
        if let Err(why) = owner.connect(
            "item_edited",
            owner,
            "on_item_edited",
            VariantArray::new_shared(),
            0,
        ) {
            panic!(format!("Failed to initialize UI: {}", why.to_string()))
        }
    }

    #[export]
    fn on_item_edited(&self, owner: TRef<Tree>) {
        // validate x,y,z offsets

        // sift through every item in the tree
        // we know that every item is only 1 deep
        let root = unsafe {
            owner.get_root().unwrap().assume_safe()
        };
        let mut treeitems: Vec<Ref<TreeItem, Shared>> = get_immidiate_treeitems(owner, root);
        
    }

    
}
