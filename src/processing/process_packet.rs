use gdnative::core_types::Vector2;
use parking_lot::RwLock;
use std::sync::Arc;

// TODO: Change to acutal data format

#[derive(Clone)]
pub struct Processed {
    data: Vec<Vector2>,
    frame_data: Option<Arc<RwLock<uvc::Frame>>>
}
impl Processed {
    fn new(data: Vec<Vector2>, imgframe: Option<Arc<RwLock<uvc::Frame>>>) -> Self {
        Processed { data,
        frame_data: imgframe}
    }
}
