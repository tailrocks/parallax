//! Frequency-spike trigger kernel (Detect stage).
//!
//! Implements the `frequency_spike` predicate from
//! docs/research/architecture/autonomous-fix-loop.md: fingerprint rate above
//! k× the trailing EWMA baseline. Pure and deterministic — bucket counts in,
//! verdict out. The min-count floor stops near-zero baselines from turning
//! noise (2 events after a quiet hour) into dispatches, and the
//! insufficient-baseline guard stops cold-start fingerprints from spiking on
//! their first buckets (those are `new_fingerprint` territory instead).

pub const DEFAULT_K: f64 = 4.0;
pub const DEFAULT_EWMA_ALPHA: f64 = 0.3;
pub const DEFAULT_MIN_BASELINE_BUCKETS: usize = 6;
pub const DEFAULT_MIN_COUNT: u64 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpikeVerdict {
    /// Latest bucket exceeds k× baseline and the absolute floor.
    Spike {
        latest: u64,
        /// EWMA baseline in fixed-point millis (deterministic equality).
        baseline_ewma_milli: u64,
    },
    NoSpike,
    /// Not enough history to claim a baseline; cold-start fingerprints are
    /// handled by the new_fingerprint trigger, not this one.
    InsufficientBaseline,
}

/// Evaluate the latest bucket against the EWMA of all prior buckets.
///
/// `counts` are per-bucket event counts for one (project, fingerprint,
/// environment), oldest first; the last element is the bucket under test.
pub fn frequency_spike(
    counts: &[u64],
    k: f64,
    alpha: f64,
    min_baseline_buckets: usize,
    min_count: u64,
) -> SpikeVerdict {
    if counts.len() < 2 || counts.len() - 1 < min_baseline_buckets {
        return SpikeVerdict::InsufficientBaseline;
    }
    let (baseline_buckets, latest_slice) = counts.split_at(counts.len() - 1);
    let latest = latest_slice[0];

    let mut ewma = baseline_buckets[0] as f64;
    for &count in &baseline_buckets[1..] {
        ewma = alpha * count as f64 + (1.0 - alpha) * ewma;
    }

    if latest >= min_count && (latest as f64) > k * ewma {
        SpikeVerdict::Spike { latest, baseline_ewma_milli: (ewma * 1000.0).round() as u64 }
    } else {
        SpikeVerdict::NoSpike
    }
}
