use crate::error::processing_error::ProcessingError;
use pyo3::types::PyModule;
use pyo3::{GILGuard, PyErr, Python};

pub struct FaceAligner<'a> {
    gil: GILGuard,
    python: Python<'a>,
    face_module: &'a PyModule,
}

impl<'a> FaceAligner<'a> {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let gil = Python::acquire_gil();
        let python = gil.python();
        let face_module = match python.import("face_alignment") {
            Ok(module) => module,
            Err(why) => {
                return Err(Box::new(ProcessingError::General(why.to_string())));
            }
        };
    }
}
