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
    localize_path,
    processing::input_processor::InputProcesser,
    show_error,
    util::{
        camera::device_utils::{DeviceConfig, DeviceFormat, PossibleDevice, Resolution},
        misc::{Backend, BackendConfig},
    },
    wtf,
};
use gdnative::{api::VSplitContainer, prelude::*, NativeClass};
use std::cell::RefCell;

#[derive(NativeClass)]
#[inherit(VSplitContainer)]
#[register_with(Self::register_signals)]
pub struct ViewportHolder {
    input_processer: RefCell<Option<InputProcesser>>,
}

#[methods]
impl ViewportHolder {
    fn register_signals(builder: &ClassBuilder<Self>) {
        let mut default_68pt_vec = Vec::new();
        let vector2 = Vector2::new(0_f32, 0_f32);
        for _ in 0..68 {
            default_68pt_vec.push(vector2);
        }

        builder.add_signal(Signal {
            name: "new_processed_frame_68pt",
            args: &[SignalArgument {
                name: "point_array_68",
                default: Variant::from_vector2_array(&TypedArray::from_vec(default_68pt_vec)),
                export_info: ExportInfo::new(VariantType::Vector2Array),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        builder.add_signal(Signal {
            name: "model_load_start",
            args: &[SignalArgument {
                name: "path",
                default: Variant::from_str("res://"),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    fn new(_owner: &VSplitContainer) -> Self {
        ViewportHolder {
            input_processer: RefCell::new(None),
        }
    }
    #[export]
    pub fn _ready(&self, owner: TRef<VSplitContainer>) {
        let emitter_tree = unsafe {
            &mut owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HSplitContainer/TabContainer/Input/GridContainer/VBoxContainer/Tree").unwrap().assume_safe()
        };

        let emitter_loader = unsafe {
            &mut owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HBoxContainer/HBoxContainer/File").unwrap().assume_safe()
        };

        wtf!(emitter_tree.connect(
            "new_input_processer",
            owner,
            "on_new_input_processer",
            VariantArray::new_shared(),
            0,
        ));

        wtf!(emitter_tree.connect(
            "kill_input_process",
            owner,
            "on_kill_signal",
            VariantArray::new_shared(),
            0,
        ));

        wtf!(emitter_loader.connect(
            "new_model_load",
            owner,
            "on_new_model_load",
            VariantArray::new_shared(),
            0,
        ));

        wtf!(emitter_loader.connect(
            "new_tscn_model_load",
            owner,
            "on_new_tscn_mdl_load",
            VariantArray::new_shared(),
            0,
        ));
    }

    #[export]
    fn set_world_same(&self, _owner: TRef<VSplitContainer>) {}

    // poll the channel to get the data
    #[export]
    pub fn _process(&self, owner: TRef<VSplitContainer>, _delta: f32) {
        if let Some(input) = &*self.input_processer.borrow() {
            let results = input.query_gotten_results();
            for pkt in results {
                let mut variant_arr: Vector2Array = Vector2Array::new();
                for pt in pkt.landmarks {
                    variant_arr.push(Vector2::new(pt.x() as f32, pt.y() as f32))
                }
                owner.emit_signal(
                    "new_processed_frame_68pt",
                    &[Variant::from_vector2_array(&variant_arr)],
                );
            }
        }
    }

    #[export]
    pub fn on_kill_signal(&self, _owner: TRef<VSplitContainer>) {
        //if let Some(mut input) = self.input_processer.replace(None) {
        //    input.kill();
        //}
    }

    #[export]
    pub fn on_new_input_processer(
        &self,
        _owner: TRef<VSplitContainer>,
        _name: Variant,
        res: Variant,
        fps: Variant,
    ) {
        {
            // fill with input processor spawn logic
            // TODO: Allow regeneration of face processer

            let device_res = match Resolution::from_variant(&res) {
                Ok(r) => r,
                Err(_) => panic!("Improper resolution format set!"),
            };

            let device_fps = match fps.try_to_i64() {
                Some(fs) => fs,
                None => panic!("Improper framerate format set!"),
            };

            // TODO: Get backend config from backend settings panel
            let backend = BackendConfig::new(device_res, Backend::Dlib);

            let device_contact = crate::CURRENT_DEVICE.with(|dev| dev.borrow().clone().unwrap());

            let device_exists = { self.input_processer.borrow().is_some() };

            if device_exists {
                let dev_cfg: DeviceConfig = PossibleDevice::from_device_contact(
                    device_contact,
                    device_res,
                    device_fps as u32,
                    DeviceFormat::MJpeg,
                )
                .into();
                wtf!(self
                    .input_processer
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .set_device_cfg(dev_cfg));
            } else {
                let input_processer = match InputProcesser::from_device_contact(
                    device_contact,
                    device_res,
                    device_fps as u32,
                    backend,
                ) {
                    Ok(return_to_monke) => Some(return_to_monke),
                    Err(why) => panic!("Could not generate InputProcesser: {}", why.to_string()),
                };
                *self.input_processer.borrow_mut() = input_processer;
            }
        }
    }

    #[export]
    pub fn on_new_model_load(&self, owner: TRef<VSplitContainer>, model_path: Variant) {
        let string_path = match GodotString::from_variant(&model_path) {
            Ok(gdstr) => gdstr.to_string(),
            Err(why) => {
                show_error!("Could not load model", why.to_string());
                return;
            }
        };
        self.emit_loaded(owner, string_path);
    }

    #[export]
    pub fn on_new_tscn_mdl_load(&self, owner: TRef<VSplitContainer>, model_path: Variant) {
        let string_path = match GodotString::from_variant(&model_path) {
            Ok(gdstr) => localize_path!(gdstr),
            Err(why) => {
                show_error!("Could not load model", why.to_string());
                return;
            }
        };
        self.emit_loaded(owner, string_path);
    }

    #[export]
    fn emit_loaded(&self, owner: TRef<VSplitContainer>, mdl_path: String) {
        owner.emit_signal("model_load_start", &[Variant::from_str(mdl_path)]);
    }
}
