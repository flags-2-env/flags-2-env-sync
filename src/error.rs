#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("sync is disabled")]
    Disabled,
    #[error("payload exceeded bound")]
    TooLarge,
}

