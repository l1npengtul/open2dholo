use crate::wtf;
use euclid::{UnknownUnit, Vector2D};
use gdnative::{
    api::{MeshInstance, Resource, Skeleton, Viewport, VisualServer},
    prelude::*,
    NativeClass,
};
use std::{borrow::BorrowMut, cell::{Cell, RefCell}};
// TODO: gen gdns file and add to inithandle

#[derive(NativeClass)]
#[inherit(Viewport)]
pub struct PreviewViewport {
    loaded_model: RefCell<Option<Ref<Resource>>>,
    name: RefCell<String>,
    neck_bone_id: Cell<i32>,
}

#[methods]
impl PreviewViewport {
    fn new(_owner: &Viewport) -> Self {
        PreviewViewport {
            loaded_model: RefCell::new(None),
            neck_bone_id: Cell::new(-1),
            name: RefCell::new(String::new()),
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
                let name = node.name().to_string();
                owner.add_child(node, true);
                for child_id in 0..owner.get_child_count() {
                    let node_name =
                        unsafe { owner.get_child(child_id).unwrap().assume_safe() }.name();
                    godot_print!("{}", node_name);
                }
                *self.name.borrow_mut() = name.clone();
                // FIXME: replace with acutal node!
                let model_skeleton = unsafe {
                    owner
                        .get_node(format!("{}/Skeleton", name))
                        .unwrap()
                        .assume_safe()
                        .cast::<Skeleton>()
                        .unwrap()
                };

                for bone_idx in 0..model_skeleton.get_bone_count() {
                    if model_skeleton
                        .get_bone_name(bone_idx)
                        .to_string()
                        .contains("neck")
                    {
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
        _facebox: Variant,
        landmarks: Variant,
        angle: Variant,
    ) {
        godot_print!("process");
        if self.loaded_model.borrow().is_some() {
            let node_name = self.name.borrow().clone();
            // FIXME: replace with acutal node!
            let model = unsafe {
                owner
                    .get_node(node_name.clone())
                    .unwrap()
                    .assume_safe()
                    .cast::<Spatial>()
                    .unwrap()
            };

            let model_skeleton = unsafe {
                owner
                    .get_node(format!("{}/Skeleton", node_name))
                    .unwrap()
                    .assume_safe()
                    .cast::<Skeleton>()
                    .unwrap()
            };

            // why
            let model_mesh_inst = unsafe {
                owner
                    .get_node(format!("{}/Skeleton/Face", node_name))
                    .unwrap()
                    .assume_safe()
                    .cast::<MeshInstance>()
                    .unwrap()
            };

            let mm = { model_mesh_inst.mesh().unwrap() };

            let model_mesh = unsafe { mm.assume_safe() };

            let model_mesh_rid = model_mesh.get_rid();

            let angle_vec3 = angle.to_vector3();
            let landmarks_vec = {
                let ld = landmarks.to_vector2_array();
                let mut p2d_vec = vec![];
                for point_idx in 0..ld.len() {
                    let vec2 = ld.get(point_idx);
                    p2d_vec.push(vec2);
                }
                p2d_vec
            };

            let current_neck_transform =
                model_skeleton.get_bone_custom_pose(self.neck_bone_id.get().into());
            let new_neck_tranform = Transform {
                basis: Basis::from_euler(angle_vec3),
                origin: current_neck_transform.origin,
            };
            model_skeleton.set_bone_custom_pose(self.neck_bone_id.get().into(), new_neck_tranform);
            let (left_eye, right_eye) = calc_ear(&landmarks_vec);
            let mouth_open = single_ear(
                *landmarks_vec.get(49).unwrap(),
                *landmarks_vec.get(51).unwrap(),
                *landmarks_vec.get(53).unwrap(),
                *landmarks_vec.get(55).unwrap(),
                *landmarks_vec.get(57).unwrap(),
                *landmarks_vec.get(59).unwrap(),
            );
            // TODO: get face transforms here
            let visual_server = unsafe { VisualServer::godot_singleton() };
            for variant in model_mesh.surface_get_blend_shape_arrays(0).iter() {
                godot_print!("{:?}", variant);
            }
        }
    }
}

// FIXME: this is ***BAD***
#[inline]
fn calc_ear(landmarks: &[Vector2D<f32, UnknownUnit>]) -> (f32, f32) {
    // eye left
    let left = single_ear(
        *landmarks.get(37).unwrap(),
        *landmarks.get(38).unwrap(),
        *landmarks.get(39).unwrap(),
        *landmarks.get(40).unwrap(),
        *landmarks.get(41).unwrap(),
        *landmarks.get(42).unwrap(),
    );
    let right = single_ear(
        *landmarks.get(43).unwrap(),
        *landmarks.get(44).unwrap(),
        *landmarks.get(45).unwrap(),
        *landmarks.get(46).unwrap(),
        *landmarks.get(47).unwrap(),
        *landmarks.get(48).unwrap(),
    );
    (left, right)
}

#[inline]
fn single_ear(
    p1: Vector2D<f32, UnknownUnit>,
    p2: Vector2D<f32, UnknownUnit>,
    p3: Vector2D<f32, UnknownUnit>,
    p4: Vector2D<f32, UnknownUnit>,
    p5: Vector2D<f32, UnknownUnit>,
    p6: Vector2D<f32, UnknownUnit>,
) -> f32 {
    (euclid_distance(p2, p6) + euclid_distance(p3, p5))
        / (2_f32 * euclid_distance(p1, p4)).clamp(0_f32, 1_f32)
}

#[inline]
fn euclid_distance(p1: Vector2D<f32, UnknownUnit>, p2: Vector2D<f32, UnknownUnit>) -> f32 {
    ((p1.x - p2.x).powf(2_f32) + (p1.y - p2.y).powf(2_f32)).sqrt()
}
