use gdnative::{
    api::{Control, Panel, VBoxContainer, OS},
    methods,
    prelude::*,
    NativeClass,
};

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Main;

#[methods]
impl Main {
    fn new(_owner: &Control) -> Self {
        Main
    }

    #[export]
    fn _ready(&self, owner: TRef<Control>) {
        let root_viewport: &Viewport = unsafe {
            &*owner
                .get_parent()
                .unwrap()
                .assume_safe()
                .cast::<Viewport>()
                .unwrap()
        };
        root_viewport.connect(
            "size_changed",
            owner,
            "on_size_change",
            VariantArray::new_shared(),
            0,
        );
    }
    #[export]
    pub fn on_size_change(&self, owner: &Control) {
        let root_viewport_size = OS::godot_singleton().window_size();
        let colorrect = unsafe {
            &*owner
                .get_node("ColorRect")
                .unwrap()
                .assume_safe()
                .cast::<ColorRect>()
                .unwrap()
        };
        let main_ui = unsafe {
            &*owner
                .get_node("Open2GHMainUINode")
                .unwrap()
                .assume_safe()
                .cast::<Control>()
                .unwrap()
        };
        let panel = unsafe {
            &*owner
                .get_node("Open2GHMainUINode/Panel")
                .unwrap()
                .assume_safe()
                .cast::<Panel>()
                .unwrap()
        };
        let vbox = unsafe {
            &*owner
                .get_node("Open2GHMainUINode/Panel/VBoxContainer")
                .unwrap()
                .assume_safe()
                .cast::<VBoxContainer>()
                .unwrap()
        };

        let vbox_size = Vector2::new(
            root_viewport_size.x - (vbox.position().x * 2.0),
            root_viewport_size.y - (vbox.position().y * 2.0),
        );

        owner.set_size(root_viewport_size, true);
        colorrect.set_size(root_viewport_size, true);
        main_ui.set_size(root_viewport_size, true);
        panel.set_size(root_viewport_size, true);
        vbox.set_size(vbox_size, true);
    }
}
