//! OTel exponential-histogram → explicit-bucket conversion.
//!
//! The native metric engine has no exponential-histogram type, and the OTLP
//! forward path cannot store one — so an exp point that reaches ingest would
//! otherwise be lost twice (dropped by the normalizer, unrepresentable
//! downstream). Conversion rescues it: every exp bucket has an exact implicit
//! upper bound (`base^index`, `base = 2^(2^-scale)`), which becomes an
//! explicit bound. Converted rows flow through the existing explicit-histogram
//! quantile/avg machinery unchanged, so one query path serves both encodings.
//!
//! Mapping, ascending by upper bound:
//! - negative buckets (index `j`, values in `[-base^(j+1), -base^j)`),
//!   most-negative first;
//! - the zero bucket (`[-zero_threshold, +zero_threshold]`), when non-empty;
//! - positive buckets (values in `(base^j, base^(j+1)]`).
//!
//! The explicit form is half-open on the same edges the quantile interpolator
//! assumes, so the error is bounded by one bucket width — the same guarantee
//! an explicit histogram carries. Overflow clamps to `±f64::MAX` (finite) and
//! non-increasing bounds merge into the previous bucket, so output bounds are
//! always strictly increasing and finite. A degenerate scale (non-positive or
//! non-finite `2^-scale`) refuses conversion instead of emitting garbage.

use parallax_proto::metrics::ExponentialHistogramDataPoint;

/// One converted datapoint: explicit bounds with per-bucket counts.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvertedHistogram {
    pub count: u64,
    pub sum: f64,
    pub bounds: Vec<f64>,
    pub bucket_counts: Vec<u64>,
}

/// Convert one exponential-histogram datapoint to explicit buckets.
/// Returns `None` when the scale is degenerate (no finite bucket grid).
#[must_use]
pub fn convert(point: &ExponentialHistogramDataPoint) -> Option<ConvertedHistogram> {
    // Grid step: bound(index) = 2^(index * step) with step = 2^-scale.
    // (i64 negation: `-i32::MIN` would overflow; out-of-f64-exponent scales
    // have no representable grid and refuse conversion.)
    let scale = i64::from(point.scale);
    if !(-1074..=1074).contains(&scale) {
        return None;
    }
    // Negated scale fits i32 on the checked range; the fallback is unreachable.
    let step = 2f64.powi(i32::try_from(-scale).unwrap_or(0));
    if !step.is_finite() || step <= 0.0 {
        return None;
    }
    let upper = |index: i64| -> f64 {
        let bound = f64::exp2(index as f64 * step);
        if bound.is_finite() { bound } else { f64::MAX }
    };

    // (upper bound, count) pairs, ascending. Zero-count buckets are dropped:
    // they carry no mass and only bloat the stored row.
    let mut pairs: Vec<(f64, u64)> = Vec::new();
    if let Some(negative) = point.negative.as_ref() {
        for (k, count) in negative.bucket_counts.iter().enumerate().rev() {
            if *count == 0 {
                continue;
            }
            // Bucket vectors are short; saturation only bounds absurd lengths.
            let index =
                i64::from(negative.offset).saturating_add(i64::try_from(k).unwrap_or(i64::MAX));
            pairs.push((-upper(index), *count));
        }
    }
    if point.zero_count > 0 {
        let threshold = if point.zero_threshold.is_finite() {
            point.zero_threshold.max(0.0)
        } else {
            0.0
        };
        pairs.push((threshold, point.zero_count));
    }
    if let Some(positive) = point.positive.as_ref() {
        for (k, count) in positive.bucket_counts.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            let index = i64::from(positive.offset)
                .saturating_add(i64::try_from(k).unwrap_or(i64::MAX))
                .saturating_add(1);
            pairs.push((upper(index), *count));
        }
    }

    // Enforce strictly increasing finite bounds; a clamped or rounded bound
    // that fails to advance merges its mass into the previous bucket.
    let mut bounds: Vec<f64> = Vec::with_capacity(pairs.len());
    let mut bucket_counts: Vec<u64> = Vec::with_capacity(pairs.len());
    for (bound, count) in pairs {
        if !bound.is_finite() {
            continue;
        }
        match bounds.last() {
            Some(&prev) if bound <= prev => {
                if let Some(slot) = bucket_counts.last_mut() {
                    *slot = slot.saturating_add(count);
                }
            }
            _ => {
                bounds.push(bound);
                bucket_counts.push(count);
            }
        }
    }

    Some(ConvertedHistogram {
        count: point.count,
        sum: point.sum.unwrap_or(0.0),
        bounds,
        bucket_counts,
    })
}
