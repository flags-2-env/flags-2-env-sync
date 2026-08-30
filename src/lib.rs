#![forbid(unsafe_code)]

pub mod boundary;
pub mod error;
pub mod sqlite;

pub use boundary::{
    decide_envelope, EnvelopeDecision, PayloadLimit, SyncBoundary, SyncMode, DEFAULT_PAYLOAD_LIMIT,
};
pub use error::SyncError;
