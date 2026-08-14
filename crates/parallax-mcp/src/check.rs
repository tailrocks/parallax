//! Projection-equivalence proof: MCP tool path ↔ CLI ↔ plain HTTP GraphQL.

use crate::gql::{self, GraphqlClient, MCP_BUNDLE_MAX_TOKENS};
use anyhow::Context;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::process::Command;

#[derive(Debug)]
pub(crate) struct CheckArgs {
    pub base_url: String,
    pub fingerprint: Option<String>,
    pub invocation_id: Option<String>,
    /// Path to the `parallax` CLI binary (default: look up on PATH).
    pub parallax_bin: String,
}

pub(crate) async fn run(args: CheckArgs) -> anyhow::Result<()> {
    let client = GraphqlClient::new(args.base_url.clone())?;
    let mut cases: Vec<Case> = Vec::new();

    if let Some(fp) = &args.fingerprint {
        cases.push(Case {
            label: "issue bundle",
            kind: CaseKind::IssueBundle {
                fingerprint: fp.clone(),
            },
        });
    }
    if let Some(rid) = &args.invocation_id {
        cases.push(Case {
            label: "invocation bundle",
            kind: CaseKind::RunBundle {
                invocation_id: rid.clone(),
            },
        });
    }

    if cases.is_empty() {
        anyhow::bail!("check requires --fingerprint and/or --invocation-id");
    }

    let mut failed = 0usize;
    for case in &cases {
        match check_one(&client, &args, case).await {
            Ok(()) => println!("equivalence: OK  ({})", case.label),
            Err(_) => {
                failed += 1;
                println!("equivalence: FAIL ({})", case.label);
                println!("details omitted because upstream errors may contain sensitive evidence");
            }
        }
    }

    if failed > 0 {
        anyhow::bail!("{failed}/{} equivalence case(s) failed", cases.len());
    }
    println!(
        "equivalence: OK for all {} case(s) (CLI ≡ HTTP ≡ MCP raw JSON; hash definition confirmed)",
        cases.len()
    );
    Ok(())
}

struct Case {
    label: &'static str,
    kind: CaseKind,
}

enum CaseKind {
    IssueBundle { fingerprint: String },
    RunBundle { invocation_id: String },
}

async fn check_one(client: &GraphqlClient, args: &CheckArgs, case: &Case) -> anyhow::Result<()> {
    // 1) MCP tool path = shared fetch used by tools (raw GraphQL json string).
    let mcp = match &case.kind {
        CaseKind::IssueBundle { fingerprint } => {
            gql::fetch_bundle(client, Some(fingerprint), None).await?
        }
        CaseKind::RunBundle { invocation_id } => {
            gql::fetch_bundle(client, None, Some(invocation_id)).await?
        }
    };

    // 2) Plain HTTP GraphQL (same client — second call; proves stability).
    let http = match &case.kind {
        CaseKind::IssueBundle { fingerprint } => {
            gql::fetch_bundle(client, Some(fingerprint), None).await?
        }
        CaseKind::RunBundle { invocation_id } => {
            gql::fetch_bundle(client, None, Some(invocation_id)).await?
        }
    };

    // 3) CLI: `parallax issue context` / `parallax invocation bundle` --format json
    // Same maxTokens as MCP fetch_bundle so the three projections bound equally.
    let max_tokens = MCP_BUNDLE_MAX_TOKENS.to_string();
    let cli_json = match &case.kind {
        CaseKind::IssueBundle { fingerprint } => run_cli_json(
            &args.parallax_bin,
            &[
                "issue",
                "context",
                fingerprint,
                "--format",
                "json",
                "--max-tokens",
                &max_tokens,
            ],
        )?,
        CaseKind::RunBundle { invocation_id } => run_cli_json(
            &args.parallax_bin,
            &[
                "invocation",
                "bundle",
                invocation_id,
                "--format",
                "json",
                "--max-tokens",
                &max_tokens,
            ],
        )?,
    };

    // Byte-identical JSON strings (strip trailing newline from CLI stdout).
    let cli_trim = cli_json.trim_end_matches('\n');
    let mcp_json = mcp.json.as_str();
    let http_json = http.json.as_str();

    if mcp_json != http_json {
        return Err(diff_err("MCP", mcp_json, "HTTP", http_json));
    }
    if mcp_json != cli_trim {
        return Err(diff_err("MCP", mcp_json, "CLI", cli_trim));
    }
    if http.canonical_hash != mcp.canonical_hash {
        anyhow::bail!(
            "canonicalHash diverged: MCP={} HTTP={}",
            mcp.canonical_hash,
            http.canonical_hash
        );
    }

    // Confirm hash definition: sha256 over sorted-key compact form of evidence
    // fields only (see parallax-evidence bundle::canonical_hash). The GraphQL
    // `canonicalHash` is computed server-side and embedded inside the JSON as
    // `canonical_hash`. Recompute from the emitted JSON body.
    let recomputed = recompute_canonical_hash(mcp_json)?;
    let embedded = extract_embedded_hash(mcp_json)?;
    if recomputed != embedded {
        anyhow::bail!(
            "recomputed hash != embedded canonical_hash:\n  recomputed={recomputed}\n  embedded={embedded}"
        );
    }
    if !mcp.canonical_hash.is_empty() && mcp.canonical_hash != embedded {
        anyhow::bail!(
            "GraphQL canonicalHash field != embedded bundle.canonical_hash:\n  field={}\n  embedded={embedded}",
            mcp.canonical_hash
        );
    }

    println!(
        "  hash={}  json_bytes={}  (CLI≡HTTP≡MCP)",
        embedded,
        mcp_json.len()
    );
    Ok(())
}

