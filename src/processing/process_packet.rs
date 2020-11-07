use gdnative::core_types::Vector2;

// TODO: Change to acutal data format

#[derive(Copy, Clone)]
pub struct Processed {
    data: Vec<Vector2>,
}
impl Processed {
    fn new(data: Vec<Vector2>) -> Self {
        Processed { data }
    }
}
