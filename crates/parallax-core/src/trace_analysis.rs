//! Pure trace analysis over normalized spans.

use crate::fingerprint::normalize_message;
use parallax_storage::model::SpanRow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalHop {
    pub span_id: String,
    pub self_time_ns: u128,
    pub gated_by_child: Option<String>,
    pub clock_suspect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CriticalPath {
    pub hops: Vec<CriticalHop>,
    pub total_gated_ns: u128,
    pub unattached: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffSpan {
    pub span_id: String,
    pub service: String,
    pub name: String,
    pub kind: String,
    pub status_code: String,
    pub duration_ns: u128,
    pub depth: usize,
    pub match_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChangedSpan {
    pub before: DiffSpan,
    pub after: DiffSpan,
    pub duration_delta_ns: i128,
    pub duration_delta_pct: f64,
    pub status_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TraceDiff {
    pub added: Vec<DiffSpan>,
    pub removed: Vec<DiffSpan>,
    pub changed: Vec<ChangedSpan>,
}

#[derive(Clone, Copy)]
struct Interval<'a> {
    span: &'a SpanRow,
    start: u128,
    end: u128,
    clock_suspect: bool,
}

pub fn critical_path(spans: &[SpanRow]) -> CriticalPath {
    if spans.is_empty() {
        return CriticalPath::default();
    }

    let by_id: HashMap<&str, &SpanRow> = spans
        .iter()
        .map(|span| (span.span_id.as_str(), span))
        .collect();
    let mut children: HashMap<&str, Vec<&SpanRow>> = HashMap::new();
    for span in spans {
        if let Some(parent) = span
            .parent_span_id
            .as_deref()
            .filter(|parent| by_id.contains_key(parent))
        {
            children.entry(parent).or_default().push(span);
        }
    }
    for spans in children.values_mut() {
        spans.sort_by(|a, b| {
            a.ts_nanos
                .cmp(&b.ts_nanos)
                .then_with(|| a.span_id.cmp(&b.span_id))
        });
    }

    let mut roots: Vec<&SpanRow> = spans
        .iter()
        .filter(|span| {
            span.parent_span_id
                .as_deref()
                .is_none_or(|parent| parent.is_empty() || !by_id.contains_key(parent))
        })
        .collect();
    if roots.is_empty() {
        roots = spans.iter().collect();
    }
    roots.sort_by(|a, b| {
        a.ts_nanos
            .cmp(&b.ts_nanos)
            .then_with(|| a.span_id.cmp(&b.span_id))
    });
    let root = roots[0];

    let mut hops = Vec::new();
    collect_critical_hops(
        root,
        root.ts_nanos,
        root.ts_nanos.saturating_add(root.duration_ns),
        &children,
        &mut hops,
    );

    let mut attached = BTreeSet::new();
    collect_attached(root, &children, &mut attached);
    let unattached = spans
        .iter()
        .filter(|span| !attached.contains(span.span_id.as_str()))
        .map(|span| span.span_id.clone())
        .collect();
    let total_gated_ns = hops.iter().map(|hop| hop.self_time_ns).sum();

    CriticalPath {
        hops,
        total_gated_ns,
        unattached,
    }
}

fn collect_attached<'a>(
    span: &'a SpanRow,
    children: &HashMap<&'a str, Vec<&'a SpanRow>>,
    attached: &mut BTreeSet<&'a str>,
) {
    if !attached.insert(span.span_id.as_str()) {
        return;
    }
    if let Some(kids) = children.get(span.span_id.as_str()) {
        for child in kids {
            collect_attached(child, children, attached);
        }
    }
}

fn collect_critical_hops<'a>(
    span: &'a SpanRow,
    effective_start: u128,
    effective_end: u128,
    children: &HashMap<&'a str, Vec<&'a SpanRow>>,
    hops: &mut Vec<CriticalHop>,
) {
    let child_intervals = children
        .get(span.span_id.as_str())
        .map(|kids| {
            kids.iter()
                .filter_map(|child| child_interval(child, effective_start, effective_end))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut cursor = effective_start;
    let mut self_time_ns: u128 = 0;
    let mut selected = Vec::new();
    let mut clock_suspect = false;

    while cursor < effective_end {
        let has_active = child_intervals
            .iter()
            .any(|child| child.start <= cursor && child.end > cursor);
        let wave_start = if has_active {
            cursor
        } else {
            let next_start = child_intervals
                .iter()
                .filter(|child| child.start > cursor)
                .min_by(|a, b| {
                    a.start
                        .cmp(&b.start)
                        .then_with(|| a.span.span_id.cmp(&b.span.span_id))
                })
                .map(|child| child.start);
            let Some(next_start) = next_start else {
                self_time_ns = self_time_ns.saturating_add(effective_end - cursor);
                break;
            };
            self_time_ns = self_time_ns.saturating_add(next_start - cursor);
            cursor = next_start;
            next_start
        };

        let mut wave_end = child_intervals
            .iter()
            .filter(|child| child.start <= wave_start && child.end > wave_start)
            .map(|child| child.end)
            .max()
            .unwrap_or(wave_start);
        loop {
            let grown = child_intervals
                .iter()
                .filter(|child| child.start <= wave_end && child.end > wave_start)
                .map(|child| child.end)
                .max()
                .unwrap_or(wave_end);
            if grown == wave_end {
                break;
            }
            wave_end = grown;
        }

        let Some(next) = child_intervals
            .iter()
            .filter(|child| child.start < wave_end && child.end > wave_start)
            .max_by(|a, b| {
                a.end
                    .cmp(&b.end)
                    .then_with(|| b.start.cmp(&a.start))
                    .then_with(|| b.span.span_id.cmp(&a.span.span_id))
            })
            .copied()
        else {
            self_time_ns = self_time_ns.saturating_add(effective_end - cursor);
            break;
        };

        if next.start > cursor {
            self_time_ns = self_time_ns.saturating_add(next.start - cursor);
        }
        clock_suspect |= next.clock_suspect;
        selected.push(next.span);
        cursor = next.end;
    }

    hops.push(CriticalHop {
        span_id: span.span_id.clone(),
        self_time_ns,
        gated_by_child: selected.first().map(|child| child.span_id.clone()),
        clock_suspect,
    });

    for child in selected {
        if let Some(interval) = child_interval(child, effective_start, effective_end) {
            collect_critical_hops(child, interval.start, interval.end, children, hops);
        }
    }
}

fn child_interval(child: &SpanRow, parent_start: u128, parent_end: u128) -> Option<Interval<'_>> {
    let raw_start = child.ts_nanos;
    let raw_end = raw_start.saturating_add(child.duration_ns);
    let start = raw_start.max(parent_start);
    let end = raw_end.min(parent_end);
    (end > start).then_some(Interval {
        span: child,
        start,
        end,
        clock_suspect: raw_start < parent_start || raw_end > parent_end,
    })
}

#[derive(Clone)]
struct MatchEntry<'a> {
    span: &'a SpanRow,
    depth: usize,
    key: String,
}

pub fn compare(before: &[SpanRow], after: &[SpanRow]) -> TraceDiff {
    let before_entries = match_entries(before);
    let after_entries = match_entries(after);
    let before_by_key: BTreeMap<_, _> = before_entries
        .iter()
        .map(|entry| (entry.key.clone(), entry.clone()))
        .collect();
    let after_by_key: BTreeMap<_, _> = after_entries
        .iter()
        .map(|entry| (entry.key.clone(), entry.clone()))
        .collect();

    let removed = before_entries
        .iter()
        .filter(|entry| !after_by_key.contains_key(&entry.key))
        .map(diff_span)
        .collect();
    let added = after_entries
        .iter()
        .filter(|entry| !before_by_key.contains_key(&entry.key))
        .map(diff_span)
        .collect();
    let changed = after_entries
        .iter()
        .filter_map(|after_entry| {
            let before_entry = before_by_key.get(&after_entry.key)?;
            let duration_delta_ns =
                after_entry.span.duration_ns as i128 - before_entry.span.duration_ns as i128;
            let status_changed = before_entry.span.status_code != after_entry.span.status_code;
            if duration_delta_ns == 0 && !status_changed {
                return None;
            }
            let duration_delta_pct = if before_entry.span.duration_ns == 0 {
                if duration_delta_ns == 0 { 0.0 } else { 100.0 }
            } else {
                duration_delta_ns as f64 / before_entry.span.duration_ns as f64 * 100.0
            };
            Some(ChangedSpan {
                before: diff_span(before_entry),
                after: diff_span(after_entry),
                duration_delta_ns,
                duration_delta_pct,
                status_changed,
            })
        })
        .collect();

    TraceDiff {
        added,
        removed,
        changed,
    }
}

fn match_entries(spans: &[SpanRow]) -> Vec<MatchEntry<'_>> {
    let by_id: HashMap<&str, &SpanRow> = spans
        .iter()
        .map(|span| (span.span_id.as_str(), span))
        .collect();
    let mut children: HashMap<Option<&str>, Vec<&SpanRow>> = HashMap::new();
    for span in spans {
        let parent = span
            .parent_span_id
            .as_deref()
            .filter(|parent| by_id.contains_key(parent));
        children.entry(parent).or_default().push(span);
    }
    for kids in children.values_mut() {
        kids.sort_by(|a, b| {
            a.ts_nanos
                .cmp(&b.ts_nanos)
                .then_with(|| a.span_id.cmp(&b.span_id))
        });
    }

    let mut entries = Vec::new();
    let mut visited = HashSet::new();
    collect_match_entries(None, "", 0, &children, &mut visited, &mut entries);
    entries
}

