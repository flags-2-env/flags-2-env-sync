#![forbid(unsafe_code)]

use crate::boundary::{decide_envelope, EnvelopeDecision, PayloadLimit, SyncMode};
use crate::error::SyncError;

/// Pure admission for a new chunk appended to a previously counted payload.
/// Arithmetic overflow rejects before any store or transport side effect.
#[must_use]
pub const fn decide_incremental(
    mode: SyncMode,
    received_bytes: usize,
    incoming_bytes: usize,
    limit: PayloadLimit,
) -> EnvelopeDecision {
    match mode {
        SyncMode::Disabled => EnvelopeDecision::Reject(SyncError::Disabled),
        SyncMode::Enabled => match received_bytes.checked_add(incoming_bytes) {
            Some(total) => decide_envelope(SyncMode::Enabled, total, limit),
            None => EnvelopeDecision::Reject(SyncError::TooLarge),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(bytes: usize) -> PayloadLimit {
        PayloadLimit::new(bytes).expect("positive test limit")
    }

    #[test]
    fn exhaustive_small_chunk_grid_agrees_with_total_envelope_policy() {
        for received in 0_usize..12 {
            for incoming in 0_usize..12 {
                for mode in [SyncMode::Disabled, SyncMode::Enabled] {
                    let expected = match mode {
                        SyncMode::Disabled => EnvelopeDecision::Reject(SyncError::Disabled),
                        SyncMode::Enabled => decide_envelope(mode, received + incoming, limit(8)),
                    };
                    assert_eq!(
                        decide_incremental(mode, received, incoming, limit(8)),
                        expected,
                        "received={received}, incoming={incoming}, mode={mode:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn exact_boundary_and_zero_byte_chunks_are_supported() {
        assert_eq!(
            decide_incremental(SyncMode::Enabled, 8, 0, limit(8)),
            EnvelopeDecision::Accept { bytes: 8 }
        );
        assert_eq!(
            decide_incremental(SyncMode::Enabled, 7, 1, limit(8)),
            EnvelopeDecision::Accept { bytes: 8 }
        );
        assert_eq!(
            decide_incremental(SyncMode::Enabled, 7, 2, limit(8)),
            EnvelopeDecision::Reject(SyncError::TooLarge)
        );
    }

    #[test]
    fn overflow_and_previously_oversized_inputs_fail_closed() {
        for received in [usize::MAX, usize::MAX - 1] {
            assert_eq!(
                decide_incremental(SyncMode::Enabled, received, 2, limit(8)),
                EnvelopeDecision::Reject(SyncError::TooLarge)
            );
        }
        assert_eq!(
            decide_incremental(SyncMode::Enabled, 9, 0, limit(8)),
            EnvelopeDecision::Reject(SyncError::TooLarge)
        );
        assert_eq!(
            decide_incremental(SyncMode::Disabled, usize::MAX, usize::MAX, limit(8)),
            EnvelopeDecision::Reject(SyncError::Disabled)
        );
    }
}
