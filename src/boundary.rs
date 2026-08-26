#![forbid(unsafe_code)]

use crate::error::SyncError;
use crate::sqlite::SqliteStore;

#[derive(Clone, Debug)]
pub struct SyncBoundary {
    pub store: SqliteStore,
    pub enabled: bool,
}

impl SyncBoundary {
    pub fn envelope(&self, bytes: &[u8]) -> Result<usize, SyncError> {
        if !self.enabled {
            return Err(SyncError::Disabled);
        }
        if bytes.len() > 64 * 1024 {
            return Err(SyncError::TooLarge);
        }
        Ok(bytes.len())
    }
}