fn collect_match_entries<'a>(
    parent: Option<&'a str>,
    parent_key: &str,
    depth: usize,
    children: &HashMap<Option<&'a str>, Vec<&'a SpanRow>>,
    visited: &mut HashSet<&'a str>,
    entries: &mut Vec<MatchEntry<'a>>,
) {
    let Some(kids) = children.get(&parent) else {
        return;
    };
    let mut sibling_counts: HashMap<String, usize> = HashMap::new();
    for span in kids {
        if !visited.insert(span.span_id.as_str()) {
            continue;
        }
        let base = format!(
            "{}>{}|{}|{}|{}",
            parent_key,
            span.service,
            span.kind,
            depth,
            normalize_operation_name(&span.name)
        );
        let sibling_index = sibling_counts.entry(base.clone()).or_insert(0);
        let key = format!("{base}|{}", *sibling_index);
        *sibling_index += 1;
        entries.push(MatchEntry {
            span,
            depth,
            key: key.clone(),
        });
        collect_match_entries(
            Some(span.span_id.as_str()),
            &key,
            depth + 1,
            children,
            visited,
            entries,
        );
    }
}

fn normalize_operation_name(name: &str) -> String {
    normalize_message(name)
}

fn diff_span(entry: &MatchEntry<'_>) -> DiffSpan {
    DiffSpan {
        span_id: entry.span.span_id.clone(),
        service: entry.span.service.clone(),
        name: entry.span.name.clone(),
        kind: entry.span.kind.clone(),
        status_code: entry.span.status_code.clone(),
        duration_ns: entry.span.duration_ns,
        depth: entry.depth,
        match_key: entry.key.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(id: &str, parent: Option<&str>, start: u128, duration: u128) -> SpanRow {
        SpanRow {
            ts_nanos: start,
            service: "api".into(),
            trace_id: "trace".into(),
            span_id: id.into(),
            parent_span_id: parent.map(Into::into),
            name: id.into(),
            kind: "SPAN_KIND_INTERNAL".into(),
            status_code: "STATUS_CODE_UNSET".into(),
            status_message: String::new(),
            duration_ns: duration,
            run_id: None,
            scope_name: String::new(),
            events: None,
            links: serde_json::Value::Null,
            attributes: serde_json::Value::Null,
            resource: serde_json::Value::Null,
        }
    }

    fn named(
        id: &str,
        parent: Option<&str>,
        service: &str,
        name: &str,
        start: u128,
        duration: u128,
        status: &str,
    ) -> SpanRow {
        let mut row = span(id, parent, start, duration);
        row.service = service.into();
        row.name = name.into();
        row.status_code = status.into();
        row
    }

    fn path_ids(path: &CriticalPath) -> Vec<&str> {
        path.hops
            .iter()
            .map(|hop| hop.span_id.as_str())
            .collect::<Vec<_>>()
    }

    #[test]
    fn critical_path_linear_chain_counts_parent_self_time() {
        let spans = vec![
            span("root", None, 0, 100),
            span("child", Some("root"), 20, 50),
            span("grandchild", Some("child"), 30, 20),
        ];

        let path = critical_path(&spans);

        assert_eq!(path_ids(&path), vec!["root", "child", "grandchild"]);
        assert_eq!(path.total_gated_ns, 100);
        assert_eq!(path.hops[0].self_time_ns, 50);
        assert_eq!(path.hops[1].self_time_ns, 30);
        assert_eq!(path.hops[2].self_time_ns, 20);
    }

    #[test]
    fn critical_path_parallel_fanout_chooses_latest_end() {
        let spans = vec![
            span("root", None, 0, 100),
            span("fast", Some("root"), 10, 20),
            span("slow", Some("root"), 12, 70),
        ];

        let path = critical_path(&spans);

        assert_eq!(path_ids(&path), vec!["root", "slow"]);
        assert_eq!(path.hops[0].gated_by_child.as_deref(), Some("slow"));
        assert_eq!(path.total_gated_ns, 100);
    }

    #[test]
    fn critical_path_switches_across_sequential_waves() {
        let spans = vec![
            span("root", None, 0, 120),
            span("a", Some("root"), 0, 40),
            span("b", Some("root"), 5, 20),
            span("c", Some("root"), 45, 65),
        ];

        let path = critical_path(&spans);

        assert_eq!(path_ids(&path), vec!["root", "a", "c"]);
        assert_eq!(path.total_gated_ns, 120);
    }

    #[test]
    fn critical_path_reports_unattached_roots() {
        let spans = vec![span("root-a", None, 20, 10), span("root-b", None, 10, 10)];

        let path = critical_path(&spans);

        assert_eq!(path_ids(&path), vec!["root-b"]);
        assert_eq!(path.unattached, vec!["root-a"]);
    }

    #[test]
    fn critical_path_clamps_child_overrun_and_flags_clock_suspect() {
        let spans = vec![
            span("root", None, 10, 50),
            span("child", Some("root"), 0, 100),
        ];

        let path = critical_path(&spans);

        assert_eq!(path_ids(&path), vec!["root", "child"]);
        assert!(path.hops[0].clock_suspect);
        assert_eq!(path.total_gated_ns, 50);
    }

    #[test]
    fn critical_path_handles_zero_duration_spans() {
        let spans = vec![span("root", None, 0, 0), span("child", Some("root"), 0, 0)];

        let path = critical_path(&spans);

        assert_eq!(path_ids(&path), vec!["root"]);
        assert_eq!(path.total_gated_ns, 0);
    }

    #[test]
    fn compare_identical_traces_has_empty_diff() {
        let spans = vec![named(
            "a",
            None,
            "api",
            "GET /orders/123",
            0,
            10,
            "STATUS_CODE_UNSET",
        )];

        let diff = compare(&spans, &spans);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.changed.is_empty());
    }

    #[test]
    fn compare_matches_names_with_volatile_ids() {
        let before = vec![named(
            "a",
            None,
            "api",
            "GET /orders/order-123",
            0,
            10,
            "STATUS_CODE_UNSET",
        )];
        let after = vec![named(
            "b",
            None,
            "api",
            "GET /orders/order-456",
            0,
            15,
            "STATUS_CODE_UNSET",
        )];

        let diff = compare(&before, &after);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].duration_delta_ns, 5);
    }

    #[test]
    fn compare_reports_added_retry_sibling() {
        let before = vec![named(
            "root",
            None,
            "api",
            "checkout",
            0,
            20,
            "STATUS_CODE_UNSET",
        )];
        let after = vec![
            named("root", None, "api", "checkout", 0, 20, "STATUS_CODE_UNSET"),
            named(
                "retry",
                Some("root"),
                "api",
                "retry",
                10,
                5,
                "STATUS_CODE_UNSET",
            ),
        ];

        let diff = compare(&before, &after);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "retry");
    }

    #[test]
    fn compare_keeps_same_operation_children_under_different_parents_distinct() {
        let before = vec![
            named(
                "root-a",
                None,
                "api",
                "checkout",
                0,
                100,
                "STATUS_CODE_UNSET",
            ),
            named(
                "branch-a",
                Some("root-a"),
                "api",
                "fanout",
                10,
                40,
                "STATUS_CODE_UNSET",
            ),
            named(
                "branch-b",
                Some("root-a"),
                "api",
                "fanout",
                20,
                40,
                "STATUS_CODE_UNSET",
            ),
            named(
                "select-a",
                Some("branch-a"),
                "db",
                "SELECT stock",
                12,
                10,
                "STATUS_CODE_UNSET",
            ),
            named(
                "select-b",
                Some("branch-b"),
                "db",
                "SELECT stock",
                22,
                10,
                "STATUS_CODE_UNSET",
            ),
        ];
        let after = vec![
            named(
                "root-b",
                None,
                "api",
                "checkout",
                0,
                100,
                "STATUS_CODE_UNSET",
            ),
            named(
                "branch-c",
                Some("root-b"),
                "api",
                "fanout",
                10,
                40,
                "STATUS_CODE_UNSET",
            ),
            named(
                "branch-d",
                Some("root-b"),
                "api",
                "fanout",
                20,
                40,
                "STATUS_CODE_UNSET",
            ),
            named(
                "select-d",
                Some("branch-d"),
                "db",
                "SELECT stock",
                22,
                10,
                "STATUS_CODE_UNSET",
            ),
        ];

        let diff = compare(&before, &after);

        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].span_id, "select-a");
        assert!(diff.added.is_empty());
    }

    #[test]
    fn compare_reports_status_change() {
        let before = vec![named(
            "a",
            None,
            "api",
            "checkout",
            0,
            10,
            "STATUS_CODE_UNSET",
        )];
        let after = vec![named(
            "b",
            None,
            "api",
            "checkout",
            0,
            10,
            "STATUS_CODE_ERROR",
        )];

        let diff = compare(&before, &after);

        assert_eq!(diff.changed.len(), 1);
        assert!(diff.changed[0].status_changed);
    }
}
