//! Claude Code session capture adapter — pure stream-json / hook normalizer
//! (plan 120, first surface).
//!
//! Explicit consent only: this module does not read settings, install hooks,
//! or auto-enable from a checkout. Callers feed NDJSON or hook stdin JSON.

use crate::redaction_policy::sanitize_text;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Tool identity recorded on every normalized session.
pub const SOURCE_TOOL: &str = "claude_code";
/// Capture surface for print-mode stream-json.
pub const CAPTURE_SURFACE_STREAM_JSON: &str = "stream_json";
/// Capture surface for documented hook stdin JSON.
pub const CAPTURE_SURFACE_HOOK_STDIN: &str = "hook_stdin";
/// Claim floor version range (decision record).
pub const VERSION_CLAIM_FLOOR: &str = "2.1.150";

const MAX_LINE_BYTES: usize = 256 * 1024;
const MAX_EVENTS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    SessionStart,
    SessionEnd,
    UserTurn,
    ModelTurn,
    ToolCall,
    ToolResult,
    Permission,
    HookEvent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedAction {
    pub kind: ActionKind,
    pub source_event_type: String,
    pub tool_name: Option<String>,
    pub status: Option<String>,
    /// SHA-256 hex of redacted structural text (never raw prompt by default).
    pub content_sha256: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedSession {
    pub source_tool: String,
    pub source_version: Option<String>,
    pub capture_surface: String,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub cwd: Option<String>,
    pub success: Option<bool>,
    pub duration_ms: Option<u64>,
    pub total_cost_usd_millis: Option<u64>,
    pub actions: Vec<NormalizedAction>,
    pub lossiness: Vec<String>,
    pub event_count: usize,
    pub skipped_oversized_lines: usize,
    pub skipped_malformed_lines: usize,
    pub duplicate_event_count: usize,
    pub conflicting_event_count: usize,
    pub conflicting_session_event_count: usize,
}

/// Parse Claude Code print-mode stream-json NDJSON into one normalized session.
#[must_use]
pub fn normalize_stream_json(ndjson: &str) -> NormalizedSession {
    let mut session = NormalizedSession {
        source_tool: SOURCE_TOOL.into(),
        source_version: None,
        capture_surface: CAPTURE_SURFACE_STREAM_JSON.into(),
        session_id: None,
        model: None,
        cwd: None,
        success: None,
        duration_ms: None,
        total_cost_usd_millis: None,
        actions: Vec::new(),
        lossiness: Vec::new(),
        event_count: 0,
        skipped_oversized_lines: 0,
        skipped_malformed_lines: 0,
        duplicate_event_count: 0,
        conflicting_event_count: 0,
        conflicting_session_event_count: 0,
    };
    let mut explicit_events = HashMap::new();

    for line in ndjson.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.len() > MAX_LINE_BYTES {
            session.skipped_oversized_lines += 1;
            push_loss(&mut session.lossiness, "oversized_line_skipped");
            continue;
        }
        if session.event_count >= MAX_EVENTS {
            push_loss(&mut session.lossiness, "event_cap_reached");
            break;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            session.skipped_malformed_lines += 1;
            push_loss(&mut session.lossiness, "malformed_json_line");
            continue;
        };
        session.event_count += 1;
        if has_conflicting_session(&session, &value) {
            session.conflicting_session_event_count += 1;
            push_loss(&mut session.lossiness, "conflicting_session_event_skipped");
            continue;
        }
        if skip_replayed_explicit_event(&mut session, &mut explicit_events, &value) {
            continue;
        }
        ingest_stream_object(&mut session, &value);
    }

    if session.session_id.is_none() {
        push_loss(&mut session.lossiness, "session_id_missing");
    }
    if !session
        .actions
        .iter()
        .any(|a| a.kind == ActionKind::HookEvent)
    {
        push_loss(&mut session.lossiness, "hook_events_absent");
    }
    push_loss(&mut session.lossiness, "prompt_body_redacted");
    push_loss(&mut session.lossiness, "tool_input_redacted");
    session
}

