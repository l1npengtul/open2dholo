use gdnative::{
    prelude::*,
    NativeClass, methods
};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Main;

#[methods]
impl Main {
    fn new(_owner: &Node) -> Self {
        Main
    }

    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("hello, world.");
    }
}

