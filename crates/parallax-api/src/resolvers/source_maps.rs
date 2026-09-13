//! GraphQL source-map artifact store (R1): upload mutation, metadata
//! listing, and per-event frame resolution at issue-detail render.
//!
//! The stored map JSON is private: no GraphQL field exposes it. Clients
//! receive server-resolved frames (`MappedFrame`) only.

use juniper::{FieldResult, graphql_object};
use parallax_analysis::sourcemap;
use parallax_storage::model;
use std::sync::Arc;

use crate::{ApiContext, field_err, internal_field_err, nanos_string, saturate_i32};

/// Upload cap: 8 MiB of map JSON per artifact.
pub(crate) const SOURCE_MAP_MAX_BYTES: usize = 8 * 1024 * 1024;

/// Stored-artifact metadata. Deliberately has no map-content field.
pub(crate) struct SourceMapArtifact {
    record: model::SourceMapRecord,
}

impl From<model::SourceMapRecord> for SourceMapArtifact {
    fn from(record: model::SourceMapRecord) -> Self {
        Self { record }
    }
}

#[graphql_object(context = ApiContext)]
impl SourceMapArtifact {
    fn service(&self) -> &str {
        &self.record.service
    }
    fn version(&self) -> &str {
        &self.record.version
    }
    fn file(&self) -> &str {
        &self.record.file
    }
    fn debug_id(&self) -> Option<&str> {
        self.record.debug_id.as_deref()
    }
    fn uploaded_at_nanos(&self) -> String {
        nanos_string(self.record.uploaded_at_nanos)
    }
    fn map_bytes(&self) -> i32 {
        saturate_i32(self.record.map_bytes)
    }
    fn map_sha256(&self) -> &str {
        &self.record.map_sha256
    }
}

/// One V8 stack frame plus its source-mapped original position, when a
/// stored map resolved it. `resolved` is false for unmapped frames (no
/// artifact, no version on the event, or no segment) — the UI renders the
/// generated position and offers the upload path.
pub(crate) struct MappedFrame {
    raw: String,
    file: String,
    line: u32,
    column: u32,
    resolved: Option<sourcemap::ResolvedPosition>,
}

#[graphql_object(context = ApiContext)]
impl MappedFrame {
    /// The raw `at …` frame line, for display fallback and copy.
    fn raw(&self) -> &str {
        &self.raw
    }
    /// Generated (minified) file.
    fn file(&self) -> &str {
        &self.file
    }
    /// Generated 1-based line.
    fn line(&self) -> i32 {
        i32::try_from(self.line).unwrap_or(i32::MAX)
    }
    /// Generated 0-based column.
    fn column(&self) -> i32 {
        i32::try_from(self.column).unwrap_or(i32::MAX)
    }
    fn resolved(&self) -> bool {
        self.resolved.is_some()
    }
    /// Original source file, when resolved.
    fn source(&self) -> Option<&str> {
        self.resolved.as_ref().map(|hit| hit.source.as_str())
    }
    /// Original 1-based line, when resolved.
    fn source_line(&self) -> Option<i32> {
        self.resolved
            .as_ref()
            .map(|hit| i32::try_from(hit.line).unwrap_or(i32::MAX))
    }
    /// Original 0-based column, when resolved.
    fn source_column(&self) -> Option<i32> {
        self.resolved
            .as_ref()
            .map(|hit| i32::try_from(hit.column).unwrap_or(i32::MAX))
    }
    /// Original symbol name, when the segment carries one.
    fn name(&self) -> Option<&str> {
        self.resolved.as_ref().and_then(|hit| hit.name.as_deref())
    }
}

/// Stored maps for one (service, version) release, newest first — metadata
/// only, never map content.
pub(crate) async fn source_maps(
    context: &ApiContext,
    service: String,
    version: String,
) -> FieldResult<Vec<SourceMapArtifact>> {
    if service.trim().is_empty() || version.trim().is_empty() {
        return Err(field_err("service and version must be non-empty"));
    }
    let records = context
        .metadata
        .source_maps(&service, &version)
        .await
        .map_err(internal_field_err)?;
    Ok(records.into_iter().map(SourceMapArtifact::from).collect())
}

/// Upload (or replace) one source map for a (service, version, file)
/// release artifact. The map must be valid source-map v3: bad uploads fail
/// here, never at frame-resolution time.
pub(crate) async fn source_map_upload(
    context: &ApiContext,
    service: String,
    version: String,
    file: String,
    map: String,
    debug_id: Option<String>,
) -> FieldResult<SourceMapArtifact> {
    if service.trim().is_empty() {
        return Err(field_err("service must be non-empty"));
    }
    if version.trim().is_empty() {
        return Err(field_err("version must be non-empty"));
    }
    if file.trim().is_empty() {
        return Err(field_err("file must be non-empty"));
    }
    if map.len() > SOURCE_MAP_MAX_BYTES {
        return Err(field_err(format!(
            "map exceeds the {}-byte upload cap",
            SOURCE_MAP_MAX_BYTES
        )));
    }
    sourcemap::parse_source_map(&map).map_err(field_err)?;
    let debug_id = debug_id.filter(|id| !id.trim().is_empty());
    let uploaded_at_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(internal_field_err)?
        .as_nanos();
    let record = context
        .metadata
        .source_map_save(&model::SourceMapUpload {
            service: service.trim(),
            version: version.trim(),
            file: file.trim(),
            debug_id: debug_id.as_deref(),
            map_json: &map,
            uploaded_at_nanos,
        })
        .await
        .map_err(internal_field_err)?;
    Ok(SourceMapArtifact::from(record))
}

/// Resolve one event's V8 frames against the stored maps for its
/// (service, version). Parse failures in stored maps are skipped (uploads
/// are validated, so this is defense only) — resolution never fails the
/// issue-detail read.
pub(crate) async fn mapped_frames_for(
    context: &ApiContext,
    event: &model::ErrorEventRow,
) -> FieldResult<Vec<MappedFrame>> {
    let Some(stacktrace) = event.stacktrace.as_deref() else {
        return Ok(Vec::new());
    };
    let parsed = sourcemap::parse_js_frames(stacktrace);
    if parsed.is_empty() {
        return Ok(Vec::new());
    }
    let Some(version) = event.service_version.as_deref() else {
        return Ok(parsed
            .into_iter()
            .map(|frame| MappedFrame {
                raw: frame.raw,
                file: frame.file,
                line: frame.line,
                column: frame.column,
                resolved: None,
            })
            .collect());
    };
    let records = context.source_maps_for(&event.service, version).await?;
    // Small per-release map count: parse once per request and match frames
    // by exact file, then by basename (frames carry full URLs, uploads
    // usually record basenames).
    let mut parsed_maps: Vec<(String, String, Arc<sourcemap::SourceMap>)> = Vec::new();
    for record in records.iter() {
        if let Ok(map) = sourcemap::parse_source_map(&record.map_json) {
            parsed_maps.push((
                record.file.clone(),
                sourcemap::frame_file_key(&record.file).to_string(),
                Arc::new(map),
            ));
        }
    }
    Ok(parsed
        .into_iter()
        .map(|frame| {
            let key = sourcemap::frame_file_key(&frame.file);
            let hit = parsed_maps.iter().find_map(|(file, base, map)| {
                if file == &frame.file || base == key {
                    map.resolve(frame.line, frame.column)
                } else {
                    None
                }
            });
            MappedFrame {
                raw: frame.raw,
                file: frame.file,
                line: frame.line,
                column: frame.column,
                resolved: hit,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests;
