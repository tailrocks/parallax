use parallax_model::SeriesPoint;

pub fn attribute_compare_score(
    selected_count: u64,
    selected_total: u64,
    baseline_count: u64,
    baseline_total: u64,
) -> f64 {
    let selected_share = if selected_total == 0 {
        0.0
    } else {
        selected_count as f64 / selected_total as f64
    };
    let baseline_share = if baseline_total == 0 {
        0.0
    } else {
        baseline_count as f64 / baseline_total as f64
    };
    (selected_share - baseline_share).clamp(0.0, 1.0)
}

/// Reset-clamped counter delta per bucket (`increase`): like `rate` but not
/// divided by the step, so the value is the raw growth inside each bucket.
pub fn increase_from_buckets(series: &[SeriesPoint]) -> Vec<SeriesPoint> {
    series
        .windows(2)
        .map(|window| SeriesPoint {
            ts_nanos: window[1].ts_nanos,
            value: (window[1].value - window[0].value).max(0.0),
        })
        .collect()
}

/// Histogram average per bucket from cumulative `_sum`/`_count` series
/// (aligned by bucket timestamp): Δsum/Δcount, reset-clamped; empty-growth
/// buckets are skipped rather than emitted as 0/0.
pub fn histogram_avg_from_cumulative(
    sums: &[SeriesPoint],
    counts: &[SeriesPoint],
) -> Vec<SeriesPoint> {
    let counts: std::collections::BTreeMap<u128, f64> =
        counts.iter().map(|p| (p.ts_nanos, p.value)).collect();
    sums.windows(2)
        .filter_map(|window| {
            let (prev, cur) = (&window[0], &window[1]);
            let count_delta = (counts.get(&cur.ts_nanos)? - counts.get(&prev.ts_nanos)?).max(0.0);
            if count_delta <= 0.0 {
                return None;
            }
            let sum_delta = (cur.value - prev.value).max(0.0);
            Some(SeriesPoint {
                ts_nanos: cur.ts_nanos,
                value: sum_delta / count_delta,
            })
        })
        .collect()
}

pub fn rate_from_buckets(series: &[SeriesPoint], step_nanos: u128) -> Vec<SeriesPoint> {
    let step_secs = step_nanos as f64 / 1e9;
    series
        .windows(2)
        .map(|window| SeriesPoint {
            ts_nanos: window[1].ts_nanos,
            value: ((window[1].value - window[0].value).max(0.0)) / step_secs,
        })
        .collect()
}

#[cfg(test)]
mod property_tests {
    //! Plan-103 bounded property suites. Defect classes and oracles are
    //! documented in docs/research/testing/property-invariants.md.
    use super::*;
    use proptest::prelude::*;

    fn finite_series(max_len: usize) -> impl Strategy<Value = Vec<SeriesPoint>> {
        proptest::collection::vec((0u64..1_000_000, -1.0e12f64..1.0e12), 0..max_len).prop_map(
            |raw| {
                let mut ts = 0u128;
                raw.into_iter()
                    .map(|(dt, value)| {
                        ts += u128::from(dt) + 1;
                        SeriesPoint {
                            ts_nanos: ts,
                            value,
                        }
                    })
                    .collect()
            },
        )
    }

    proptest! {
        /// Counter deltas clamp at reset: rate and increase are never
        /// negative and never non-finite for finite inputs.
        #[test]
        fn rate_and_increase_never_negative(series in finite_series(64), step in 1u128..1_000_000_000_000) {
            for point in rate_from_buckets(&series, step) {
                prop_assert!(point.value >= 0.0 && point.value.is_finite());
            }
            for point in increase_from_buckets(&series) {
                prop_assert!(point.value >= 0.0 && point.value.is_finite());
            }
            prop_assert_eq!(
                increase_from_buckets(&series).len(),
                series.len().saturating_sub(1)
            );
        }

        /// Histogram Δsum/Δcount averages are finite and non-negative for
        /// cumulative (non-decreasing) inputs, and zero-growth buckets are
        /// skipped rather than emitted as 0/0.
        #[test]
        fn histogram_avg_finite_nonnegative(
            deltas in proptest::collection::vec((0.0f64..1.0e9, 0u32..1_000), 0..32)
        ) {
            let mut sum = 0.0;
            let mut count = 0.0;
            let mut sums = Vec::new();
            let mut counts = Vec::new();
            for (index, (ds, dc)) in deltas.iter().enumerate() {
                sum += ds;
                count += f64::from(*dc);
                let ts = (index as u128 + 1) * 1_000;
                sums.push(SeriesPoint { ts_nanos: ts, value: sum });
                counts.push(SeriesPoint { ts_nanos: ts, value: count });
            }
            for point in histogram_avg_from_cumulative(&sums, &counts) {
                prop_assert!(point.value.is_finite(), "finite avg");
                prop_assert!(point.value >= 0.0, "non-negative avg");
            }
        }
    }
}
