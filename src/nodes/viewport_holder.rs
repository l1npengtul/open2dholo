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

use crate::{processing::input_processor::InputProcesser, util::camera::device_utils::Resolution};
use gdnative::{api::VSplitContainer, prelude::*, NativeClass};
use std::cell::RefCell;

#[derive(NativeClass)]
#[inherit(VSplitContainer)]
pub struct ViewportHolder {
    input_processer: RefCell<Option<InputProcesser<'static>>>,
}

#[methods]
impl ViewportHolder {
    fn new(_owner: &VSplitContainer) -> Self {
        ViewportHolder {
            input_processer: RefCell::new(None),
        }
    }
    #[export]
    pub fn _ready(&self, owner: TRef<VSplitContainer>) {
        let emitter = unsafe {
            &mut owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HSplitContainer/TabContainer/Input/GridContainer/VBoxContainer/Tree").unwrap().assume_safe()
        };
        if let Err(why) = emitter.connect(
            "new_input_processer",
            owner,
            "on_new_input_processer",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to connect signals: {}!", why.to_string())
        }

        if let Err(why) = emitter.connect(
            "kill_input_process",
            owner,
            "on_kill_signal",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to connect signals: {}!", why.to_string())
        }
    }

    #[export]
    fn set_world_same(&self, _owner: TRef<VSplitContainer>) {}

    // poll the channel to get the data
    #[export]
    pub fn _process(&self, _owner: TRef<VSplitContainer>, _delta: f32) {
        if let Some(input) = &*self.input_processer.borrow() {
            match input.capture_frame() {
                Ok(data) => {
                    println!("{}", data.len())
                }
                Err(why) => {
                    println!("{}", why);
                }
            }
        }
    }

    #[export]
    pub fn on_kill_signal(&self, _owner: TRef<VSplitContainer>) {
        //     if let Some(mut input) = self.input_processer.replace(None) {
        //         input.kill();
        //     }
    }

    #[export]
    pub fn on_new_input_processer(
        &self,
        _owner: TRef<VSplitContainer>,
        name: Variant,
        res: Variant,
        fps: Variant,
    ) {
        {
            // fill with input processor spawn logic

            let device_res = match Resolution::from_variant(&res) {
                Ok(r) => r,
                Err(_) => panic!("Improper resolution format set!"),
            };

            let device_fps = match fps.try_to_i64() {
                Some(fs) => fs,
                None => panic!("Improper framerate format set!"),
            };

            let device_name = match name.try_to_string() {
                Some(n) => n,
                None => "".to_string(),
            };

            let device_contact = crate::CURRENT_DEVICE.with(|dev| dev.borrow().clone().unwrap());
            godot_print!("input_proc");

            let input_processer = match InputProcesser::from_device_contact(
                Some(device_name),
                device_contact,
                device_res,
                device_fps as u32,
                DetectorType::DLibFhog,
                DetectorHardware::Cpu,
            ) {
                Ok(return_to_monke) => Some(return_to_monke),
                Err(why) => panic!("Could not generate InputProcesser: {}", why.to_string()),
            };
            *self.input_processer.borrow_mut() = input_processer;
        }
    }

    fn kill_input_processer(&mut self) {
        // if let Some(processer) = self.input_processer.get_mut() {
        //     // processer.kill();
        // }
    }
}

impl<'a> Drop for ViewportHolder {
    fn drop(&mut self) {
        // self.kill_input_processer();
    }
}
