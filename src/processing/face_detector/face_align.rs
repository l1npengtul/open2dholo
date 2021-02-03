use crate::error::processing_error::ProcessingError;
use pyo3::types::PyModule;
use pyo3::{GILGuard, PyErr, Python};
use

pub enum DetectorBackend {
    DLib,
    SFD,
    Folder,
}

pub enum DetectorDimension {
    TwoD,
    TwoPointFiveD,
    ThreeD,
}

pub enum DetectorHW {
    CPU,
    GPU,
    // GPU_ROCm, // TODO
}

pub struct FaceAligner<'a> {
    gil: GILGuard,
    python: Python<'a>,
    face_module: &'a PyModule,
    face_obj:
    backend: DetectorBackend,
    dimension: DetectorDimension,
    hardware: DetectorHW,
}

impl<'a> FaceAligner<'a> {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let gil = Python::acquire_gil();
        let python = gil.python();
        let face_module = match PyModule::from_code(
            python,
            include_str!("python/facealign.py"),
            "facealign.py",
            "facealign",
        ) {
            Ok(m) => m,
            Err(why) => {
                return Err(Box::new(ProcessingError::General(why.to_string())));
            }
        };
    }
}
