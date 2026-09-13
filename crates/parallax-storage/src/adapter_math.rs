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

/// Linear-interpolated quantile over one explicit bucket grid: `bounds[i]`
/// is the upper bound of `counts[i]` (ascending, first bucket `(-inf,
/// bounds[0]]`). Both stores use this for converted exponential histograms
/// so the memory and Greptime paths cannot drift apart. Empty mass → 0.
pub fn explicit_bucket_quantile(bounds: &[f64], counts: &[u64], q: f64) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let target = q.clamp(0.0, 1.0) * total as f64;
    let mut cumulative = 0u64;
    for (index, count) in counts.iter().enumerate() {
        let next = cumulative.saturating_add(*count);
        if next as f64 >= target {
            let lower = if index == 0 {
                0.0
            } else {
                bounds.get(index - 1).copied().unwrap_or(0.0)
            };
            let upper = bounds.get(index).copied().unwrap_or(lower);
            let within = if *count == 0 {
                0.0
            } else {
                (target - cumulative as f64) / *count as f64
            };
            return lower + (upper - lower) * within;
        }
        cumulative = next;
    }
    bounds.last().copied().unwrap_or(0.0)
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
mod tests {
    use super::*;

    #[test]
    fn exp_quantile_interpolates_within_bucket() {
        // bounds [2, 4], counts [3, 5]: total 8, p50 target 4 → second bucket,
        // lower 2, within (4-3)/5 = 0.2 → 2 + 2*0.2 = 2.4.
        let q = explicit_bucket_quantile(&[2.0, 4.0], &[3, 5], 0.5);
        assert!((q - 2.4).abs() < 1e-9, "got {q}");
    }

    #[test]
    fn exp_quantile_empty_mass_is_zero() {
        for q in [0.5, 0.99] {
            assert!(explicit_bucket_quantile(&[], &[], q).abs() < f64::EPSILON);
            assert!(explicit_bucket_quantile(&[1.0], &[0], q).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn exp_quantile_negative_bounds_stay_negative() {
        // Converted negative bucket: bounds [-2, 0], all mass in first bucket.
        let q = explicit_bucket_quantile(&[-2.0, 0.0], &[7, 2], 0.5);
        assert!(q < 0.0, "got {q}");
    }
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
