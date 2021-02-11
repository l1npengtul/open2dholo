use crate::processing::face_detector::detectors::util::{
    DetectorHardware, DetectorTrait, Point2D, PointType, Rect,
};
use dlib_face_recognition::{
    FaceDetector, FaceDetectorCnn, FaceDetectorTrait, ImageMatrix, LandmarkPredictor,
    LandmarkPredictorTrait, Rectangle,
};
use gdnative::prelude::godot_print;

pub struct DLibDetector {
    detector: Box<dyn FaceDetectorTrait>,
    landmark: Box<dyn LandmarkPredictorTrait>,
}

impl DLibDetector {
    pub fn new(is_cnn: bool) -> Self {
        if is_cnn {
            DLibDetector {
                detector: Box::new(FaceDetector::default()),
                landmark: Box::new(LandmarkPredictor::default()),
            }
        } else {
            DLibDetector {
                detector: Box::new(FaceDetectorCnn::default()),
                landmark: Box::new(LandmarkPredictor::default()),
            }
        }
    }
}

impl DetectorTrait for DLibDetector {
    fn detect_face_rects(&self, img_height: u32, img_width: u32, img_data: &[u8]) -> Vec<Rect> {
        let img_matrix =
            unsafe { ImageMatrix::new(img_height as usize, img_width as usize, img_data.as_ptr()) };
        let mut rect_vec = Vec::new();
        for face in self.detector.face_locations(&img_matrix).iter() {
            rect_vec.push(Rect::from_rectangle(face));
        }
        rect_vec
    }

    fn detect_landmarks(
        &self,
        rect: &Rect,
        img_height: u32,
        img_width: u32,
        img_data: &[u8],
    ) -> PointType {
        let img_matrix =
            unsafe { ImageMatrix::new(img_height as usize, img_width as usize, img_data.as_ptr()) };
        let mut face_landmarks = Vec::new();
        for pt in self
            .landmark
            .face_landmarks(&img_matrix, &rect.as_rectangle())
            .iter()
        {
            face_landmarks.push(Point2D::from_point(pt));
        }
        if face_landmarks.len() == 68 {
            PointType::Pt2D(face_landmarks)
        } else {
            godot_print!("points: {}", face_landmarks.len());
            PointType::NoPt
        }
    }
}