fn run_cli_json(bin: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to spawn `{bin}`: {e}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "Parallax CLI equivalence command failed ({}); arguments and stderr omitted because they may contain sensitive evidence",
            output.status
        );
    }
    String::from_utf8(output.stdout).context("Parallax CLI emitted non-UTF-8 JSON")
}

fn extract_embedded_hash(json: &str) -> anyhow::Result<String> {
    let value: Value = serde_json::from_str(json)?;
    value
        .get("canonical_hash")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("bundle JSON missing canonical_hash"))
}

/// Reproduce bundle-v2's version-scoped hash over the emitted envelope: drop
/// request/build fields from the envelope and its immutable v1 `data`, then
/// hash canonical JSON with the `sha256-jcs:` prefix.
pub(crate) fn recompute_canonical_hash(json: &str) -> anyhow::Result<String> {
    let mut value: Value = serde_json::from_str(json)?;
    if let Value::Object(map) = &mut value {
        map.remove("canonical_hash");
        map.remove("generator");
        if let Some(Value::Object(data)) = map.get_mut("data") {
            data.remove("canonical_hash");
            data.remove("generator");
            data.remove("bounded");
        }
    }
    let digest = Sha256::digest(canonical(&value).as_bytes());
    Ok(format!(
        "sha256-jcs:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

fn canonical(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap_or_default(),
                        canonical(&map[k])
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        Value::Array(items) => {
            format!(
                "[{}]",
                items.iter().map(canonical).collect::<Vec<_>>().join(",")
            )
        }
        leaf => serde_json::to_string(leaf).unwrap_or_default(),
    }
}

fn diff_err(a_name: &str, a: &str, b_name: &str, b: &str) -> anyhow::Error {
    // Find first differing byte for a tight diagnostic.
    let pos = a
        .bytes()
        .zip(b.bytes())
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| a.len().min(b.len()));
    let a_hash = Sha256::digest(a.as_bytes());
    let b_hash = Sha256::digest(b.as_bytes());
    anyhow::anyhow!(
        "byte mismatch {a_name} vs {b_name} at offset {pos} \
         (len {a_name}={} {b_name}={}; sha256 {a_name}={a_hash:x} {b_name}={b_hash:x})",
        a.len(),
        b.len()
    )
}

#[cfg(test)]
mod tests;
