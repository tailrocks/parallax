//! Bounded Sentry envelope framing parser (plan 118, first slice).
//!
//! Implements the serialization format from
//! <https://develop.sentry.dev/sdk/foundations/envelopes/> with Parallax's
//! contract limits from
//! [`docs/research/decisions/sentry-envelope-adapter.md`](../../../../docs/research/decisions/sentry-envelope-adapter.md).
//!
//! This module is pure: it does not talk to the spool, GreptimeDB, or Turso.
//! HTTP mapping, project/DSN auth, and error-event normalization land in later
//! slices. Compatibility claims require sanitized real-SDK fixtures; the unit
//! tests below use protocol-correct hand-crafted envelopes only.

use serde_json::Value;

/// Contract limits (decision record, 2026-07-17).
pub const MAX_ENVELOPE_BYTES: usize = 1_048_576;
pub const MAX_HEADER_LINE_BYTES: usize = 8 * 1024;
pub const MAX_ITEMS: usize = 16;
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 768 * 1024;
pub const MAX_ACCEPTED_EVENTS: usize = 1;

/// Parse/accept outcome for an envelope (internal; not agent-visible).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeOutcome {
    /// One accepted event item plus any unsupported side items.
    Accepted {
        event_id: String,
        event_json: Value,
        unsupported_items: Vec<UnsupportedItem>,
    },
    /// Terminal rejection with a stable reason code.
    Rejected { reason: RejectReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedItem {
    pub item_type: String,
    pub payload_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    EmptyInput,
    EnvelopeTooLarge,
    MalformedEnvelopeHeader,
    HeaderLineTooLarge,
    MalformedItemHeader,
    PrematureEof,
    LengthOverflow,
    TrailingGarbageAfterPayload,
    DuplicateEventItem,
    EventPayloadTooLarge,
    EventPayloadNotJson,
    NoEventItem,
    TooManyItems,
}

impl RejectReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyInput => "empty_input",
            Self::EnvelopeTooLarge => "envelope_too_large",
            Self::MalformedEnvelopeHeader => "malformed_envelope_header",
            Self::HeaderLineTooLarge => "header_line_too_large",
            Self::MalformedItemHeader => "malformed_item_header",
            Self::PrematureEof => "premature_eof",
            Self::LengthOverflow => "length_overflow",
            Self::TrailingGarbageAfterPayload => "trailing_garbage_after_payload",
            Self::DuplicateEventItem => "duplicate_event_item",
            Self::EventPayloadTooLarge => "event_payload_too_large",
            Self::EventPayloadNotJson => "event_payload_not_json",
            Self::NoEventItem => "no_event_item",
            Self::TooManyItems => "too_many_items",
        }
    }
}

/// Parse a decompressed Sentry envelope body into an accepted event or a
/// typed rejection. Single-pass; does not retain borrowed input.
#[must_use]
pub fn parse_envelope(bytes: &[u8]) -> EnvelopeOutcome {
    if bytes.is_empty() {
        return reject(RejectReason::EmptyInput);
    }
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return reject(RejectReason::EnvelopeTooLarge);
    }

    let mut cursor = 0usize;
    let envelope_header = match read_header_line(bytes, &mut cursor) {
        Ok(line) => line,
        Err(reason) => return reject(reason),
    };
    let envelope_header_json: Value = match serde_json::from_slice::<Value>(envelope_header) {
        Ok(value) if value.is_object() => value,
        _ => return reject(RejectReason::MalformedEnvelopeHeader),
    };

    let header_event_id = envelope_header_json
        .get("event_id")
        .and_then(Value::as_str)
        .map(normalize_event_id);

    let mut accepted_event: Option<(String, Value)> = None;
    let mut unsupported = Vec::new();
    let mut item_count = 0usize;

    while cursor < bytes.len() {
        // Trailing newline after the last item is allowed.
        if cursor == bytes.len() - 1 && bytes[cursor] == b'\n' {
            break;
        }
        if item_count >= MAX_ITEMS {
            return reject(RejectReason::TooManyItems);
        }

        let item_header_bytes = match read_header_line(bytes, &mut cursor) {
            Ok(line) => line,
            Err(reason) => return reject(reason),
        };
        // Empty trailing line after a final payload newline → done.
        if item_header_bytes.is_empty() {
            break;
        }
        let item_header: Value = match serde_json::from_slice::<Value>(item_header_bytes) {
            Ok(value) if value.is_object() => value,
            _ => return reject(RejectReason::MalformedItemHeader),
        };
        let item_type = item_header
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if item_type.is_empty() {
            return reject(RejectReason::MalformedItemHeader);
        }

        let length = match item_header.get("length") {
            None => None,
            Some(value) => {
                let Some(n) = value.as_u64() else {
                    return reject(RejectReason::MalformedItemHeader);
                };
                match usize::try_from(n) {
                    Ok(len) => Some(len),
                    Err(_) => return reject(RejectReason::LengthOverflow),
                }
            }
        };

        let payload = match read_payload(bytes, &mut cursor, length) {
            Ok(payload) => payload,
            Err(reason) => return reject(reason),
        };
        item_count += 1;

        if item_type == "event" {
            if accepted_event.is_some() {
                return reject(RejectReason::DuplicateEventItem);
            }
            if payload.len() > MAX_EVENT_PAYLOAD_BYTES {
                return reject(RejectReason::EventPayloadTooLarge);
            }
            let event_json: Value = match serde_json::from_slice::<Value>(payload) {
                Ok(value) if value.is_object() => value,
                _ => return reject(RejectReason::EventPayloadNotJson),
            };
            let event_id = event_json
                .get("event_id")
                .and_then(Value::as_str)
                .map(normalize_event_id)
                .or(header_event_id.clone())
                .unwrap_or_default();
            if event_id.is_empty() || !is_32_hex(&event_id) {
                // Missing/invalid event id is still accepted as a parse of the
                // item; project mapping / HTTP layer assigns terminal outcomes.
                // For this pure slice we keep the payload and empty id.
            }
            accepted_event = Some((event_id, event_json));
        } else {
            unsupported.push(UnsupportedItem {
                item_type,
                payload_len: payload.len(),
            });
        }
    }

    match accepted_event {
        Some((event_id, event_json)) => EnvelopeOutcome::Accepted {
            event_id,
            event_json,
            unsupported_items: unsupported,
        },
        None => reject(RejectReason::NoEventItem),
    }
}