fn skip_replayed_explicit_event(
    session: &mut NormalizedSession,
    explicit_events: &mut HashMap<String, String>,
    value: &Value,
) -> bool {
    let Some(event_id) = explicit_event_identity(value) else {
        return false;
    };
    let shape = value.as_object().map(structural_hash).unwrap_or_default();
    let Some(previous) = explicit_events.get(&event_id) else {
        explicit_events.insert(event_id, shape);
        return false;
    };
    if previous == &shape {
        session.duplicate_event_count += 1;
        push_loss(&mut session.lossiness, "duplicate_event_skipped");
    } else {
        session.conflicting_event_count += 1;
        push_loss(&mut session.lossiness, "conflicting_event_id_skipped");
    }
    true
}

fn has_conflicting_session(session: &NormalizedSession, value: &Value) -> bool {
    let Some(expected) = session.session_id.as_deref() else {
        return false;
    };
    value
        .get("session_id")
        .and_then(Value::as_str)
        .is_some_and(|observed| observed != expected)
}

fn explicit_event_identity(value: &Value) -> Option<String> {
    if let Some(uuid) = value.get("uuid").and_then(Value::as_str)
        && !uuid.trim().is_empty()
    {
        return Some(format!("uuid:{uuid}"));
    }
    let tool_use_id = value.get("tool_use_id").and_then(Value::as_str)?;
    if tool_use_id.trim().is_empty() {
        return None;
    }
    let hook = value
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Some(format!("hook:{hook}:{tool_use_id}"))
}

