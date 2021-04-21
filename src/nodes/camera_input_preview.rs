use std::cell::RefCell;
use crate::wtf;
use gdnative::{api::Control, prelude::*, NativeClass};

// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Control)]
pub struct CameraInputPreview {
    processed_68pt_normalized: RefCell<TypedArray<Vector2>>,
}

#[methods]
impl CameraInputPreview {
    fn new(_owner: &Control) -> Self {
        CameraInputPreview {
            processed_68pt_normalized: RefCell::new(TypedArray::new()),
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

    // Draw points her~e
    #[export]
    fn _draw(&self, owner: &Control) {
        //
    }

    #[export]
    pub fn on_new_processed_frame_68pt(&self, _owner: TRef<Control>, pointarray: Variant) {
        let vec2_arr = pointarray.to_vector2_array();
        if vec2_arr.len() == 68 {
            *self.processed_68pt_normalized.borrow_mut() = vec2_arr;
        }
    }
}
