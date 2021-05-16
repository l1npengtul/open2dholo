use cv::{
    camera::pinhole::NormalizedKeyPoint,
    estimate::LambdaTwist,
    nalgebra::{Isometry, Point2, Point3, Rotation, U3},
    Estimator, FeatureWorldMatch, Projective, WorldPoint,
};
use facial_processing::utils::{face::FaceLandmark, misc::EulerAngles};
use image::{ImageBuffer, Rgb};
pub struct FacePnP {
    lambda: LambdaTwist,
    face_points: [Point3<f64>; 6],
}

impl FacePnP {
    pub fn new() -> Self {
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
            lambda,
            face_points,
        }
    }

    // FIXME: precalculate facial points
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

        // TODO: check out second rotation
        for pose in Estimator::estimate(&self.lambda, face_points_with_nrm_img_points) {
            let isometry: &Isometry<f64, U3, Rotation<f64, U3>> = pose.as_ref();
            let (x, y, z) = isometry.rotation.euler_angles();
            return Some(EulerAngles { x, y, z });
        }
        None
    }
}

impl Default for FacePnP {
    fn default() -> Self {
        Self::new()
    }
}