/// Normalize one Claude Code hook stdin JSON object.
#[must_use]
pub fn normalize_hook_event(raw: &Value) -> Option<NormalizedAction> {
    let object = raw.as_object()?;
    let event_name = object
        .get("hook_event_name")
        .or_else(|| object.get("hookEventName"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let kind = match event_name {
        "SessionStart" => ActionKind::SessionStart,
        "SessionEnd" | "Stop" | "StopFailure" => ActionKind::SessionEnd,
        "PreToolUse" => ActionKind::ToolCall,
        "PostToolUse" | "PostToolUseFailure" => ActionKind::ToolResult,
        "PermissionRequest" | "PermissionDenied" => ActionKind::Permission,
        "UserPromptSubmit" => ActionKind::UserTurn,
        other if !other.is_empty() => ActionKind::HookEvent,
        _ => ActionKind::Unknown,
    };
    let tool_name = object
        .get("tool_name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let status = match event_name {
        "PostToolUseFailure" | "StopFailure" | "PermissionDenied" => Some("error".into()),
        "PostToolUse" | "Stop" | "SessionEnd" => Some("ok".into()),
        _ => None,
    };
    // Never persist tool_input / prompt text; hash a sanitized structural stub.
    let content_sha256 = structural_hash(object);
    Some(NormalizedAction {
        kind,
        source_event_type: event_name.to_string(),
        tool_name,
        status,
        content_sha256: Some(content_sha256),
        input_tokens: None,
        output_tokens: None,
    })
}

fn ingest_stream_object(session: &mut NormalizedSession, value: &Value) {
    let Some(object) = value.as_object() else {
        push_loss(&mut session.lossiness, "non_object_event");
        return;
    };
    if session.session_id.is_none()
        && let Some(id) = object.get("session_id").and_then(Value::as_str)
    {
        session.session_id = Some(id.to_string());
    }
    let event_type = object
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let subtype = object.get("subtype").and_then(Value::as_str);
    match event_type {
        "system" if subtype == Some("init") => ingest_system_init(session, object),
        "assistant" => ingest_assistant(session, object),
        "user" => ingest_user(session, object),
        "result" => ingest_result(session, object, subtype),
        "hook" => ingest_hook_row(session, value, object),
        other => {
            session.actions.push(NormalizedAction {
                kind: ActionKind::Unknown,
                source_event_type: other.to_string(),
                tool_name: None,
                status: None,
                content_sha256: Some(structural_hash(object)),
                input_tokens: None,
                output_tokens: None,
            });
            push_loss(&mut session.lossiness, "unknown_event_type");
        }
    }
}

fn ingest_system_init(session: &mut NormalizedSession, object: &serde_json::Map<String, Value>) {
    if let Some(model) = object.get("model").and_then(Value::as_str) {
        session.model = Some(model.to_string());
    }
    if let Some(cwd) = object.get("cwd").and_then(Value::as_str) {
        // Paths can leak usernames; keep basename only.
        session.cwd = Some(path_leaf(cwd));
    }
    if let Some(version) = object
        .get("claude_code_version")
        .or_else(|| object.get("version"))
        .and_then(Value::as_str)
    {
        session.source_version = Some(version.to_string());
    }
    session.actions.push(NormalizedAction {
        kind: ActionKind::SessionStart,
        source_event_type: "system.init".into(),
        tool_name: None,
        status: Some("ok".into()),
        content_sha256: None,
        input_tokens: None,
        output_tokens: None,
    });
}

fn ingest_assistant(session: &mut NormalizedSession, object: &serde_json::Map<String, Value>) {
    let (input_tokens, output_tokens) = usage_tokens(object);
    session.actions.push(NormalizedAction {
        kind: ActionKind::ModelTurn,
        source_event_type: "assistant".into(),
        tool_name: None,
        status: None,
        content_sha256: Some(structural_hash(object)),
        input_tokens,
        output_tokens,
    });
}

fn ingest_user(session: &mut NormalizedSession, object: &serde_json::Map<String, Value>) {
    session.actions.push(NormalizedAction {
        kind: ActionKind::UserTurn,
        source_event_type: "user".into(),
        tool_name: None,
        status: None,
        content_sha256: Some(structural_hash(object)),
        input_tokens: None,
        output_tokens: None,
    });
}

fn ingest_result(
    session: &mut NormalizedSession,
    object: &serde_json::Map<String, Value>,
    subtype: Option<&str>,
) {
    let is_error = object
        .get("is_error")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    session.success = Some(!is_error);
    if let Some(ms) = object.get("duration_ms").and_then(Value::as_u64) {
        session.duration_ms = Some(ms);
    }
    if let Some(cost) = object.get("total_cost_usd").and_then(Value::as_f64) {
        session.total_cost_usd_millis = cost_to_millis(cost);
    }
    if is_error {
        push_loss(&mut session.lossiness, "result_is_error");
    }
    let (input_tokens, output_tokens) = usage_tokens(object);
    session.actions.push(NormalizedAction {
        kind: ActionKind::SessionEnd,
        source_event_type: format!("result.{}", subtype.unwrap_or("unknown")),
        tool_name: None,
        status: Some(if is_error { "error" } else { "ok" }.into()),
        content_sha256: None,
        input_tokens,
        output_tokens,
    });
}

fn ingest_hook_row(
    session: &mut NormalizedSession,
    value: &Value,
    object: &serde_json::Map<String, Value>,
) {
    if let Some(action) = normalize_hook_event(value) {
        session.actions.push(action);
        return;
    }
    session.actions.push(NormalizedAction {
        kind: ActionKind::HookEvent,
        source_event_type: "hook".into(),
        tool_name: None,
        status: None,
        content_sha256: Some(structural_hash(object)),
        input_tokens: None,
        output_tokens: None,
    });
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "non-negative finite USD cost scaled to millis"
)]
fn cost_to_millis(cost: f64) -> Option<u64> {
    if cost.is_finite() && cost >= 0.0 {
        Some((cost * 1000.0).round() as u64)
    } else {
        None
    }
}

fn usage_tokens(object: &serde_json::Map<String, Value>) -> (Option<i64>, Option<i64>) {
    let usage = object.get("usage").and_then(Value::as_object);
    let input = usage
        .and_then(|u| u.get("input_tokens"))
        .and_then(Value::as_i64);
    let output = usage
        .and_then(|u| u.get("output_tokens"))
        .and_then(Value::as_i64);
    (input, output)
}

fn structural_hash(object: &serde_json::Map<String, Value>) -> String {
    // Hash only non-body keys so secret-shaped prompt text never enters storage.
    let mut keys: Vec<&String> = object.keys().collect();
    keys.sort();
    let mut hasher = Sha256::new();
    for key in keys {
        if is_body_key(key) {
            continue;
        }
        hasher.update(key.as_bytes());
        hasher.update([0]);
        if let Some(value) = object.get(key) {
            match value {
                Value::String(s) => {
                    let sanitized = sanitize_text(s);
                    hasher.update(sanitized.as_bytes());
                }
                Value::Number(n) => hasher.update(n.to_string().as_bytes()),
                Value::Bool(b) => hasher.update([u8::from(*b)]),
                Value::Null => {}
                other => {
                    // Nested objects/arrays: type tag only, not content.
                    hasher.update(other_type_tag(other).as_bytes());
                }
            }
        }
        hasher.update([0xff]);
    }
    hex_digest(hasher.finalize().as_slice())
}

fn is_body_key(key: &str) -> bool {
    matches!(
        key,
        "message"
            | "result"
            | "content"
            | "tool_input"
            | "tool_response"
            | "prompt"
            | "last_assistant_message"
            | "transcript_path"
    )
}

fn other_type_tag(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        _ => "other",
    }
}

