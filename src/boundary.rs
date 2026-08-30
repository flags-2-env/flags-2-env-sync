#![forbid(unsafe_code)]

use crate::error::SyncError;
use crate::sqlite::SqliteStore;
use std::num::NonZeroUsize;

pub const DEFAULT_PAYLOAD_LIMIT: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyncMode {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadLimit(NonZeroUsize);

impl PayloadLimit {
    pub fn new(bytes: usize) -> Result<Self, SyncError> {
        NonZeroUsize::new(bytes)
            .map(Self)
            .ok_or(SyncError::InvalidPayloadLimit)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvelopeDecision {
    Accept { bytes: usize },
    Reject(SyncError),
}

/// Pure decision core: no storage, network, clock, or process state is read.
#[must_use]
pub const fn decide_envelope(
    mode: SyncMode,
    payload_bytes: usize,
    limit: PayloadLimit,
) -> EnvelopeDecision {
    match mode {
        SyncMode::Disabled => EnvelopeDecision::Reject(SyncError::Disabled),
        SyncMode::Enabled if payload_bytes > limit.get() => {
            EnvelopeDecision::Reject(SyncError::TooLarge)
        }
        SyncMode::Enabled => EnvelopeDecision::Accept {
            bytes: payload_bytes,
        },
    }
}

#[derive(Clone, Debug)]
pub struct SyncBoundary {
    store: SqliteStore,
    mode: SyncMode,
    payload_limit: PayloadLimit,
}

impl SyncBoundary {
    pub fn new(
        store: SqliteStore,
        mode: SyncMode,
        payload_limit: usize,
    ) -> Result<Self, SyncError> {
        Ok(Self {
            store,
            mode,
            payload_limit: PayloadLimit::new(payload_limit)?,
        })
    }

    pub fn with_default_limit(store: SqliteStore, mode: SyncMode) -> Self {
        Self {
            store,
            mode,
            payload_limit: PayloadLimit(
                NonZeroUsize::new(DEFAULT_PAYLOAD_LIMIT)
                    .expect("the compile-time default payload limit is positive"),
            ),
        }
    }

    #[must_use]
    pub const fn store(&self) -> &SqliteStore {
        &self.store
    }

    pub fn envelope(&self, bytes: &[u8]) -> Result<usize, SyncError> {
        match decide_envelope(self.mode, bytes.len(), self.payload_limit) {
            EnvelopeDecision::Accept { bytes } => Ok(bytes),
            EnvelopeDecision::Reject(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(bytes: usize) -> PayloadLimit {
        PayloadLimit::new(bytes).expect("positive test limit")
    }

    #[test]
    fn decision_table_is_exhaustive_and_fail_closed() {
        let cases = [
            (
                SyncMode::Disabled,
                0,
                EnvelopeDecision::Reject(SyncError::Disabled),
            ),
            (
                SyncMode::Disabled,
                9,
                EnvelopeDecision::Reject(SyncError::Disabled),
            ),
            (SyncMode::Enabled, 0, EnvelopeDecision::Accept { bytes: 0 }),
            (SyncMode::Enabled, 8, EnvelopeDecision::Accept { bytes: 8 }),
            (
                SyncMode::Enabled,
                9,
                EnvelopeDecision::Reject(SyncError::TooLarge),
            ),
        ];

        for (mode, payload_bytes, expected) in cases {
            assert_eq!(decide_envelope(mode, payload_bytes, limit(8)), expected);
        }
    }

    #[test]
    fn zero_cannot_form_a_payload_limit() {
        assert_eq!(PayloadLimit::new(0), Err(SyncError::InvalidPayloadLimit));
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    fn arbitrary_positive_limit() -> PayloadLimit {
        let raw: usize = kani::any();
        kani::assume(raw > 0);
        let Some(non_zero) = NonZeroUsize::new(raw) else {
            unreachable!("Kani assumption excludes zero");
        };
        PayloadLimit(non_zero)
    }

    #[kani::proof]
    fn disabled_sync_never_accepts_payloads() {
        let decision = decide_envelope(SyncMode::Disabled, kani::any(), arbitrary_positive_limit());
        assert!(matches!(
            decision,
            EnvelopeDecision::Reject(SyncError::Disabled)
        ));
    }

    #[kani::proof]
    fn accepted_payloads_are_within_the_declared_bound() {
        let bytes: usize = kani::any();
        let limit = arbitrary_positive_limit();
        if let EnvelopeDecision::Accept { bytes: accepted } =
            decide_envelope(SyncMode::Enabled, bytes, limit)
        {
            assert_eq!(accepted, bytes);
            assert!(accepted <= limit.get());
        }
    }

    #[kani::proof]
    fn oversized_payloads_are_rejected() {
        let bytes: usize = kani::any();
        let limit = arbitrary_positive_limit();
        kani::assume(bytes > limit.get());
        assert!(matches!(
            decide_envelope(SyncMode::Enabled, bytes, limit),
            EnvelopeDecision::Reject(SyncError::TooLarge)
        ));
    }
}
