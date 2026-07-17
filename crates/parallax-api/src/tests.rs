use super::{build_schema, export_schema_sdl, normalize_schema_sdl};

#[test]
fn schema_sdl_snapshot() {
    let sdl = export_schema_sdl();
    for needle in [
        "type Query",
        "type Mutation",
        "issues(",
        "trace(",
        "logsAround(",
        "serviceMap(",
        "metricSeries(",
        "bundle(",
        "story(",
        "sql(",
        "invocation(",
        "invocations(",
        "observedInvocations(",
        "tracesByInvocation(",
        "logsByInvocation(",
        "sessions(",
        "screenVisits(",
        "uiActions(",
        "backgroundCycles(",
        "jobs(",
        "conversations(",
        "invocationStart(",
        "invocationFinish(",
    ] {
        assert!(
            sdl.contains(needle),
            "schema SDL missing sentinel {needle:?}"
        );
    }
    // The runs vocabulary is fully retired (operator, 2026-07-17).
    for forbidden in [
        "observedRuns",
        "tracesByRun",
        "logsByRun",
        "runStart",
        "runFinish",
        "runId",
    ] {
        assert!(
            !sdl.contains(forbidden),
            "schema SDL still exposes retired field {forbidden:?}"
        );
    }
}

#[test]
fn export_schema_sdl_is_byte_deterministic() {
    let a = export_schema_sdl();
    let b = export_schema_sdl();
    assert_eq!(a, b, "two successive exports must be byte-identical");
    assert!(
        a.ends_with('\n'),
        "export must end with exactly one newline"
    );
    assert!(!a.ends_with("\n\n"), "export must not end with blank lines");
    assert!(!a.contains('\r'), "export must use LF line endings only");
    assert!(!a.trim().is_empty(), "export must not be empty");
    // Raw as_sdl may differ only by trailing newlines; normalize is the contract.
    let raw = build_schema().as_sdl();
    assert_eq!(normalize_schema_sdl(&raw), a);
    assert_eq!(normalize_schema_sdl(&format!("{raw}\n\n\n")), a);
    assert_eq!(normalize_schema_sdl(&raw.replace('\n', "\r\n")), a);
}