fn reject(reason: RejectReason) -> EnvelopeOutcome {
    EnvelopeOutcome::Rejected { reason }
}

fn read_header_line<'a>(bytes: &'a [u8], cursor: &mut usize) -> Result<&'a [u8], RejectReason> {
    if *cursor >= bytes.len() {
        return Err(RejectReason::PrematureEof);
    }
    let start = *cursor;
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
        if i - start > MAX_HEADER_LINE_BYTES {
            return Err(RejectReason::HeaderLineTooLarge);
        }
    }
    let line = &bytes[start..i];
    if i < bytes.len() {
        *cursor = i + 1; // consume '\n'
    } else {
        *cursor = i; // EOF after header line (allowed for empty envelope)
    }
    if line.len() > MAX_HEADER_LINE_BYTES {
        return Err(RejectReason::HeaderLineTooLarge);
    }
    Ok(line)
}

fn read_payload<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    length: Option<usize>,
) -> Result<&'a [u8], RejectReason> {
    match length {
        Some(len) => {
            if len > MAX_ENVELOPE_BYTES {
                return Err(RejectReason::LengthOverflow);
            }
            let end = cursor
                .checked_add(len)
                .ok_or(RejectReason::LengthOverflow)?;
            if end > bytes.len() {
                return Err(RejectReason::PrematureEof);
            }
            let payload = &bytes[*cursor..end];
            *cursor = end;
            // Length-prefixed payloads must end with '\n' or EOF.
            if *cursor < bytes.len() {
                if bytes[*cursor] != b'\n' {
                    return Err(RejectReason::TrailingGarbageAfterPayload);
                }
                *cursor += 1;
            }
            Ok(payload)
        }
        None => {
            // Implicit length: read until next newline or EOF. `\r` before `\n`
            // is part of the payload per the Sentry spec.
            let start = *cursor;
            let mut i = start;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            let payload = &bytes[start..i];
            if i < bytes.len() {
                *cursor = i + 1;
            } else {
                *cursor = i;
            }
            Ok(payload)
        }
    }
}

