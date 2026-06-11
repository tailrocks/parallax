//! Pre-aggregation on ingest: fingerprint counters per time bucket.
//!
//! The cost architecture (autonomous-fix-loop.md §Cost) says the Detector and
//! the UI read aggregates while raw data exists for bundles. This is that
//! aggregate: error events roll up to (fingerprint, bucket) counters at write
//! time, and the frequency-spike trigger evaluates the counter series — never
//! the raw firehose. The size ratio between raw events and their rollup is the
//! executable form of the impossible-triangle cost argument, asserted in the
//! test suite.

use crate::derive::ErrorEvent;
use crate::spike::{frequency_spike, SpikeVerdict};
use serde::Serialize;
use std::collections::BTreeMap;

pub const DEFAULT_BUCKET_NANOS: u128 = 60 * 1_000_000_000; // 1-minute buckets

#[derive(Debug, Serialize)]
pub struct RollupStore {
    pub bucket_nanos: u128,
    /// fingerprint -> bucket start (nanos) -> event count.
    pub counts: BTreeMap<String, BTreeMap<u128, u64>>,
}

impl RollupStore {
    pub fn from_events(events: &[ErrorEvent], bucket_nanos: u128) -> Self {
        let mut counts: BTreeMap<String, BTreeMap<u128, u64>> = BTreeMap::new();
        for event in events {
            let t: u128 = event.time_unix_nano.parse().unwrap_or(0);
            let bucket = (t / bucket_nanos) * bucket_nanos;
            *counts
                .entry(event.fingerprint.clone())
                .or_default()
                .entry(bucket)
                .or_insert(0) += 1;
        }
        Self { bucket_nanos, counts }
    }

    /// Dense per-bucket series from the first to the last observed bucket,
    /// zero-filled in between — the shape the spike kernel consumes.
    pub fn dense_series(&self, fingerprint: &str) -> Vec<u64> {
        let Some(buckets) = self.counts.get(fingerprint) else {
            return Vec::new();
        };
        let (Some(&first), Some(&last)) = (buckets.keys().next(), buckets.keys().last()) else {
            return Vec::new();
        };
        let mut series = Vec::new();
        let mut bucket = first;
        while bucket <= last {
            series.push(buckets.get(&bucket).copied().unwrap_or(0));
            bucket += self.bucket_nanos;
        }
        series
    }

    pub fn total(&self, fingerprint: &str) -> u64 {
        self.counts.get(fingerprint).map(|b| b.values().sum()).unwrap_or(0)
    }
}

/// The full Detect chain: raw events → rollup → dense series → spike verdict.
pub fn spike_check(
    store: &RollupStore,
    fingerprint: &str,
    k: f64,
    alpha: f64,
    min_baseline_buckets: usize,
    min_count: u64,
) -> SpikeVerdict {
    frequency_spike(&store.dense_series(fingerprint), k, alpha, min_baseline_buckets, min_count)
}
