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

use crate::processing::face_detector::detectors::util::{DetectorHardware, DetectorType};
use crate::processing::input_processor::InputProcessingThreadless;
use crate::util::camera::device_utils::Resolution;
use gdnative::{api::VSplitContainer, prelude::*, NativeClass};
use std::cell::RefCell;

#[derive(NativeClass)]
#[inherit(VSplitContainer)]
pub struct ViewportHolder {
    input_processer: RefCell<Option<InputProcessingThreadless<'static>>>,
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
            &mut owner.get_node("/root/Open2DH/Open2GHMainUINode/Panel/VBoxContainer/HSplitContainer/TabContainer/Input/GridContainer/VBoxContainer/Tree").unwrap().assume_safe()
        };
        // TODO: Deprecate
        if let Err(why) = emitter.connect(
            "new_input_processer_uvc",
            owner,
            "on_new_input_processer_uvc",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to connect signals: {}!", why.to_string())
        }
        // TODO: Deprecate
        if let Err(why) = emitter.connect(
            "new_input_processer_v4l",
            owner,
            "on_new_input_processer_v4l",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to connect signals: {}!", why.to_string())
        }

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
        // godot_print!("proc1");

        if let Some(input) = &*self.input_processer.borrow() {
            godot_print!("proc1");
            if let Err(why) = input.capture_and_record() {
                godot_print!("{}", why.to_string());
            }
            godot_print!("proc2");

            // I've built abstraction over abstraction and now I am
            // waddling through my own shit trying to figure out why
            // this shitty ass method call is freezing the
            // entire fucking program
            // EDIT - `Create a blocking iterator over the values received ...`
            // RTFM kids, it will save you hours of headache.

            let a = input.query_gotten_results();
            godot_print!("{}", a.len());
            if let Err(why) = input.capture_and_record() {
                godot_print!("{}", why.to_string());
            }
        }
    }

    #[export]
    pub fn on_kill_signal(&self, _owner: TRef<VSplitContainer>) {
        //     if let Some(mut input) = self.input_processer.replace(None) {
        //         input.kill();
        //     }
    }

    // #[export]
    // pub fn on_new_input_processer_uvc(
    //     &mut self,
    //     _owner: TRef<VSplitContainer>,
    //     res: Variant,
    //     fps: Variant,
    // ) {
    //     let _resolution = match Resolution::from_variant(&res) {
    //         Ok(r) => r,
    //         Err(_why) => panic!("Improper resolution format set!"),
    //     };
    //
    //     let _framerate = match fps.try_to_i64() {
    //         Some(fs) => fs as u32,
    //         None => panic!("Improper resolution format set!"),
    //     };
    //
    //     let _format = DeviceFormat::MJPEG;
    //
    //     let mut ret_bool = false;
    //
    //     let (dev_ven, dev_prod, dev_ser) = crate::CURRENT_DEVICE.with(|dev| match &*dev.borrow() {
    //         Some(dev) => {
    //             let (a, b, c) = match dev {
    //                 DeviceContact::UVCAM {
    //                     vendor_id,
    //                     product_id,
    //                     serial,
    //                 } => (
    //                     vendor_id.to_owned(),
    //                     product_id.to_owned(),
    //                     serial.to_owned(),
    //                 ),
    //                 DeviceContact::V4L2 { .. } => {
    //                     ret_bool = true;
    //                     (None, None, None)
    //                 }
    //             };
    //             (a, b, c)
    //         }
    //         None => {
    //             ret_bool = true;
    //             (None, None, None)
    //         }
    //     });
    //
    //     if ret_bool {
    //         return;
    //     }
    //
    //     let vendor = match dev_ven {
    //         Some(i) => Some(i32::from(i)),
    //         None => None,
    //     };
    //
    //     let product = match dev_prod {
    //         Some(i) => Some(i32::from(i)),
    //         None => None,
    //     };
    //
    //     let uvc_device = match UVCameraDevice::new(vendor, product, dev_ser) {
    //         Ok(d) => d,
    //         Err(why) => panic!("Error getting device: {}", why.to_string()),
    //     };
    //
    //     // start the input processer
    //     // let input_processer = match InputProcessing::new(uvc_device.get_inner()) {
    //     //     Ok(input) => input,
    //     //     Err(_) => panic!("Cannot start input processer!"),
    //     // };
    //     // let channel = input_processer.get_thread_output();
    //     // *self.input_processer.borrow_mut() = Some(input_processer);
    //     // *self.process_channel.borrow_mut() = Some(channel);
    // }
    //
    // #[export]
    // pub fn on_new_input_processer_v4l(
    //     &self,
    //     _owner: TRef<VSplitContainer>,
    //     res: Variant,
    //     fps: Variant,
    // ) {
    //     let mut ret_bool = false;
    //     let dev_locat = crate::CURRENT_DEVICE.with(|dev| match &*dev.borrow() {
    //         Some(dev) => {
    //             let mut temp = &PathIndex::Index(0);
    //             match dev {
    //                 DeviceContact::UVCAM { .. } => {
    //                     ret_bool = true;
    //                 }
    //                 DeviceContact::V4L2 { location } => {
    //                     temp = location;
    //                 }
    //             }
    //             temp.to_owned()
    //         }
    //         None => {
    //             ret_bool = true;
    //             PathIndex::Index(0)
    //         }
    //     });
    //
    //     if ret_bool {
    //         return;
    //     }
    //
    //     let v4l_device = match V4LinuxDevice::new_location(dev_locat) {
    //         Ok(d) => d,
    //         Err(_) => panic!("Error getting device!"),
    //     };
    //
    //     match Resolution::from_variant(&res) {
    //         Ok(r) => {
    //             if let Err(why) = v4l_device.set_resolution(&r) {
    //                 panic!("Improper resolution format set: {}!", why.to_string())
    //             }
    //         }
    //         Err(_) => panic!("Improper resolution format set!"),
    //     };
    //
    //     match fps.try_to_i64() {
    //         Some(fs) => {
    //             if fs > 0 {
    //                 if let Err(why) = v4l_device.set_framerate(&(fs as u32)) {
    //                     panic!("Improper framerate set: {}!", why.to_string())
    //                 }
    //             } else {
    //                 panic!("Improper Framerate set!")
    //             }
    //         }
    //         None => panic!("Improper resolution format set!"),
    //     };
    //
    //     v4l_device.set_camera_format(DeviceFormat::MJPEG);
    //
    //     godot_print!("a");
    //
    //     // start the input processer
    //     // let input_processer = match InputProcessing::new(v4l_device.get_inner()) {
    //     //     Ok(input) => input,
    //     //     Err(_) => panic!("Cannot start input processer!"),
    //     // };
    //
    //     // let channel = input_processer.get_thread_output();
    //     // *self.input_processer.borrow_mut() = Some(input_processer);
    //     // *self.process_channel.borrow_mut() = Some(channel);
    // }

    #[export]
    pub fn on_new_input_processer(
        &self,
        _owner: TRef<VSplitContainer>,
        name: Variant,
        res: Variant,
        fps: Variant,
    ) {
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

        let input_processer = match InputProcessingThreadless::from_device_contact(
            Some(device_name),
            device_contact,
            device_res,
            device_fps as u32,
            DetectorType::DLibFHOG,
            DetectorHardware::Cpu,
        ) {
            Ok(return_to_monke) => Some(return_to_monke),
            Err(why) => panic!("Could not generate InputProcesser: {}", why.to_string()),
        };
        godot_print!("set");

        *self.input_processer.borrow_mut() = input_processer;
        godot_print!("ret void");
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
