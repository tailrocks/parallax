use super::*;

#[expect(
    clippy::expect_used,
    reason = "Bundle serialization is a type invariant; panic is the fail-loud contract"
)]
pub(super) fn estimate_bundle_tokens(bundle: &Bundle) -> usize {
    estimate_tokens(&serde_json::to_string(bundle).expect("Bundle serialization is infallible"))
}

pub(super) fn retain_top_trace_spans(trace: &mut TraceSection, keep: usize) {
    if keep >= trace.spans.len() {
        return;
    }
    let mut ranked: Vec<usize> = (0..trace.spans.len()).collect();
    ranked.sort_by(|&a, &b| {
        let a_span = &trace.spans[a];
        let b_span = &trace.spans[b];
        let a_error = a_span.status_code.contains("ERROR");
        let b_error = b_span.status_code.contains("ERROR");
        b_error
            .cmp(&a_error)
            .then_with(|| b_span.duration_us.cmp(&a_span.duration_us))
            .then_with(|| a.cmp(&b))
    });
    let mut selected = vec![false; trace.spans.len()];
    for index in ranked.into_iter().take(keep.max(1)) {
        selected[index] = true;
    }
    let spans = std::mem::take(&mut trace.spans);
    trace.spans = spans
        .into_iter()
        .enumerate()
        .filter_map(|(index, span)| selected[index].then_some(span))
        .collect();
}

pub(super) fn bound_trace_spans(bundle: &mut Bundle, max_tokens: usize) {
    let Some(original_len) = bundle.trace.as_ref().map(|trace| trace.spans.len()) else {
        return;
    };
    if original_len <= 1 {
        return;
    }
    let mut keep = original_len;
    while estimate_bundle_tokens(bundle) > max_tokens && keep > 1 {
        keep = keep.saturating_sub((keep / 4).max(1));
        if let Some(trace) = bundle.trace.as_mut() {
            retain_top_trace_spans(trace, keep);
        }
    }
    let final_len = bundle
        .trace
        .as_ref()
        .map(|trace| trace.spans.len())
        .unwrap_or(0);
    let dropped = original_len.saturating_sub(final_len);
    if dropped > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: dropped {dropped} trace spans to fit budget"
        ));
    }
}

pub(super) fn decimate_points(points: &mut Vec<MetricPointLine>, keep: usize) -> usize {
    if keep >= points.len() {
        return 0;
    }
    let original_len = points.len();
    let keep = keep.max(1);
    let mut selected = vec![false; original_len];
    if keep == 1 {
        selected[original_len - 1] = true;
    } else {
        for slot in 0..keep {
            selected[slot * (original_len - 1) / (keep - 1)] = true;
        }
    }
    let mut selected_count = selected.iter().filter(|&&keep| keep).count();
    for keep_slot in &mut selected {
        if selected_count >= keep {
            break;
        }
        if !*keep_slot {
            *keep_slot = true;
            selected_count += 1;
        }
    }
    let old = std::mem::take(points);
    *points = old
        .into_iter()
        .enumerate()
        .filter_map(|(index, point)| selected[index].then_some(point))
        .collect();
    original_len - points.len()
}

pub(super) fn bound_metric_windows(bundle: &mut Bundle, max_tokens: usize) {
    let mut dropped = 0usize;
    loop {
        if estimate_bundle_tokens(bundle) <= max_tokens {
            break;
        }
        let mut changed = false;
        for window in &mut bundle.metric_windows {
            if window.points.len() <= 2 {
                continue;
            }
            let keep = window
                .points
                .len()
                .saturating_sub((window.points.len() / 4).max(1));
            dropped += decimate_points(&mut window.points, keep.max(2));
            changed = true;
        }
        if !changed {
            break;
        }
    }
    if dropped > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: dropped {dropped} metric points to fit budget"
        ));
    }
}
