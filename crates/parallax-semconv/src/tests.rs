use super::*;

#[test]
fn preserves_load_bearing_wire_names() -> Result<(), String> {
    let actual = (
        SERVICE_NAME,
        EVENT_NAME,
        CLI_INVOCATION_ID,
        BUNDLE_WINDOW_METRICS,
    );
    let expected = (
        "service.name",
        "event.name",
        "cli.invocation.id",
        &[
            "process.cpu.utilization",
            "process.memory.usage",
            "tokio.runtime.alive_tasks",
        ][..],
    );
    if actual != expected {
        return Err(format!("semantic-convention wire-name drift: {actual:?}"));
    }
    Ok(())
}
