use crate::wtf;
use gdnative::{api::Control, prelude::*, NativeClass};
use std::cell::{Cell, RefCell};

// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Control)]
pub struct CameraInputPreview {
    processed_68pt_normalized: RefCell<Vec<Vector2>>,
    x_max: Cell<f32>,
    y_max: Cell<f32>,
}

#[methods]
impl CameraInputPreview {
    fn new(_owner: &Control) -> Self {
        CameraInputPreview {
            processed_68pt_normalized: RefCell::new(Vec::new()),
            x_max: Cell::new(0_f32),
            y_max: Cell::new(0_f32),
        }
    }

    #[export]
    fn _ready(&self, owner: TRef<Control>) {
        let input_process = unsafe {
            owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HSplitContainer/VSplitContainer/HSplitContainer2").unwrap().assume_safe()
        };

        wtf!(input_process.connect(
            "new_processed_frame_68pt",
            owner,
            "on_new_processed_frame_68pt",
            VariantArray::new_shared(),
            0
        ));
    }

    // Draw points here
    #[export]
    fn _draw(&self, owner: &Control) {
        let current_size = owner.size();
        let ratio = (self.x_max.get() / current_size.x).min(self.y_max.get() / current_size.y);
        for v2 in self.processed_68pt_normalized.borrow().iter() {
            owner.draw_circle(
                Vector2::new(v2.x / ratio, v2.y / ratio),
                1.0_f64,
                Color::rgb(255_f32, 255_f32, 255_f32),
            )
        }
    }

    #[export]
    pub fn on_new_processed_frame_68pt(&self, _owner: TRef<Control>, pointarray: Variant) {
        let vec2_arr = pointarray.to_vector2_array();
        if vec2_arr.len() == 68 {
            let mut x_mx = 0_f32;
            let mut y_mx = 0_f32;
            let mut vec = vec![];
            for i in 0..vec2_arr.len() {
                let vec2 = vec2_arr.get(i);
                if vec2.x > x_mx {
                    x_mx = vec2.x;
                }
                if vec2.y > y_mx {
                    y_mx = vec2.y;
                }
                vec.push(vec2);
            }
            *self.processed_68pt_normalized.borrow_mut() = vec;
            self.x_max.set(x_mx);
            self.y_max.set(y_mx);
        }
    }
}
