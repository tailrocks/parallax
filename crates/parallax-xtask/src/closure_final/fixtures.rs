use super::*;

pub(super) fn run() -> Result<()> {
    let sha = "0123456789abcdef0123456789abcdef01234567";
    let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let output_digest = format!("{:x}", Sha256::digest(b"passed"));
    let a = parse_attestation(&format!(
        "auditor=source-a;c0={sha};c1={sha};tree={sha};result=pass;digest={digest}"
    ))?;
    let b = parse_attestation(&format!(
        "auditor=source-b;c0={sha};c1={sha};tree={sha};result=pass;digest={digest}"
    ))?;
    validate_attestations(&a, &b, sha, sha, sha)?;
    let packet = serde_json::json!({
        "schema_version": 1,
        "audited_commit": sha,
        "clean_state": true,
        "auditor": "source-a",
        "independent": true,
        "tool_versions": {"rustc": "1.97.0"},
        "commands": [{"command": "cargo xtask ci --full", "exit_code": 0, "output": "passed", "output_sha256": output_digest}],
        "artifact_hashes": {"target/report.json": digest},
        "findings": [],
        "exceptions": [],
        "blocked_triggers": [{"plan": "089", "evidence": "upstream feature graph", "unblock": "native TLS support"}],
        "result": "pass"
    });
    packet::validate(&packet, sha, "source-a")?;
    ensure!(
        parse_attestation(&format!(
            "auditor=source-a;c0={sha};c1={sha};tree={sha};result=fail;digest={digest}"
        ))
        .is_err(),
        "tampered result fixture passed"
    );
    ensure!(
        validate_paths(&["src/main.rs".into()], &CLOSURE_PATHS, false).is_err(),
        "tampered path fixture passed"
    );
    let mut hollow_packet = packet;
    hollow_packet["commands"] = serde_json::json!([]);
    ensure!(
        packet::validate(&hollow_packet, sha, "source-a").is_err(),
        "hollow packet fixture passed"
    );
    let trailers = "closure\n\nSigned-off-by: Codex <codex@openai.com>\nCo-authored-by: Codex <codex@openai.com>\n";
    validate_commit_trailers(trailers)?;
    ensure!(
        validate_commit_trailers(&trailers.replace("Signed-off-by: ", "")).is_err(),
        "missing DCO fixture passed"
    );
    Ok(())
}
