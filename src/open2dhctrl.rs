use gdnative::{
    prelude::*,
    NativeClass, methods,
    api::{Panel, VBoxContainer, Control}
};
use std::convert::TryInto;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Main{
    colorrect_path : Option<ColorRect>,
    open2dh_mainui : Option<Control>,
    panel_node : Option<Panel>,
    vbox_main : Option<VBoxContainer>,
}

#[methods]
impl Main {
    fn new(owner: &Control) -> Self {
        Main{
            colorrect_path: None, //unsafe {*owner.get_node(NodePath::from_str("ColorRect")).unwrap().assume_safe().cast::<ColorRect>().unwrap() },
            open2dh_mainui : None, //  unsafe {*owner.get_node(NodePath::from_str("Open2GHMainUINode")).unwrap().assume_safe().cast::<Control>().unwrap()},
            panel_node:  None, //unsafe {*owner.get_node(NodePath::from_str("Open2GHMainUINode/Panel")).unwrap().assume_safe().cast::<Panel>().unwrap()},
            vbox_main: None, // unsafe {*owner.get_node(NodePath::from_str("Open2GHMainUINode/Panel/VBoxContainer")).unwrap().assume_safe().cast::<VBoxContainer>().unwrap()},
        }
    }

    #[export]
    fn _ready(&self, owner: TRef<Control>) {
        let root_viewport : &Viewport  = unsafe {&*owner.get_viewport().unwrap().assume_safe()};
        root_viewport.connect("on_resize", owner, "on_screen_resize", VariantArray::new_shared(), 0);
        let vector : Vector2 = root_viewport.size();
        godot_print!("{}",vector.x);
        godot_print!("{}",vector.y);
    }
    #[export]
    pub fn on_screen_resize(&self, _owner: &Control){
        godot_print!("resize");
    }
}

