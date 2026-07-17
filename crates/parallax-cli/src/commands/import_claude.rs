//! Consent-only Claude Code stream-json import (plan 120).

use parallax_evidence::claude_code::{SOURCE_TOOL, normalize_stream_json};
use parallax_metadata::{
    AgentSessionImportAccept, AgentSessionImportRecord, TursoMetadataStore, payload_sha256_hex,
};
use parallax_server::Config;
use std::path::Path;

pub(crate) async fn import_claude(path: &Path, json: bool) -> anyhow::Result<()> {
    if !path.is_file() {
        anyhow::bail!("import path is not a file: {}", path.display());
    }
    let ndjson = tokio::fs::read_to_string(path).await?;
    let session = normalize_stream_json(&ndjson);
    let canonical = serde_json::to_string(&session)?;
    let payload_hash = payload_sha256_hex(canonical.as_bytes());
    let import_id = format!(
        "claude_code:{}:{}",
        session.session_id.as_deref().unwrap_or("unknown"),
        &payload_hash[..16]
    );
    let config = Config::load(None).unwrap_or_default();
    let meta_path = config.data_dir().join("meta.db");
    if let Some(parent) = meta_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let store = TursoMetadataStore::open(&meta_path).await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let record = AgentSessionImportRecord {
        import_id: import_id.clone(),
        source_tool: SOURCE_TOOL.into(),
        source_version: session.source_version.clone(),
        capture_surface: session.capture_surface.clone(),
        session_id: session.session_id.clone(),
        payload_hash,
        action_count: u32::try_from(session.actions.len()).unwrap_or(u32::MAX),
        lossiness: session.lossiness.clone(),
        canonical_json: canonical.clone(),
        imported_at_nanos: now,
    };
    let accept = store.accept_agent_session_import(&record).await?;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "import_id": import_id,
                "accept": match accept {
                    AgentSessionImportAccept::Inserted => "inserted",
                    AgentSessionImportAccept::Duplicate => "duplicate",
                },
                "session": session,
            })
        );
    } else {
        println!(
            "import {} ({}) session={} actions={} lossiness={}",
            import_id,
            match accept {
                AgentSessionImportAccept::Inserted => "inserted",
                AgentSessionImportAccept::Duplicate => "duplicate",
            },
            session.session_id.as_deref().unwrap_or("-"),
            session.actions.len(),
            session.lossiness.len()
        );
    }
    Ok(())
}
