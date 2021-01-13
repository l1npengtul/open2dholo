use crate::util::camera::camera_device::{UVCameraDevice, V4LinuxDevice};
use crate::util::camera::device_utils::{DeviceContact, PathIndex};
use crate::util::camera::webcam::Webcam;
use crate::{
    processing::input_processer::InputProcessing,
    util::{
        camera::device_utils::{DeviceFormat, Resolution},
        packet::Processed,
    },
};
use downcast_rs::__std::os::raw::c_int;
use flume::Receiver;
use gdnative::{api::VSplitContainer, prelude::*, NativeClass};
use std::cell::RefCell;

#[derive(NativeClass)]
#[inherit(VSplitContainer)]
pub struct ViewportHolder {
    input_processer: RefCell<Option<InputProcessing>>,
    process_channel: RefCell<Option<Receiver<Processed>>>,
}

#[methods]
impl ViewportHolder {
    fn new(_owner: &VSplitContainer) -> Self {
        ViewportHolder {
            input_processer: RefCell::new(None),
            process_channel: RefCell::new(None),
        }
    }
    #[export]
    pub fn _ready(&self, owner: TRef<VSplitContainer>) {
        let emitter = unsafe {
            &mut owner.get_node("/root/Open2DH/Open2GHMainUINode/Panel/VBoxContainer/HSplitContainer/TabContainer/Input/GridContainer/VBoxContainer/Tree").unwrap().assume_safe()
        };
        if let Err(why) = emitter.connect(
            "new_input_processer_uvc",
            owner,
            "on_new_input_processer_uvc",
            VariantArray::new_shared(),
            0,
        ) {
            panic!("Failed to connect signals: {}!", why.to_string())
        }
        if let Err(why) = emitter.connect(
            "new_input_processer_v4l",
            owner,
            "on_new_input_processer_v4l",
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
        if let Some(a) = &*self.process_channel.borrow() {
            if let Ok(_pro) = a.try_recv() {
                godot_print!("Processed Packet!");
            }
        }
    }

    #[export]
    pub fn on_new_input_processer_uvc(
        &mut self,
        _owner: TRef<VSplitContainer>,
        res: Variant,
        fps: Variant,
        fmt: Variant,
    ) {
        let res = match Resolution::from_variant(res) {
            Ok(r) => r,
            Err(_) => panic!("Improper resolution format set!"),
        };

        let fps = match fps.try_to_i64() {
            Some(fs) => fs as u32,
            None => panic!("Improper resolution format set!"),
        };

        let fmt = DeviceFormat::MJPEG;

        let mut ret_bool = false;

        let (dev_ven, dev_prod, dev_ser) = crate::CURRENT_DEVICE.with(|dev| match &*dev.borrow() {
            Some(dev) => {
                let (a, b, c) = match dev {
                    DeviceContact::UVCAM {
                        vendor_id,
                        product_id,
                        serial,
                    } => (
                        vendor_id.to_owned(),
                        product_id.to_owned(),
                        serial.to_owned(),
                    ),
                    DeviceContact::V4L2 { .. } => {
                        ret_bool = true;
                        (None, None, None)
                    }
                };
                (a, b, c)
            }
            None => {
                ret_bool = true;
                (None, None, None)
            }
        });

        if ret_bool {
            return;
        }

        let vendor = match dev_ven {
            Some(i) => Some(i as c_int),
            None => None,
        };

        let product = match dev_prod {
            Some(i) => Some(i as c_int),
            None => None,
        };

        let uvc_device = match UVCameraDevice::new(vendor, product, dev_ser.clone().to_owned()) {
            Ok(d) => d,
            Err(why) => panic!("Error getting device: {}", why.to_string()),
        };

        // start the input processer
        let input_processer = match InputProcessing::new(uvc_device.get_inner()) {
            Ok(input) => input,
            Err(_) => panic!("Cannot start input processer!"),
        };
        let channel = input_processer.get_thread_output();
        *self.input_processer.borrow_mut() = Some(input_processer);
        *self.process_channel.borrow_mut() = Some(channel);
    }

    #[export]
    pub fn on_new_input_processer_v4l(
        &self,
        _owner: TRef<VSplitContainer>,
        res: Variant,
        fps: Variant,
        fmt: Variant,
    ) {
        let mut ret_bool = false;
        let dev_locat = crate::CURRENT_DEVICE.with(|dev| match &*dev.borrow() {
            Some(dev) => {
                let mut temp = &PathIndex::Index(0);
                match dev {
                    DeviceContact::UVCAM { .. } => {
                        ret_bool = true;
                    }
                    DeviceContact::V4L2 { location } => {
                        temp = location;
                    }
                }
                temp.to_owned()
            }
            None => {
                ret_bool = true;
                PathIndex::Index(0)
            }
        });

        if ret_bool {
            return;
        }

        let v4l_device = match V4LinuxDevice::new_location(&dev_locat) {
            Ok(d) => d,
            Err(why) => panic!("Error getting device!"),
        };

        match Resolution::from_variant(res) {
            Ok(r) => {
                if let Err(why) = v4l_device.set_resolution(&r) {
                    panic!("Improper resolution format set: {}!", why.to_string())
                }
            }
            Err(_) => panic!("Improper resolution format set!"),
        };

        match fps.try_to_i64() {
            Some(fs) => {
                if fs > 0 {
                    if let Err(why) = v4l_device.set_framerate(&(fs as u32)) {
                        panic!("Improper framerate set: {}!", why.to_string())
                    }
                } else {
                    panic!("Improper Framerate set!")
                }
            }
            None => panic!("Improper resolution format set!"),
        };

        match DeviceFormat::from_variant(fmt) {
            Ok(ft) => {
                v4l_device.set_camera_format(ft);
            }
            Err(_) => panic!("Improper format set!"),
        };

        godot_print!("a");

        // start the input processer
        let input_processer = match InputProcessing::new(v4l_device.get_inner()) {
            Ok(input) => input,
            Err(_) => panic!("Cannot start input processer!"),
        };

        let channel = input_processer.get_thread_output();
        *self.input_processer.borrow_mut() = Some(input_processer);
        *self.process_channel.borrow_mut() = Some(channel);
    }

    fn kill_input_processer(&mut self) {
        if let Some(processer) = self.input_processer.get_mut() {
            processer.kill();
        }
    }
}

impl Drop for ViewportHolder {
    fn drop(&mut self) {
        self.kill_input_processer();
    }
}
