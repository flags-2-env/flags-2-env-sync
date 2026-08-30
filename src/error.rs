#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SyncError {
    #[error("sync is disabled")]
    Disabled,
    #[error("payload exceeded bound")]
    TooLarge,
    #[error("payload bound must be positive")]
    InvalidPayloadLimit,
}
