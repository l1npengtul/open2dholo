use gdnative::{
    prelude::*,
    NativeClass, methods,
    api::{Panel, VBoxContainer, Control}
};
use std::convert::TryInto;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Main{
    colorrect_path : GodotString,
    open2dh_mainui : GodotString,
    panel_node : GodotString,
    vbox_main : GodotString,
    current_size : Vector2,
}

#[methods]
impl Main {
    fn new(owner: &Control) -> Self {
        Main{
            colorrect_path: GodotString::from_str(""), //unsafe {*owner.get_node(NodePath::from_str("ColorRect")).unwrap().assume_safe().cast::<ColorRect>().unwrap() },
            open2dh_mainui : GodotString::from_str(""), //  unsafe {*owner.get_node(NodePath::from_str("Open2GHMainUINode")).unwrap().assume_safe().cast::<Control>().unwrap()},
            panel_node:  GodotString::from_str(""), //unsafe {*owner.get_node(NodePath::from_str("Open2GHMainUINode/Panel")).unwrap().assume_safe().cast::<Panel>().unwrap()},
            vbox_main: GodotString::from_str(""), // unsafe {*owner.get_node(NodePath::from_str("Open2GHMainUINode/Panel/VBoxContainer")).unwrap().assume_safe().cast::<VBoxContainer>().unwrap()},
            current_size: Vector2::new(0.0,0.0),
        }
    }

    #[export]
    fn _ready(&self, owner: TRef<Control>) {
        //self.colorrect_path = GodotString::from_str("ColorRect");
        //self.open2dh_mainui = GodotString::from_str("Open2GHMainUINode");
        //self.panel_node = GodotString::from_str("Open2GHMainUINode/Panel");
        //self.vbox_main = GodotString::from_str("Open2GHMainUINode/Panel/VBoxContainer");
        let root_viewport : &Viewport  = unsafe {&*owner.get_parent().unwrap().assume_safe().cast::<Viewport>().unwrap()};
        root_viewport.connect("size_changed", owner, "on_screen_resize", VariantArray::new_shared(), 0);
    }
    #[export]
    pub fn on_size_change(&self, owner: &Control){
        godot_print!("a");
    }

}