fn normalize_event_id(raw: &str) -> String {
    raw.chars()
        .filter(|c| *c != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_32_hex(value: &str) -> bool {
    value.len() == 32 && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn env_with_event(event_id: &str, event: &Value) -> Vec<u8> {
        let event_bytes = serde_json::to_vec(event).expect("event json");
        let mut out = Vec::new();
        out.extend_from_slice(format!(r#"{{"event_id":"{event_id}"}}"#).as_bytes());
        out.push(b'\n');
        out.extend_from_slice(
            format!(r#"{{"type":"event","length":{}}}"#, event_bytes.len()).as_bytes(),
        );
        out.push(b'\n');
        out.extend_from_slice(&event_bytes);
        out.push(b'\n');
        out
    }

    #[test]
    fn accepts_length_prefixed_event_and_skips_unsupported_side_item() {
        let event = json!({
            "event_id": "9ec79c33ec9942ab8353589fcb2e04dc",
            "message": "hello world",
            "level": "error",
            "platform": "native"
        });
        let mut bytes = env_with_event("9ec79c33ec9942ab8353589fcb2e04dc", &event);
        // Append an attachment side item (unsupported).
        let attach = b"hello.txt\n";
        bytes.extend_from_slice(
            format!(r#"{{"type":"attachment","length":{}}}"#, attach.len()).as_bytes(),
        );
        bytes.push(b'\n');
        bytes.extend_from_slice(attach);

        match parse_envelope(&bytes) {
            EnvelopeOutcome::Accepted {
                event_id,
                event_json,
                unsupported_items,
            } => {
                assert_eq!(event_id, "9ec79c33ec9942ab8353589fcb2e04dc");
                assert_eq!(event_json["message"], "hello world");
                assert_eq!(unsupported_items.len(), 1);
                assert_eq!(unsupported_items[0].item_type, "attachment");
                assert_eq!(unsupported_items[0].payload_len, attach.len());
            }
            other => panic!("expected accept, got {other:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_event_items() {
        let event = json!({
            "event_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "message": "one"
        });
        let mut bytes = env_with_event("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", &event);
        let second = serde_json::to_vec(&json!({"message":"two"})).unwrap();
        bytes.extend_from_slice(
            format!(r#"{{"type":"event","length":{}}}"#, second.len()).as_bytes(),
        );
        bytes.push(b'\n');
        bytes.extend_from_slice(&second);
        bytes.push(b'\n');
        match parse_envelope(&bytes) {
            EnvelopeOutcome::Rejected {
                reason: RejectReason::DuplicateEventItem,
            } => {}
            other => panic!("expected duplicate reject, got {other:?}"),
        }
    }

    #[test]
    fn rejects_no_event_item() {
        let bytes = br#"{"event_id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}
{"type":"session","length":2}
{}
"#;
        match parse_envelope(bytes) {
            EnvelopeOutcome::Rejected {
                reason: RejectReason::NoEventItem,
            } => {}
            other => panic!("expected no_event, got {other:?}"),
        }
    }

    #[test]
    fn rejects_premature_eof_on_length_prefix() {
        let bytes = br#"{"event_id":"cccccccccccccccccccccccccccccccc"}
{"type":"event","length":100}
short
"#;
        match parse_envelope(bytes) {
            EnvelopeOutcome::Rejected {
                reason: RejectReason::PrematureEof,
            } => {}
            other => panic!("expected premature_eof, got {other:?}"),
        }
    }

    #[test]
    fn rejects_trailing_garbage_after_length_payload() {
        let event = br#"{"message":"x"}"#;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(br#"{"event_id":"dddddddddddddddddddddddddddddddd"}"#);
        bytes.push(b'\n');
        bytes.extend_from_slice(
            format!(r#"{{"type":"event","length":{}}}"#, event.len()).as_bytes(),
        );
        bytes.push(b'\n');
        bytes.extend_from_slice(event);
        bytes.extend_from_slice(b"X"); // not newline
        match parse_envelope(&bytes) {
            EnvelopeOutcome::Rejected {
                reason: RejectReason::TrailingGarbageAfterPayload,
            } => {}
            other => panic!("expected trailing_garbage, got {other:?}"),
        }
    }

    #[test]
    fn accepts_implicit_length_event_terminated_by_newline() {
        let bytes = br#"{"event_id":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}
{"type":"event"}
{"message":"implicit","level":"error"}
"#;
        match parse_envelope(bytes) {
            EnvelopeOutcome::Accepted {
                event_id,
                event_json,
                unsupported_items,
            } => {
                assert_eq!(event_id, "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                assert_eq!(event_json["message"], "implicit");
                assert!(unsupported_items.is_empty());
            }
            other => panic!("expected accept, got {other:?}"),
        }
    }

    #[test]
    fn rejects_envelope_over_contract_limit() {
        let mut huge = vec![b'x'; MAX_ENVELOPE_BYTES + 1];
        huge[0] = b'{';
        huge[1] = b'}';
        match parse_envelope(&huge) {
            EnvelopeOutcome::Rejected {
                reason: RejectReason::EnvelopeTooLarge,
            } => {}
            other => panic!("expected too large, got {other:?}"),
        }
    }

    #[test]
    fn normalizes_dashed_event_id() {
        let event = json!({
            "event_id": "12c2d058-d584-4270-9aa2-eca08bf20986",
            "message": "dashed"
        });
        let bytes = env_with_event("12c2d058-d584-4270-9aa2-eca08bf20986", &event);
        match parse_envelope(&bytes) {
            EnvelopeOutcome::Accepted { event_id, .. } => {
                assert_eq!(event_id, "12c2d058d58442709aa2eca08bf20986");
            }
            other => panic!("expected accept, got {other:?}"),
        }
    }
}
