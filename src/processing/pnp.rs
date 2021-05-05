use arrsac::Arrsac;
use cv_core::{
    nalgebra::{IsometryMatrix3, Point2, Point3},
    FeatureWorldMatch, Projective, WorldPoint,
};
use cv_pinhole::NormalizedKeyPoint;
use facial_processing::utils::{face::FaceLandmark, misc::EulerAngles};
use image::{ImageBuffer, Rgb};
use lambda_twist::LambdaTwist;
use rand::{rngs::SmallRng, SeedableRng};
use sample_consensus::Consensus;
use std::cell::RefCell;

pub struct FacePnP {
    arrsac: RefCell<Arrsac<SmallRng>>,
    lambda: LambdaTwist,
    face_points: [Point3<f64>; 6],
}

impl FacePnP {
    pub fn new() -> Self {
        let arrsac = RefCell::new(Arrsac::new(0.01, SmallRng::from_seed([0; 32])));
        let lambda = LambdaTwist::new();
        let face_points = [
            Point3::new(0.0, 0.0, 0.0),          // Nose Tip
            Point3::new(0.0, -330.0, -65.0),     // Chin
            Point3::new(-225.0, 170.0, -135.0),  // Left corner left eye
            Point3::new(225.0, 170.0, -135.0),   // Right corner right eye
            Point3::new(-150.0, -150.0, -125.0), // Mouth Corner left
            Point3::new(150.0, -150.0, -125.0),  // Mouth Corner right Mouth Corner right
        ];

        FacePnP {
            arrsac,
            lambda,
            face_points,
        }
    }

    pub fn calculate(
        &self,
        image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
        face_landmarks: FaceLandmark,
    ) -> Option<EulerAngles> {
        let img_x = f64::from(image.width());
        let img_y = f64::from(image.height());

        let facial_landmarks_6pt: Vec<NormalizedKeyPoint> = face_landmarks
            .pnp_landmarks()
            .iter()
            .map(|pt| NormalizedKeyPoint(Point2::new(pt.x() / img_x, pt.y() / img_y)))
            .collect();

        let face_points_with_nrm_img_points = facial_landmarks_6pt
            .iter()
            .zip(self.face_points.iter())
            .map(|(nrm_img_pt, world_face_pt)| {
                FeatureWorldMatch(*nrm_img_pt, WorldPoint::from_point(*world_face_pt))
            });

        match self
            .arrsac
            .borrow_mut()
            .model(&self.lambda, face_points_with_nrm_img_points.clone())
        {
            Some(angles) => {
                let (euler_x, euler_y, euler_z) = {
                    let cam_to_world: IsometryMatrix3<f64> = angles.into();
                    cam_to_world.rotation.euler_angles()
                };
                Some(EulerAngles {
                    x: euler_x,
                    y: euler_y,
                    z: euler_z,
                })
            }
            None => None,
        }
    }
}

impl Default for FacePnP {
    fn default() -> Self {
        Self::new()
    }
}
