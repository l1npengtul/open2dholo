use arrsac::Arrsac;
use facial_processing::utils::{face::FaceLandmark, misc::EulerAngles};
use image::{ImageBuffer, Rgb};
use rand::{rngs::SmallRng, SeedableRng};
use std::cell::RefCell;
use lambda_twist::LambdaTwist;
use cv_core::{CameraPoint, nalgebra::Point3};
use cv_core::{WorldPoint, WorldToCamera, Pose, Projective};

pub struct FacePnP {
    arrsac: RefCell<Arrsac<SmallRng>>,
    lambda: LambdaTwist,
    face_points: Vec<CameraPoint>,
}

impl FacePnP {
    pub fn new() -> Self {
        let mut arrsac = RefCell::new(Arrsac::new(0.01, SmallRng::seed_from_u64(0)));
        let mut lambda = LambdaTwist::new();
        let face_points: Vec<CameraPoint> = [
            Point3::new(0.0, 0.0, 0.0),          // Nose Tip
            Point3::new(0.0, -330.0, -65.0),     // Chin
            Point3::new(-225.0, 170.0, -135.0),  // Left corner left eye
            Point3::new(225.0, 170.0, -135.0),   // Right corner right eye
            Point3::new(-150.0, -150.0, -125.0), // Mouth Corner left
            Point3::new(150.0, -150.0, -125.0)   // Mouth Corner right Mouth Corner right
        ].iter().map(|p| {
            let w2c = WorldToCamera::identity();
            w2c.transform(WorldPoint::from_point(*p))
        }).collect();

        FacePnP {
            arrsac,
            lambda,
            face_points,
        }
    }

    pub fn calculate(&self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>, face_landmarks: FaceLandmark) -> EulerAngles {
        let img_x = image.width();
        let img_y = image.height();

        // TODO: add pnp arrsac calc
    }
}