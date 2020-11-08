use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadSendMessageError {
    #[error("Error sending message to thread.")]
    ErrorSending,
}
