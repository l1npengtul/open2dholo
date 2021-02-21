use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Error converting from {from} to {to}")]
    ConversionFromError { from: String, to: String },
    #[error("Could not convert: Conversion succeeded but match failed: {0}")]
    MatchFailedError(String),
}
