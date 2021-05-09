use crate::wtf;
use gdnative::{
    api::{Resource, Skeleton, Viewport},
    prelude::*,
    NativeClass,
};
use std::cell::{Cell, RefCell};
// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Viewport)]
pub struct PreviewViewport {
    loaded_model: RefCell<Option<Ref<Resource>>>,
    neck_bone_id: Cell<i32>,
}

#[methods]
impl PreviewViewport {
    fn new(_owner: &Viewport) -> Self {
        PreviewViewport {
            loaded_model: RefCell::new(None),
            neck_bone_id: Cell::new(-1),
        }
    }

    #[export]
    fn _ready(&self, owner: TRef<Viewport>) {
        let model_load_origin = unsafe {
            &mut owner.get_node("/root/Open2DHolo/Open2DHoloMainUINode/Panel/VBoxContainer/HSplitContainer/VSplitContainer/HSplitContainer2").unwrap().assume_safe()
        };

        wtf!(model_load_origin.connect(
            "model_load_start",
            owner,
            "on_model_load_start",
            VariantArray::new_shared(),
            0
        ));

        wtf!(model_load_origin.connect(
            "frame_processed",
            owner,
            "on_frame_processed",
            VariantArray::new_shared(),
            0,
        ));
    }

    #[export]
    fn on_model_load_start(&self, owner: TRef<Viewport>, path: Variant) {
        godot_print!("?");
        let path_string = path.to_string();
        let loader = ResourceLoader::godot_singleton();
        match loader.load(path_string, "", false) {
            // What does `type_hint` do?
            Some(mdl) => {
                godot_print!("?");
                *self.loaded_model.borrow_mut() = Some(mdl);
                self.start_track_model(owner)
            }
            None => {
                godot_print!("failed to load model!");
            }
        }
    }

    #[export]
    fn start_track_model(&self, owner: TRef<Viewport>) {
        match &mut *self.loaded_model.borrow_mut() {
            Some(mdl) => {
                let node = unsafe {
                    mdl.assume_safe()
                        .cast::<PackedScene>()
                        .unwrap()
                        .instance(0)
                        .unwrap()
                        .assume_safe()
                };
                owner.add_child(node, true);
                for child_id in 0..owner.get_child_count() {
                    let node_name =
                        unsafe { owner.get_child(child_id).unwrap().assume_safe() }.name();
                    godot_print!("{}", node_name);
                }
                // FIXME: replace with acutal node!
                let model_skeleton = unsafe {
                    owner
                        .get_node("Spatial/Skeleton")
                        .unwrap()
                        .assume_safe()
                        .cast::<Skeleton>()
                        .unwrap()
                };

                for bone_idx in 0..model_skeleton.get_bone_count()  {
                    if model_skeleton.get_bone_name(bone_idx).to_string().contains("neck") {
                        self.neck_bone_id.set(bone_idx as i32);
                    }
                }
            }
            None => {}
        }
    }

    #[export]
    fn on_frame_processed(
        &self,
        owner: TRef<Viewport>,
        facebox: Variant,
        landmarks: Variant,
        angle: Variant,
    ) {
        if self.loaded_model.borrow().is_some() {
            // FIXME: replace with acutal node!
            let model = unsafe {
                owner
                    .get_node("Spatial")
                    .unwrap()
                    .assume_safe()
                    .cast::<Spatial>()
                    .unwrap()
            };

            let model_skeleton = unsafe {
                owner
                    .get_node("Spatial/Skeleton")
                    .unwrap()
                    .assume_safe()
                    .cast::<Skeleton>()
                    .unwrap()
            };

            let angle_vec3 = angle.to_vector3();
            let current_neck_transform = model_skeleton.get_bone_custom_pose(self.neck_bone_id.get().into());
            let new_neck_tranform = Transform {
                basis: Basis::from_euler(angle_vec3),
                origin: current_neck_transform.origin,
            };
            model_skeleton.set_bone_custom_pose(self.neck_bone_id.get().into(), new_neck_tranform);
            
            // TODO: get face transforms here
        }
    }
}
