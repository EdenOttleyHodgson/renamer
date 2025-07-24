use std::error::Error;
use thiserror::Error;

pub(crate) type SendableErr = Box<dyn Error + Send + Sync>;

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Cannot rename a path ending in \"..\"")]
    CannotRenameDotDot,
    #[error("{0}")]
    Other(SendableErr),
    #[error("Unknown")]
    Unknown,
}

impl From<SendableErr> for ActionError {
    fn from(v: SendableErr) -> Self {
        match v.downcast::<Self>() {
            Ok(x) => *x,
            Err(e) => ActionError::Other(e),
        }
    }
}
