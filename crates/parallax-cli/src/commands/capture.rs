//! Bounded child-output capture for `invocation start` wrapper mode.
//!
//! The child's stdout/stderr stream to the terminal live (unchanged UX)
//! while a bounded head of each is retained for telemetry. Secrets are
//! redacted from the stored command line at capture, before anything is
//! persisted or exported.

use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Per-stream capture cap: the stored head of stdout/stderr. Bytes past the
/// cap still stream to the terminal; only the stored copy is bounded.
pub(crate) const CAPTURE_MAX_BYTES: usize = 64 * 1024;

/// Bounded head of one child stream: stored text plus bytes omitted past
/// the cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapturedStream {
    pub(crate) text: String,
    pub(crate) truncated_bytes: u64,
}

impl CapturedStream {
    pub(crate) fn empty() -> Self {
        Self {
            text: String::new(),
            truncated_bytes: 0,
        }
    }

    /// Split finished bytes into the stored head plus the truncation count.
    /// Head truncation (not tail): the stored prefix is a stable function of
    /// the stream, so capture → persist → read round-trips deterministically.
    #[cfg(test)]
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let kept = bytes.len().min(CAPTURE_MAX_BYTES);
        Self {
            text: String::from_utf8_lossy(&bytes[..kept]).into_owned(),
            truncated_bytes: (bytes.len() - kept) as u64,
        }
    }

    /// GraphQL `Int` (i32) saturation for the truncation count.
    pub(crate) fn truncated_i32(&self) -> i32 {
        i32::try_from(self.truncated_bytes).unwrap_or(i32::MAX)
    }
}

/// Stream `reader` to `writer` live while retaining the bounded head.
/// Memory stays flat (`CAPTURE_MAX_BYTES` + one chunk) no matter how much
/// the child emits.
pub(crate) async fn tee_bounded<R, W>(reader: R, writer: W) -> io::Result<CapturedStream>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = reader;
    let mut writer = writer;
    let mut kept: Vec<u8> = Vec::new();
    let mut total: u64 = 0;
    let mut chunk = [0u8; 8192];
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        total += read as u64;
        if kept.len() < CAPTURE_MAX_BYTES {
            let room = CAPTURE_MAX_BYTES - kept.len();
            kept.extend_from_slice(&chunk[..read.min(room)]);
        }
        writer.write_all(&chunk[..read]).await?;
    }
    writer.flush().await?;
    Ok(CapturedStream {
        text: String::from_utf8_lossy(&kept).into_owned(),
        truncated_bytes: total.saturating_sub(kept.len() as u64),
    })
}

/// Redact secrets from the stored command line at capture. Display echo and
/// process spawn still use the raw argv; only the persisted/exported copy is
/// cleaned.
pub(crate) fn redact_command_line(command: &[String]) -> String {
    parallax_redaction::sanitize_text(&command.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_stream_round_trips_whole() {
        let captured = CapturedStream::from_bytes(b"ok\n");
        assert_eq!(captured.text, "ok\n");
        assert_eq!(captured.truncated_bytes, 0);
    }

    #[test]
    fn long_stream_keeps_head_and_counts_tail() {
        let bytes = vec![b'x'; CAPTURE_MAX_BYTES + 100];
        let captured = CapturedStream::from_bytes(&bytes);
        assert_eq!(captured.text.len(), CAPTURE_MAX_BYTES);
        assert_eq!(captured.truncated_bytes, 100);
    }

    #[test]
    fn truncation_count_saturates_to_graphql_int() {
        let captured = CapturedStream {
            text: String::new(),
            truncated_bytes: i32::MAX as u64 + 1,
        };
        assert_eq!(captured.truncated_i32(), i32::MAX);
    }

    #[test]
    fn command_line_secrets_redacted_at_capture() {
        let command = vec![
            "deploy".to_string(),
            "--token=ghp_abcdefghij1234567890".to_string(),
        ];
        let redacted = redact_command_line(&command);
        assert!(!redacted.contains("ghp_abcdefghij1234567890"), "{redacted}");
        assert!(redacted.contains("REDACTED"), "{redacted}");
        assert!(redacted.starts_with("deploy"), "{redacted}");
    }

    #[tokio::test]
    async fn tee_streams_full_output_but_stores_head() {
        let input = vec![b'y'; CAPTURE_MAX_BYTES + 7];
        let mut passthrough: Vec<u8> = Vec::new();
        let captured = tee_bounded(&input[..], &mut passthrough).await.unwrap();
        assert_eq!(passthrough, input);
        assert_eq!(captured.text.len(), CAPTURE_MAX_BYTES);
        assert_eq!(captured.truncated_bytes, 7);
    }
}
