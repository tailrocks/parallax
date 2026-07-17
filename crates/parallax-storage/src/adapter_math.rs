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