fn path_leaf(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn push_loss(list: &mut Vec<String>, reason: &str) {
    if !list.iter().any(|r| r == reason) {
        list.push(reason.to_string());
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Live-shaped auth-error `result` captured from Claude Code 2.1.212
    /// (`claude -p` without login, 2026-07-17). Sanitized — no credentials.
    const LIVE_AUTH_RESULT: &str = r#"{"type":"result","subtype":"success","is_error":true,"duration_ms":33,"duration_api_ms":0,"num_turns":1,"result":"Not logged in · Please run /login","stop_reason":"stop_sequence","session_id":"4d0c22a3-1893-4287-8b43-b3b2306cdee8","total_cost_usd":0,"usage":{"input_tokens":0,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"output_tokens":0},"uuid":"d60703c3-78c7-4245-9eb7-7991ba1ef783"}"#;

    #[test]
    fn stream_json_live_auth_error_result_normalizes() {
        let session = normalize_stream_json(LIVE_AUTH_RESULT);
        assert_eq!(
            session.session_id.as_deref(),
            Some("4d0c22a3-1893-4287-8b43-b3b2306cdee8")
        );
        assert_eq!(session.success, Some(false));
        assert_eq!(session.duration_ms, Some(33));
        assert_eq!(session.capture_surface, CAPTURE_SURFACE_STREAM_JSON);
        assert!(
            session
                .actions
                .iter()
                .any(|a| a.kind == ActionKind::SessionEnd)
        );
        assert!(session.lossiness.iter().any(|r| r == "result_is_error"));
        assert!(
            session
                .lossiness
                .iter()
                .any(|r| r == "prompt_body_redacted")
        );
        // Body text must not appear in action hashes input path — ensure
        // raw login message is not stored on the session struct fields.
        let encoded = serde_json::to_string(&session).expect("json");
        assert!(
            !encoded.contains("Not logged in"),
            "raw result body leaked into normalized session: {encoded}"
        );
    }

    #[test]
    fn stream_json_multi_event_session() {
        let ndjson = r#"
{"type":"system","subtype":"init","session_id":"sess-1","cwd":"/Users/example/proj","model":"claude-opus","claude_code_version":"2.1.212"}
{"type":"user","session_id":"sess-1","message":{"role":"user","content":"secret prompt sk-ant-api03-FAKE"}}
{"type":"assistant","session_id":"sess-1","usage":{"input_tokens":10,"output_tokens":4},"message":{"role":"assistant","content":"pong"}}
{"type":"result","subtype":"success","is_error":false,"session_id":"sess-1","duration_ms":120,"total_cost_usd":0.001,"usage":{"input_tokens":10,"output_tokens":4}}
"#;
        let session = normalize_stream_json(ndjson);
        assert_eq!(session.session_id.as_deref(), Some("sess-1"));
        assert_eq!(session.model.as_deref(), Some("claude-opus"));
        assert_eq!(session.cwd.as_deref(), Some("proj"));
        assert_eq!(session.source_version.as_deref(), Some("2.1.212"));
        assert_eq!(session.success, Some(true));
        assert_eq!(session.actions.len(), 4);
        assert_eq!(session.actions[0].kind, ActionKind::SessionStart);
        assert_eq!(session.actions[1].kind, ActionKind::UserTurn);
        assert_eq!(session.actions[2].kind, ActionKind::ModelTurn);
        assert_eq!(session.actions[2].input_tokens, Some(10));
        assert_eq!(session.actions[3].kind, ActionKind::SessionEnd);
        let encoded = serde_json::to_string(&session).expect("json");
        assert!(!encoded.contains("sk-ant-api03"));
        assert!(!encoded.contains("secret prompt"));
        assert!(!encoded.contains("/Users/example"));
    }

    #[test]
    fn hook_pre_tool_use_redacts_input() {
        let raw = json!({
            "hook_event_name": "PreToolUse",
            "session_id": "sess-2",
            "tool_name": "Bash",
            "tool_input": {"command": "cat ~/.ssh/id_rsa"}
        });
        let action = normalize_hook_event(&raw).expect("action");
        assert_eq!(action.kind, ActionKind::ToolCall);
        assert_eq!(action.tool_name.as_deref(), Some("Bash"));
        assert!(action.content_sha256.is_some());
        let encoded = serde_json::to_string(&action).expect("json");
        assert!(!encoded.contains("id_rsa"));
        assert!(!encoded.contains("tool_input"));
    }

    #[test]
    fn oversized_line_is_skipped() {
        let huge = format!(
            r#"{{"type":"user","message":"{}"}}"#,
            "x".repeat(MAX_LINE_BYTES)
        );
        let session = normalize_stream_json(&huge);
        assert_eq!(session.skipped_oversized_lines, 1);
        assert!(session.actions.is_empty());
    }

    #[test]
    fn success_path_fixture_normalizes_session_end() {
        // Sanitized success stream-json fixture (version floor 2.1.150).
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/claude_code/success-stream-json.ndjson"
        );
        let ndjson = std::fs::read_to_string(path).expect("fixture");
        let session = normalize_stream_json(&ndjson);
        assert_eq!(session.session_id.as_deref(), Some("sess-success-001"));
        assert_eq!(session.model.as_deref(), Some("claude-opus-4"));
        assert_eq!(session.success, Some(true));
        assert_eq!(session.duration_ms, Some(1500));
        assert!(
            session
                .actions
                .iter()
                .any(|a| a.kind == ActionKind::SessionEnd)
        );
        assert_eq!(session.source_version.as_deref(), Some("2.1.150"));
    }

    #[test]
    fn explicit_ids_make_restart_redelivery_idempotent_and_fail_closed() {
        let first = r#"{"type":"system","subtype":"init","session_id":"sess-a","uuid":"u1"}
{"type":"result","subtype":"success","session_id":"sess-a","uuid":"u2","is_error":false}"#;
        let restarted = format!("{first}\n{first}");
        let session = normalize_stream_json(&restarted);
        assert_eq!(session.actions.len(), 2);
        assert_eq!(session.duplicate_event_count, 2);
        assert_eq!(session.conflicting_event_count, 0);
        assert_eq!(session.success, Some(true));

        let conflict = normalize_stream_json(
            r#"{"type":"result","subtype":"success","session_id":"sess-a","uuid":"same","is_error":false}
{"type":"result","subtype":"success","session_id":"sess-a","uuid":"same","is_error":true}
{"type":"user","session_id":"sess-b","uuid":"other"}"#,
        );
        assert_eq!(conflict.actions.len(), 1);
        assert_eq!(conflict.conflicting_event_count, 1);
        assert_eq!(conflict.conflicting_session_event_count, 1);
        assert_eq!(conflict.success, Some(true));
        assert!(
            conflict
                .lossiness
                .iter()
                .any(|reason| reason == "conflicting_event_id_skipped")
        );
    }
}
