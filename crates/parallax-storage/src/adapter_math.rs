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
