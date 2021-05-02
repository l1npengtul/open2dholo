use arrsac::Arrsac;
use opencv::core::Point3d;
use rand::{rngs::SmallRng, SeedableRng};
use std::cell::RefCell;
use cv::

pub struct FacePnP {
    arrsac: RefCell<Arrsac<SmallRng>>,
    face_points: [Point3d; 6],
}

impl FacePnP {
    pub fn new() -> Self {
        let mut arrsac = Arrsac::new(0.01, SmallRng::seed_from_u64(0));
        let face_points = [
            Point3d::new(0.0, 0.0, 136.0),          // Nose Tip
            Point3d::new(0.0, -330.0, 71.0),     // Chin
            Point3d::new(-225.0, 170.0, 1.0),  // Left corner left eye
            Point3d::new(225.0, 170.0, 1.0),   // Right corner right eye
            Point3d::new(-150.0, -150.0, 11.0), // Mouth Corner left
            Point3d::new(150.0, -150.0, 11.0)   // Mouth Corner right
        ];
    }
}