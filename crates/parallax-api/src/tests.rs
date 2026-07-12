use super::build_schema;

#[test]
fn schema_sdl_snapshot() {
    let sdl = build_schema().as_sdl();
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
    ] {
        assert!(
            sdl.contains(needle),
            "schema SDL missing sentinel {needle:?}"
        );
    }
}
