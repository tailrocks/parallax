use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use super::is_hex;

pub(super) fn validate(value: &serde_json::Value, c0: &str, auditor: &str) -> Result<()> {
    let object = value
        .as_object()
        .context("closure packet must be a JSON object")?;
    let expected = BTreeSet::from([
        "artifact_hashes",
        "audited_commit",
        "auditor",
        "blocked_triggers",
        "clean_state",
        "commands",
        "exceptions",
        "findings",
        "independent",
        "result",
        "schema_version",
        "tool_versions",
    ]);
    ensure!(
        object.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected,
        "closure packet fields differ from schema"
    );
    ensure!(
        value["schema_version"] == 1,
        "invalid closure packet schema"
    );
    ensure!(
        value["audited_commit"] == c0,
        "closure packet names wrong C0"
    );
    ensure!(value["auditor"] == auditor, "closure packet auditor drift");
    ensure!(
        value["clean_state"] == true,
        "closure checkout was not clean"
    );
    ensure!(value["independent"] == true, "auditor was not independent");
    ensure!(
        value["result"] == "pass",
        "closure packet result is not pass"
    );
    validate_string_map(&value["tool_versions"], "tool_versions")?;
    validate_hash_map(&value["artifact_hashes"])?;
    validate_commands(&value["commands"])?;
    validate_record_array(&value["findings"], "findings", false)?;
    validate_record_array(&value["exceptions"], "exceptions", false)?;
    validate_record_array(&value["blocked_triggers"], "blocked_triggers", true)?;
    Ok(())
}

fn validate_string_map(value: &serde_json::Value, label: &str) -> Result<()> {
    let values = value
        .as_object()
        .with_context(|| format!("{label} must be an object"))?;
    ensure!(!values.is_empty(), "{label} must not be empty");
    for (key, value) in values {
        ensure!(!key.trim().is_empty(), "{label} contains an empty key");
        ensure!(
            value.as_str().is_some_and(|value| !value.trim().is_empty()),
            "{label}.{key} must be a non-empty string"
        );
    }
    Ok(())
}

fn validate_hash_map(value: &serde_json::Value) -> Result<()> {
    let values = value
        .as_object()
        .context("artifact_hashes must be an object")?;
    ensure!(!values.is_empty(), "artifact_hashes must not be empty");
    for (path, value) in values {
        let hash = value.as_str().context("artifact hash must be a string")?;
        ensure!(!path.trim().is_empty(), "artifact hash path is empty");
        ensure!(is_hex(hash, 64), "artifact hash is not lowercase SHA-256");
    }
    Ok(())
}

fn validate_commands(value: &serde_json::Value) -> Result<()> {
    let commands = value.as_array().context("commands must be an array")?;
    ensure!(!commands.is_empty(), "commands must not be empty");
    for command in commands {
        let object = command.as_object().context("command must be an object")?;
        ensure!(
            object.keys().map(String::as_str).collect::<BTreeSet<_>>()
                == BTreeSet::from(["command", "evidence_sha256", "exit_code"]),
            "command fields differ from schema"
        );
        ensure!(
            command["command"]
                .as_str()
                .is_some_and(|value| !value.trim().is_empty()),
            "command text is empty"
        );
        ensure!(command["exit_code"] == 0, "closure command did not pass");
        ensure!(
            command["evidence_sha256"]
                .as_str()
                .is_some_and(|hash| is_hex(hash, 64)),
            "command evidence hash is invalid"
        );
    }
    Ok(())
}

fn validate_record_array(value: &serde_json::Value, label: &str, required: bool) -> Result<()> {
    let records = value
        .as_array()
        .with_context(|| format!("{label} must be an array"))?;
    if required {
        ensure!(!records.is_empty(), "{label} must not be empty");
    }
    for record in records {
        let object = record
            .as_object()
            .with_context(|| format!("{label} entry must be an object"))?;
        ensure!(!object.is_empty(), "{label} entry must not be empty");
        for (key, value) in object {
            ensure!(!key.trim().is_empty(), "{label} contains an empty key");
            ensure!(
                value.as_str().is_some_and(|value| !value.trim().is_empty()),
                "{label}.{key} must be a non-empty string"
            );
        }
    }
    Ok(())
}
