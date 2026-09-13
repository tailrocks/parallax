use super::*;
use parallax_model::{SourceMapRecord, SourceMapUpload};

impl TursoMetadataStore {
    pub async fn source_map_save(
        &self,
        upload: &SourceMapUpload<'_>,
    ) -> anyhow::Result<SourceMapRecord> {
        let millis = nanos_to_millis(upload.uploaded_at_nanos);
        let bytes = u64::try_from(upload.map_json.len()).unwrap_or(u64::MAX);
        let sha256 = payload_sha256_hex(upload.map_json.as_bytes());
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO source_maps
                   (service, version, file, debug_id, uploaded_at, map_bytes, map_sha256, map_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(service, version, file) DO UPDATE SET
                   debug_id = excluded.debug_id,
                   uploaded_at = excluded.uploaded_at,
                   map_bytes = excluded.map_bytes,
                   map_sha256 = excluded.map_sha256,
                   map_json = excluded.map_json",
                (
                    upload.service,
                    upload.version,
                    upload.file,
                    upload.debug_id,
                    millis,
                    bytes_to_db(bytes),
                    sha256.as_str(),
                    upload.map_json,
                ),
            )
            .await?;
        Ok(SourceMapRecord {
            service: upload.service.to_string(),
            version: upload.version.to_string(),
            file: upload.file.to_string(),
            debug_id: upload.debug_id.map(str::to_string),
            uploaded_at_nanos: millis_to_nanos(millis),
            map_bytes: bytes,
            map_sha256: sha256,
            map_json: upload.map_json.to_string(),
        })
    }

    pub async fn source_maps(
        &self,
        service: &str,
        version: &str,
    ) -> anyhow::Result<Vec<SourceMapRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT service, version, file, debug_id, uploaded_at, map_bytes, map_sha256, map_json
                 FROM source_maps
                 WHERE service = ?1 AND version = ?2
                 ORDER BY uploaded_at DESC, file ASC
                 LIMIT 500",
                (service, version),
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(Self::source_map_from_row(&row));
        }
        Ok(records)
    }

    pub async fn source_map_releases(&self, service: &str) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT DISTINCT version FROM source_maps
                 WHERE service = ?1 ORDER BY version ASC LIMIT 500",
                (service,),
            )
            .await?;
        let mut versions = Vec::new();
        while let Some(row) = rows.next().await? {
            versions.push(text(&row, 0));
        }
        Ok(versions)
    }

    fn source_map_from_row(row: &turso::Row) -> SourceMapRecord {
        SourceMapRecord {
            service: text(row, 0),
            version: text(row, 1),
            file: text(row, 2),
            debug_id: opt_text(row, 3),
            uploaded_at_nanos: millis_to_nanos(integer(row, 4)),
            map_bytes: bytes_from_db(integer(row, 5)),
            map_sha256: text(row, 6),
            map_json: text(row, 7),
        }
    }
}
