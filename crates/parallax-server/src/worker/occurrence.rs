use parallax_storage::model::ErrorEventRow;

pub(super) fn occurrence_id(event: &ErrorEventRow) -> String {
    if !event.trace_id.is_empty()
        && event.trace_id.chars().any(|character| character != '0')
        && !event.span_id.is_empty()
        && event.span_id.chars().any(|character| character != '0')
    {
        format!(
            "v1:span:{}:{}:{}",
            event.trace_id, event.span_id, event.fingerprint
        )
    } else {
        format!(
            "v1:event:{}:{}:{}",
            event.service, event.ts_nanos, event.fingerprint
        )
    }
}
