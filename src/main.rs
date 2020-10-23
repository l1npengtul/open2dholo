use gdnative::{
    prelude::*,
    NativeClass, nativescript, methods, api::*
};

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Main;

#[methods]
impl Main {
    
}

fn init(handle: InitHandle){
    handle.add_class::<Main>();
}
