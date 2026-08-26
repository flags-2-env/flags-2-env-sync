#![forbid(unsafe_code)]

#[derive(Clone, Debug, Default)]
pub struct SqliteStore {
    pub path: String,
}

