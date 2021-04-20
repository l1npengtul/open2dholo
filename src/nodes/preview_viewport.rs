use gdnative::{api::Viewport, prelude::*, NativeClass};

// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Viewport)]
pub struct PreviewViewport;

#[methods]
impl PreviewViewport {
    fn new(_owner: &Viewport) -> Self {
        PreviewViewport
    }

    #[export]
    fn _ready(&self, _owner: &Viewport) {

    }
}