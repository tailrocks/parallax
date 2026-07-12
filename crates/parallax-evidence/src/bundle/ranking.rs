use super::*;

pub(super) fn rank_hypotheses(
    primary_issue: Option<&Issue>,
    events: &[ErrorEventRow],
    trace: Option<&TraceSection>,
    anchor: &Anchor,
) -> Vec<Hypothesis> {
    let mut hypotheses = Vec::new();
    let message = events
        .first()
        .map(|e| e.message.to_lowercase())
        .unwrap_or_default();
    let anchor_evidence = format!("{} {}", anchor.kind, anchor.id);
    let error_type = primary_issue
        .map(|i| i.error_type.as_str())
        .or_else(|| events.first().map(|e| e.error_type.as_str()))
        .unwrap_or("The error");

    if [
        "timed out",
        "timeout",
        "pool",
        "connection refused",
        "connection reset",
    ]
    .iter()
    .any(|p| message.contains(p))
    {
        hypotheses.push(Hypothesis {
            kind: "dependency_failure",
            statement: format!(
                "{error_type} points at a downstream dependency timing out or saturated; check \
                 that dependency's capacity and latency in this window."
            ),
            confidence: "medium",
            evidence: vec!["latest event message".to_string(), anchor_evidence.clone()],
        });
    }

    if let Some(trace) = trace {
        if let Some(slowest) = trace.spans.iter().max_by_key(|s| s.duration_us)
            && slowest.duration_us > 1_000_000
        {
            hypotheses.push(Hypothesis {
                kind: "slow_span",
                statement: format!(
                    "Span `{}` in {} took {}ms — the dominant cost in the failing trace.",
                    slowest.name,
                    slowest.service,
                    slowest.duration_us / 1_000
                ),
                confidence: "medium",
                evidence: vec![format!("trace {}", trace.trace_id)],
            });
        }
        if let Some(db) = trace.spans.iter().find(|s| s.db_query.is_some()) {
            hypotheses.push(Hypothesis {
                kind: "database_involved",
                statement: format!(
                    "The failing trace touches the database in `{}` — inspect the captured \
                     query and its plan.",
                    db.name
                ),
                confidence: "low",
                evidence: vec![format!("trace {}", trace.trace_id)],
            });
        }
    }

    if hypotheses.is_empty() {
        hypotheses.push(Hypothesis {
            kind: "insufficient_evidence",
            statement: "The evidence does not support a root-cause hypothesis; see \
                        missing_evidence for what to instrument next."
                .into(),
            confidence: "low",
            evidence: vec![anchor_evidence],
        });
    }
    hypotheses
}
