use gdnative::{
    prelude::*,
    NativeClass, methods,
    api::{Panel, VBoxContainer, Control, OS}
};
use std::convert::TryInto;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Main;

#[methods]
impl Main {
    fn new(owner: &Control) -> Self {
        Main
    }

    #[export]
    fn _ready(&self, owner: TRef<Control>) {
        let root_viewport : &Viewport  = unsafe {&*owner.get_parent().unwrap().assume_safe().cast::<Viewport>().unwrap()};
        root_viewport.connect("size_changed", owner, "on_size_change", VariantArray::new_shared(), 0);
    }
    #[export]
    pub fn on_size_change(&self, owner: &Control){
        let root_viewport_size  = OS::new()
        //let colorrect = unsafe {&*owner.get_node("ColorRect").unwrap().assume_safe().cast::<ColorRect>().unwrap()};
        //let main_ui = unsafe {&*owner.get_node("Open2GHMainUINode").unwrap().assume_safe().cast::<Control>().unwrap()};
        //let panel = unsafe {&*owner.get_node("Open2GHMainUINode/Panel").unwrap().assume_safe().cast::<Panel>().unwrap()};
        let vbox = unsafe {&*owner.get_node("Open2GHMainUINode/Panel/VBoxContainer").unwrap().assume_safe().cast::<VBoxContainer>().unwrap()};

        //colorrect.set_size(root_viewport_size, true);
        //main_ui.set_size(root_viewport_size, true);
        //panel.set_size(root_viewport_size, true);
        vbox.set_size(root_viewport_size, true);
        //owner.set_size(root_viewport_size, true);
    }

}

