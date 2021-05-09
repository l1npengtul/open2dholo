use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Invalid number of bones: expected > 0 but got {0}: {1}")]
    InvalidBoneNumberError(i64, String),
}
