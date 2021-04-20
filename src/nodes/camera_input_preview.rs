use gdnative::{api::Control, prelude::*, NativeClass};

// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Control)]
pub struct CameraInputPreview;

#[methods]
impl CameraInputPreview {
    fn new(_owner: &Control) -> Self {
        CameraInputPreview
    }

    #[export]
    fn _ready(&self, _owner: &Control) {

    }

    // Draw points here
    #[export]
    fn _draw(&self, _owner: &Control) {

    }

    #[export]
    pub fn on_new_frame_draw(&self, _owner: &Control, _pointarray: Variant) {

    }
}