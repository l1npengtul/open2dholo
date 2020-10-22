use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Main;

#[methods]
impl Main {
    
}

fn init(handle: InitHandle){
    handle.add_class::<Main>();
}