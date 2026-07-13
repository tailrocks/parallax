//! In-memory trace search filtering, aggregation, sorting, and paging.

use super::*;

pub(super) fn search(
    store: &MemoryStore,
    query: &adapter::TraceQuery,
) -> anyhow::Result<adapter::TraceList> {
    let inner = store.lock();
    // `service` matches any trace the service participates in (a span of
    // that service anywhere), not only the root span.
    // Windowed participation + aggregates (plan 075; aligned both adapters).
    let in_window = |ts: u128| {
        query.from_nanos.is_none_or(|from| ts >= from) && query.to_nanos.is_none_or(|to| ts <= to)
    };
    let participating: Option<HashSet<&str>> = query.service.as_deref().map(|svc| {
        inner
            .spans
            .iter()
            .filter(|s| s.service == svc && in_window(s.ts_nanos))
            .map(|s| s.trace_id.as_str())
            .collect()
    });
    // Representative span per trace: the root (no parent), else — when no
    // root was stored — the earliest span, so all-INTERNAL traces still
    // list instead of vanishing.
    let mut rep: HashMap<&str, &SpanRow> = HashMap::new();
    for span in &inner.spans {
        let is_root = span.parent_span_id.as_deref().is_none_or(str::is_empty);
        match rep.get(span.trace_id.as_str()) {
            None => {
                rep.insert(&span.trace_id, span);
            }
            Some(cur) => {
                let cur_root = cur.parent_span_id.as_deref().is_none_or(str::is_empty);
                // Prefer a root; among the same class prefer the earliest.
                let replace = match (cur_root, is_root) {
                    (false, true) => true,
                    (true, false) => false,
                    _ => span.ts_nanos < cur.ts_nanos,
                };
                if replace {
                    rep.insert(&span.trace_id, span);
                }
            }
        }
    }
    // Representative-span filters; newest first.
    let roots: Vec<&SpanRow> = rep
        .into_values()
        .filter(|s| {
            participating
                .as_ref()
                .is_none_or(|set| set.contains(s.trace_id.as_str()))
        })
        .filter(|s| query.from_nanos.is_none_or(|from| s.ts_nanos >= from))
        .filter(|s| query.to_nanos.is_none_or(|to| s.ts_nanos <= to))
        .filter(|s| query.min_duration_ns.is_none_or(|min| s.duration_ns >= min))
        .filter(|s| query.max_duration_ns.is_none_or(|max| s.duration_ns <= max))
        .filter(|s| {
            query
                .name_contains
                .as_deref()
                .is_none_or(|needle| s.name.contains(needle))
        })
        .collect();
    let mut traces: Vec<_> = roots
        .into_iter()
        .map(|root| {
            let mut span_count = 0;
            let mut has_error = false;
            for span in &inner.spans {
                if span.trace_id == root.trace_id && in_window(span.ts_nanos) {
                    span_count += 1;
                    has_error |= span.status_code == "STATUS_CODE_ERROR";
                }
            }
            adapter::TraceSummary {
                trace_id: root.trace_id.clone(),
                root_name: root.name.clone(),
                service: root.service.clone(),
                start_nanos: root.ts_nanos,
                duration_ns: root.duration_ns,
                span_count,
                has_error,
            }
        })
        .collect();
    if query.error_only {
        traces.retain(|t| t.has_error);
    }
    match query.sort {
        adapter::TraceSort::StartDesc => {
            traces.sort_by_key(|t| std::cmp::Reverse(t.start_nanos));
        }
        adapter::TraceSort::DurationDesc => {
            traces.sort_by_key(|t| std::cmp::Reverse(t.duration_ns));
        }
        adapter::TraceSort::DurationAsc => traces.sort_by_key(|t| t.duration_ns),
        adapter::TraceSort::SpanCountDesc => {
            traces.sort_by_key(|t| std::cmp::Reverse(t.span_count));
        }
    }
    let total = traces.len() as u64;
    let items = traces
        .into_iter()
        .skip(query.offset)
        .take(query.limit)
        .collect();
    Ok(adapter::TraceList { items, total })
}
