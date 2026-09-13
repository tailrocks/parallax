//! Source-map v3 resolution for minified JS stacks (R1).
//!
//! Pure decoder: parses a source-map JSON document's `mappings` (VLQ +
//! delta-encoded segments) and resolves generated `(line, column)` positions
//! to original source positions. No dependency on the `sourcemap` crate —
//! the v3 mapping format is small enough to own, and owning it keeps the
//! Rust-first core free of a JS-ecosystem dependency.
//!
//! Conventions follow the spec: generated lines/columns and mapping fields
//! are 0-based in the map; stack-trace line numbers are 1-based and are
//! converted on the way in. Resolution is greatest-lower-bound on the
//! generated column within the line, matching Symbolicator/esbuild behavior
//! for positions between segment starts.

use serde_json::Value;

/// One decoded mapping segment: a generated position bound to an original
/// position. 1-field segments (unmapped generated columns) decode to `None`
/// at lookup time and are kept only as column boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Segment {
    generated_column: u32,
    source: u32,
    source_line: u32,
    source_column: u32,
    name: Option<u32>,
}

/// A parsed source map ready for resolution.
#[derive(Debug, Clone)]
pub struct SourceMap {
    sources: Vec<String>,
    names: Vec<String>,
    /// Per generated line (0-based), segments sorted by generated column.
    lines: Vec<Vec<Segment>>,
}

/// A resolved original position. Lines/columns are 1-based/0-based to match
/// stack-trace display (`file:line:col` keeps the V8 column convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPosition {
    pub source: String,
    pub line: u32,
    pub column: u32,
    pub name: Option<String>,
}

/// Parse and validate a source-map v3 JSON document. Rejects non-v3 maps,
/// missing `mappings`, and undecodable VLQ so bad uploads fail at the API
/// boundary instead of poisoning resolution.
pub fn parse_source_map(map_json: &str) -> Result<SourceMap, String> {
    let document: Value = serde_json::from_str(map_json)
        .map_err(|error| format!("invalid source map JSON: {error}"))?;
    let version = document.get("version").and_then(Value::as_u64);
    if version != Some(3) {
        return Err(format!(
            "unsupported source map version: expected 3, got {}",
            version.map_or("missing".to_string(), |v| v.to_string())
        ));
    }
    let sources = string_array(&document, "sources")?;
    if sources.is_empty() {
        return Err("source map has no sources".to_string());
    }
    let names = match document.get("names") {
        None => Vec::new(),
        Some(_) => string_array(&document, "names")?,
    };
    let mappings = document
        .get("mappings")
        .and_then(Value::as_str)
        .ok_or_else(|| "source map has no mappings".to_string())?;
    let mut decoder = MappingsDecoder::default();
    let mut lines = Vec::new();
    for line_text in mappings.split(';') {
        lines.push(decoder.decode_line(line_text)?);
    }
    Ok(SourceMap {
        sources,
        names,
        lines,
    })
}

/// Delta state for the `mappings` field: the generated column resets every
/// line, while source/line/column/name persist across the whole map.
#[derive(Debug, Default)]
struct MappingsDecoder {
    source: i64,
    source_line: i64,
    source_column: i64,
    name: i64,
}

impl MappingsDecoder {
    fn decode_line(&mut self, line_text: &str) -> Result<Vec<Segment>, String> {
        let mut generated_column = 0i64;
        let mut segments = Vec::new();
        if line_text.is_empty() {
            return Ok(segments);
        }
        for segment_text in line_text.split(',') {
            segments.push(self.decode_segment(segment_text, &mut generated_column)?);
        }
        Ok(segments)
    }

    fn decode_segment(
        &mut self,
        segment_text: &str,
        generated_column: &mut i64,
    ) -> Result<Segment, String> {
        let fields = decode_fields(segment_text)?;
        let Some((head, rest)) = fields.split_first() else {
            return Err("empty mapping segment".to_string());
        };
        *generated_column = generated_column
            .checked_add(*head)
            .ok_or_else(|| "mapping overflow".to_string())?;
        let generated = fit(*generated_column, "negative generated column")?;
        if rest.is_empty() {
            // Unmapped generated column: a boundary only.
            return Ok(Segment {
                generated_column: generated,
                source: u32::MAX,
                source_line: 0,
                source_column: 0,
                name: None,
            });
        }
        if rest.len() != 3 && rest.len() != 4 {
            return Err(format!(
                "mapping segment has {} fields, want 1, 4, or 5",
                fields.len()
            ));
        }
        self.source += rest[0];
        self.source_line += rest[1];
        self.source_column += rest[2];
        let name_index = match rest.get(3) {
            None => None,
            Some(delta) => {
                self.name += delta;
                Some(fit(self.name, "negative name index")?)
            }
        };
        Ok(Segment {
            generated_column: generated,
            source: fit(self.source, "negative mapping field")?,
            source_line: fit(self.source_line, "negative mapping field")?,
            source_column: fit(self.source_column, "negative mapping field")?,
            name: name_index,
        })
    }
}

fn fit(value: i64, error: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("{error}: mapping overflow"))
}

