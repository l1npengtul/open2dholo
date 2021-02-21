use dlib_face_recognition::{Point, Rectangle};
use std::io::{IoSliceMut, Read};
use std::path::Path;

#[derive(Copy, Clone)]
pub enum DetectorHardware {
    Cpu,
    GpuCuda,
    // GpuROCm // soon, nvidia is big boomer proprietary cuda shit so we need to ship separate libtorch
}

#[derive(Copy, Clone)]
pub enum DetectorDimensionality {
    Pettan2D,
    // rushia, suisei, shion, gura, ina, matsuri, kanata
    // i beg for my life
    Illusion2HalfD,
    // 2.5D - like this: https://www.reddit.com/r/Hololive/comments/jwatsi/rushia_in_a_boing_boing_optical_illusion_shirt_by/
    BoingBoing3D, // 3D
}

#[derive(Clone)]
pub enum PointType {
    NoPt,
    Pt2D(Vec<Point2D>),
    Pt3D(Vec<Point3D>),
}

#[derive(Copy, Clone)]
pub enum DetectorType {
    DLibFHOG,
    DLibCNN,
    // Torch,
}

pub trait DetectorTrait: Send {
    // la la la filler lala
    fn detect_face_rects(&self, img_height: u32, img_width: u32, img_data: &[u8]) -> Vec<Rect>;
    fn detect_landmarks(
        &self,
        rect: &Rect,
        img_height: u32,
        img_width: u32,
        img_data: &[u8],
    ) -> PointType;
}

#[derive(Copy, Clone)]
pub struct Rect {
    left_bottom_x1: i32,
    left_bottom_y1: i32,
    right_top_x2: i32,
    right_top_y2: i32,
}

impl Rect {
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Rect {
            left_bottom_x1: x1,
            left_bottom_y1: y1,
            right_top_x2: x2,
            right_top_y2: y2,
        }
    }

    pub fn from_points(p1: &Point2D, p2: &Point2D) -> Self {
        Rect {
            left_bottom_x1: p1.x() as i32,
            left_bottom_y1: p1.y() as i32,
            right_top_x2: p2.x() as i32,
            right_top_y2: p2.y() as i32,
        }
    }

    pub fn from_rectangle(r: &Rectangle) -> Rect {
        Rect {
            left_bottom_x1: r.left as i32,
            left_bottom_y1: r.bottom as i32,
            right_top_x2: r.right as i32,
            right_top_y2: r.top as i32,
        }
    }

    pub fn x1(&self) -> i32 {
        self.left_bottom_x1
    }

    pub fn x2(&self) -> i32 {
        self.right_top_x2
    }

    pub fn y1(&self) -> i32 {
        self.left_bottom_y1
    }

    pub fn y2(&self) -> i32 {
        self.right_top_y2
    }

    pub fn all(&self) -> (i32, i32, i32, i32) {
        (
            self.left_bottom_x1,
            self.left_bottom_y1,
            self.right_top_x2,
            self.right_top_y2,
        )
    }

    pub fn as_point(&self, get: u8) -> Point2D {
        return match get {
            0 => Point2D::new(self.left_bottom_x1 as u32, self.left_bottom_y1 as u32),
            1 => Point2D::new(self.right_top_x2 as u32, self.right_top_y2 as u32),
            _ => Point2D::new(self.left_bottom_x1 as u32, self.left_bottom_y1 as u32),
        };
    }

    pub fn as_points(&self) -> (Point2D, Point2D) {
        (
            Point2D::new(self.left_bottom_x1 as u32, self.left_bottom_y1 as u32),
            Point2D::new(self.right_top_x2 as u32, self.right_top_y2 as u32),
        )
    }

    pub fn as_rectangle(&self) -> Rectangle {
        Rectangle {
            left: self.left_bottom_x1 as i64,
            top: self.right_top_y2 as i64,
            right: self.right_top_x2 as i64,
            bottom: self.left_bottom_y1 as i64,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Point3D {
    x: u32,
    y: u32,
    z: u32,
}

impl Point3D {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Point3D { x, y, z }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn z(&self) -> u32 {
        self.z
    }
}

#[derive(Copy, Clone)]
pub struct Point2D {
    x: u32,
    y: u32,
}

impl Point2D {
    pub fn new(x: u32, y: u32) -> Self {
        Point2D { x, y }
    }

    pub fn from_point(p: &Point) -> Self {
        Point2D {
            x: p.x() as u32,
            y: p.y() as u32,
        }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }
}

// // struct to hold model data
// pub struct ModelHolder {
//     data: Vec<u8>,
// }
//
// impl ModelHolder {
//     pub fn new(path: &str) -> Self {
//         let mut data = Vec::new();
//         let data_arr = include_bytes!(path);
//         for byte in data_arr {
//             data.push(*byte);
//         }
//         ModelHolder { data }
//     }
// }
//
// impl Read for ModelHolder {
//     fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
//         unimplemented!()
//     }
//
//     fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
//         for byte in self.data {
//             buf.push(byte);
//         }
//         Ok(buf.len())
//     }
// }
