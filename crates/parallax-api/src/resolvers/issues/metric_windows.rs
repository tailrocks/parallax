use juniper::FieldResult;
use parallax_analysis::semconv;
use parallax_storage::model::MetricAgg;

use crate::{ApiContext, internal_field_err};

pub(crate) async fn bundle_metric_windows(
    context: &ApiContext,
    inputs: &parallax_evidence::bundle::BundleInputs,
) -> FieldResult<Vec<parallax_evidence::bundle::MetricWindow>> {
    use parallax_evidence::bundle::{BundleAnchor, MetricWindow};
    const PAD_NANOS: u128 = 5 * 60 * 1_000_000_000;
    let (from, to, step_seconds, run_scope, service) =
        if let BundleAnchor::Invocation { invocation, .. } = &inputs.anchor {
            let last_activity = inputs
                .trace_logs
                .iter()
                .map(|l| l.ts_nanos)
                .chain(
                    inputs
                        .trace_spans
                        .iter()
                        .map(|s| s.ts_nanos + s.duration_ns),
                )
                .max();
            let start = invocation.started_at_nanos;
            let end = invocation
                .ended_at_nanos
                .into_iter()
                .chain(last_activity)
                .max()
                .unwrap_or(start);
            (
                start.saturating_sub(5_000_000_000),
                end + 30_000_000_000,
                5u32,
                Some(invocation.invocation_id.clone()),
                None,
            )
        } else {
            let anchor_ts = inputs
                .events
                .first()
                .map(|e| e.ts_nanos)
                .or_else(|| inputs.trace_spans.first().map(|s| s.ts_nanos));
            let Some(anchor_ts) = anchor_ts else {
                return Ok(Vec::new());
            };
            let invocation_id = inputs
                .trace_spans
                .iter()
                .find_map(|s| s.invocation_id.clone());
            let service = inputs
                .trace_spans
                .first()
                .map(|s| s.service.clone())
                .or_else(|| inputs.events.first().map(|e| e.service.clone()));
            (
                anchor_ts.saturating_sub(PAD_NANOS),
                anchor_ts + PAD_NANOS,
                30u32,
                invocation_id,
                service,
            )
        };
    let scope = if run_scope.is_some() {
        "invocation"
    } else {
        "service"
    };
    let step = u128::from(step_seconds) * 1_000_000_000;
    let (cpu, memory, tokio_tasks) = tokio::try_join!(
        context.store.metric_series(
            semconv::BUNDLE_WINDOW_METRICS[0],
            service.as_deref(),
            run_scope.as_deref(),
            &[],
            from..=to,
            step,
            MetricAgg::Avg,
        ),
        context.store.metric_series(
            semconv::BUNDLE_WINDOW_METRICS[1],
            service.as_deref(),
            run_scope.as_deref(),
            &[],
            from..=to,
            step,
            MetricAgg::Avg,
        ),
        context.store.metric_series(
            semconv::BUNDLE_WINDOW_METRICS[2],
            service.as_deref(),
            run_scope.as_deref(),
            &[],
            from..=to,
            step,
            MetricAgg::Avg,
        ),
    )
    .map_err(internal_field_err)?;
    let mut windows = Vec::new();
    for (metric, points) in [
        (semconv::BUNDLE_WINDOW_METRICS[0], cpu),
        (semconv::BUNDLE_WINDOW_METRICS[1], memory),
        (semconv::BUNDLE_WINDOW_METRICS[2], tokio_tasks),
    ] {
        if let Some(window) = MetricWindow::from_points(
            metric,
            scope,
            from,
            to,
            step_seconds,
            points.into_iter().map(|p| (p.ts_nanos, p.value)).collect(),
        ) {
            windows.push(window);
        }
    }
    Ok(windows)
}