fn string_array(document: &Value, key: &str) -> Result<Vec<String>, String> {
    let Some(array) = document.get(key).and_then(Value::as_array) else {
        return Err(format!("source map has no {key}"));
    };
    array
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("source map {key} must be strings"))
        })
        .collect()
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_value(byte: u8) -> Option<u32> {
    BASE64.iter().position(|candidate| *candidate == byte).map(
        #[expect(
            clippy::cast_possible_truncation,
            reason = "position in a 64-entry table always fits u32"
        )]
        |index| index as u32,
    )
}

/// Decode one VLQ value from the front of `chars`, returning the signed
/// value. `chars` is advanced past the consumed characters.
fn decode_vlq(chars: &mut std::str::Chars<'_>) -> Result<i64, String> {
    let mut result: i64 = 0;
    let mut shift = 0u32;
    loop {
        let Some(byte) = chars.next().map(|c| {
            if c.is_ascii() {
                Ok(c as u8)
            } else {
                Err("non-ASCII character in mappings".to_string())
            }
        }) else {
            return Err("truncated VLQ value in mappings".to_string());
        };
        let digit = base64_value(byte?).ok_or_else(|| "invalid base64 in mappings".to_string())?;
        let continued = digit & 32 != 0;
        let value = digit & 31;
        if shift >= 63 && value > 1 {
            return Err("VLQ value overflows i64".to_string());
        }
        result |= i64::from(value) << shift;
        shift += 5;
        if !continued {
            break;
        }
        if shift > 70 {
            return Err("overlong VLQ value in mappings".to_string());
        }
    }
    let negative = result & 1 != 0;
    let magnitude = result >> 1;
    Ok(if negative { -magnitude } else { magnitude })
}

fn decode_fields(text: &str) -> Result<Vec<i64>, String> {
    let mut chars = text.chars();
    let mut fields = Vec::new();
    while chars.clone().next().is_some() {
        fields.push(decode_vlq(&mut chars)?);
    }
    Ok(fields)
}

impl SourceMap {
    /// Resolve a 1-based generated line + 0-based column to the original
    /// position. Returns `None` for unmapped lines/columns (including
    /// 1-field boundary segments and dangling source/name indexes).
    #[must_use]
    pub fn resolve(&self, line_1based: u32, column: u32) -> Option<ResolvedPosition> {
        let index = usize::try_from(line_1based.checked_sub(1)?).ok()?;
        let segments = self.lines.get(index)?;
        // Greatest-lower-bound: the last segment starting at or before the
        // column wins (segments are in ascending generated-column order).
        let mut winner: Option<&Segment> = None;
        for segment in segments {
            if segment.generated_column > column {
                break;
            }
            winner = Some(segment);
        }
        let found = winner?;
        if found.source == u32::MAX {
            return None;
        }
        let source = self.sources.get(usize::try_from(found.source).ok()?)?;
        let name = found
            .name
            .and_then(|n| self.names.get(usize::try_from(n).ok()?).cloned());
        Some(ResolvedPosition {
            source: source.clone(),
            line: found.source_line.saturating_add(1),
            column: found.source_column,
            name,
        })
    }

    /// Original source file names, for upload validation reporting.
    #[must_use]
    pub fn sources(&self) -> &[String] {
        &self.sources
    }
}

/// One V8-style stack frame with a generated position, parsed from raw
/// stacktrace text (`at fn (file:line:col)` / `at file:line:col`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFrame {
    pub raw: String,
    pub function: Option<String>,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Parse the V8 frames out of a stacktrace. Non-V8 lines (python, java,
/// rust, go, bare text) are skipped — source maps only resolve JS.
#[must_use]
pub fn parse_js_frames(stacktrace: &str) -> Vec<ParsedFrame> {
    stacktrace
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let inner = trimmed.strip_prefix("at ")?.trim();
            // `at fn (file:line:col)` or `at file:line:col`.
            let (function, location) = if let Some(open) = inner.rfind(" (")
                && inner.ends_with(')')
            {
                (
                    Some(inner[..open].to_string()),
                    &inner[open + 2..inner.len() - 1],
                )
            } else {
                (None, inner)
            };
            // Split trailing :line:col; the file itself may contain colons
            // (URL scheme/port), so split from the right.
            let mut parts = location.rsplitn(3, ':');
            let col: u32 = parts.next()?.parse().ok()?;
            let line: u32 = parts.next()?.parse().ok()?;
            let file = parts.next()?;
            if file.is_empty() || line == 0 {
                return None;
            }
            Some(ParsedFrame {
                raw: trimmed.to_string(),
                function,
                file: file.to_string(),
                line,
                column: col,
            })
        })
        .collect()
}

/// Basename of a frame file for artifact matching: strips scheme, host,
/// path, and query/fragment so `https://cdn/x/app.min.js?d=1` matches an
/// upload recorded as `app.min.js`.
#[must_use]
pub fn frame_file_key(file: &str) -> &str {
    let no_fragment = file.split(['#', '?']).next().unwrap_or(file);
    no_fragment.rsplit('/').next().unwrap_or(no_fragment)
}

#[cfg(test)]
mod tests;
