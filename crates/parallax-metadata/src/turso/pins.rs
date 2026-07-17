//! Evidence pins (plan 106): store sanitized immutable bundle-v2 JSON in Turso
//! so agent-facing evidence survives native telemetry TTL.

use super::*;
use sha2::{Digest, Sha256};

/// Soft max for one pin payload (512 KiB). Larger bundles are refused.
pub const EVIDENCE_PIN_MAX_BYTES: usize = 512 * 1024;
pub const EVIDENCE_PIN_PROTECTION_CAP: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidencePinProtection {
    pub generation: String,
    pub active_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidencePinRecord {
    pub pin_id: String,
    pub anchor_kind: String,
    pub anchor_id: String,
    pub schema_version: String,
    pub canonical_hash: String,
    pub bundle_json: String,
    pub byte_len: u64,
    pub pinned_at_nanos: u128,
    pub expires_at_nanos: Option<u128>,
    pub pinned_by: String,
    pub source_state: String,
}

impl TursoMetadataStore {
    /// Deterministic generation of every live pin at `now_nanos`. The extra
    /// row detects cap overflow; incomplete protection snapshots fail closed.
    pub async fn evidence_pin_protection(
        &self,
        now_nanos: u128,
    ) -> anyhow::Result<EvidencePinProtection> {
        let now = nanos_to_millis(now_nanos);
        let limit = i64::try_from(EVIDENCE_PIN_PROTECTION_CAP + 1)
            .map_err(|_| anyhow::anyhow!("pin protection cap is not representable"))?;
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT pin_id, anchor_kind, anchor_id, canonical_hash, expires_at
                 FROM evidence_pins
                 WHERE expires_at IS NULL OR expires_at > ?1
                 ORDER BY pin_id, anchor_kind, anchor_id, canonical_hash
                 LIMIT ?2",
                (now, limit),
            )
            .await?;
        let mut identities = Vec::new();
        while let Some(row) = rows.next().await? {
            identities.push((
                text(&row, 0),
                text(&row, 1),
                text(&row, 2),
                text(&row, 3),
                opt_integer(&row, 4),
            ));
        }
        if identities.len() > EVIDENCE_PIN_PROTECTION_CAP {
            anyhow::bail!(
                "active evidence pins exceed protection cap {}",
                EVIDENCE_PIN_PROTECTION_CAP
            );
        }
        let active_count = identities.len();
        let encoded = serde_json::to_vec(&identities)?;
        Ok(EvidencePinProtection {
            generation: format!("pins:sha256:{:x}", Sha256::digest(encoded)),
            active_count,
        })
    }

    /// Idempotent pin by `(anchor_kind, anchor_id, canonical_hash)`.
    pub async fn evidence_pin_upsert(
        &self,
        pin: &EvidencePinRecord,
    ) -> anyhow::Result<EvidencePinRecord> {
        if pin.bundle_json.len() > EVIDENCE_PIN_MAX_BYTES {
            anyhow::bail!(
                "evidence pin exceeds {} byte bound (got {})",
                EVIDENCE_PIN_MAX_BYTES,
                pin.bundle_json.len()
            );
        }
        if pin.pin_id.is_empty() || pin.anchor_id.is_empty() || pin.canonical_hash.is_empty() {
            anyhow::bail!("evidence pin requires pin_id, anchor_id, and canonical_hash");
        }
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO evidence_pins (
               pin_id, anchor_kind, anchor_id, schema_version, canonical_hash,
               bundle_json, byte_len, pinned_at, expires_at, pinned_by, source_state
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(pin_id) DO UPDATE SET
               bundle_json = excluded.bundle_json,
               byte_len = excluded.byte_len,
               schema_version = excluded.schema_version,
               canonical_hash = excluded.canonical_hash,
               expires_at = excluded.expires_at,
               source_state = excluded.source_state",
            (
                pin.pin_id.as_str(),
                pin.anchor_kind.as_str(),
                pin.anchor_id.as_str(),
                pin.schema_version.as_str(),
                pin.canonical_hash.as_str(),
                pin.bundle_json.as_str(),
                i64::try_from(pin.byte_len).unwrap_or(i64::MAX),
                nanos_to_millis(pin.pinned_at_nanos),
                pin.expires_at_nanos.map(nanos_to_millis),
                pin.pinned_by.as_str(),
                pin.source_state.as_str(),
            ),
        )
        .await?;
        drop(conn);
        self.evidence_pin(&pin.pin_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("evidence pin did not persist"))
    }

    pub async fn evidence_pin(&self, pin_id: &str) -> anyhow::Result<Option<EvidencePinRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT pin_id, anchor_kind, anchor_id, schema_version, canonical_hash,
                        bundle_json, byte_len, pinned_at, expires_at, pinned_by, source_state
                 FROM evidence_pins WHERE pin_id = ?1",
                (pin_id,),
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        Ok(Some(row_to_pin(&row)?))
    }

    pub async fn evidence_pins_for_anchor(
        &self,
        anchor_kind: &str,
        anchor_id: &str,
    ) -> anyhow::Result<Vec<EvidencePinRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT pin_id, anchor_kind, anchor_id, schema_version, canonical_hash,
                        bundle_json, byte_len, pinned_at, expires_at, pinned_by, source_state
                 FROM evidence_pins
                 WHERE anchor_kind = ?1 AND anchor_id = ?2
                 ORDER BY pinned_at DESC",
                (anchor_kind, anchor_id),
            )
            .await?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            out.push(row_to_pin(&row)?);
        }
        Ok(out)
    }

    pub async fn evidence_pin_delete(&self, pin_id: &str) -> anyhow::Result<bool> {
        let deleted = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM evidence_pins WHERE pin_id = ?1", (pin_id,))
            .await?;
        Ok(deleted > 0)
    }
}

fn row_to_pin(row: &turso::Row) -> anyhow::Result<EvidencePinRecord> {
    Ok(EvidencePinRecord {
        pin_id: text(row, 0),
        anchor_kind: text(row, 1),
        anchor_id: text(row, 2),
        schema_version: text(row, 3),
        canonical_hash: text(row, 4),
        bundle_json: text(row, 5),
        byte_len: u64::try_from(integer(row, 6)).unwrap_or(0),
        pinned_at_nanos: millis_to_nanos(integer(row, 7)),
        expires_at_nanos: opt_integer(row, 8).map(millis_to_nanos),
        pinned_by: text(row, 9),
        source_state: text(row, 10),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pin_round_trip_idempotent_and_bounded() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("open");
        let pin = EvidencePinRecord {
            pin_id: "pin-1".into(),
            anchor_kind: "issue".into(),
            anchor_id: "fp-abc".into(),
            schema_version: "bundle-v2".into(),
            canonical_hash: "sha256-jcs:deadbeef".into(),
            bundle_json: r#"{"schema_version":"bundle-v2","data":{}}"#.into(),
            byte_len: 40,
            pinned_at_nanos: 1_000_000,
            expires_at_nanos: Some(2_000_000),
            pinned_by: "local-operator".into(),
            source_state: "present".into(),
        };
        let first = store.evidence_pin_upsert(&pin).await.expect("pin");
        assert_eq!(first.pin_id, "pin-1");
        let second = store.evidence_pin_upsert(&pin).await.expect("re-pin");
        assert_eq!(first, second);
        let listed = store
            .evidence_pins_for_anchor("issue", "fp-abc")
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert!(store.evidence_pin_delete("pin-1").await.expect("delete"));
        assert!(store.evidence_pin("pin-1").await.expect("get").is_none());

        let mut huge = pin.clone();
        huge.pin_id = "pin-huge".into();
        huge.bundle_json = "x".repeat(EVIDENCE_PIN_MAX_BYTES + 1);
        huge.byte_len = huge.bundle_json.len() as u64;
        store
            .evidence_pin_upsert(&huge)
            .await
            .expect_err("must refuse oversize");
    }

    #[tokio::test]
    async fn protection_generation_is_deterministic_and_excludes_expired_boundary() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("open");
        let pin = |id: &str, expires_at_nanos| EvidencePinRecord {
            pin_id: id.into(),
            anchor_kind: "issue".into(),
            anchor_id: format!("fp-{id}"),
            schema_version: "bundle-v2".into(),
            canonical_hash: format!("sha256-jcs:{id}"),
            bundle_json: r#"{"schema_version":"bundle-v2","data":{}}"#.into(),
            byte_len: 40,
            pinned_at_nanos: 1_000_000,
            expires_at_nanos,
            pinned_by: "local-operator".into(),
            source_state: "present".into(),
        };
        store
            .evidence_pin_upsert(&pin("active", None))
            .await
            .expect("active pin");
        store
            .evidence_pin_upsert(&pin("boundary", Some(20_000_000)))
            .await
            .expect("boundary pin");
        let before = store
            .evidence_pin_protection(19_000_000)
            .await
            .expect("before expiry");
        let at_boundary = store
            .evidence_pin_protection(20_000_000)
            .await
            .expect("at expiry");

        assert_eq!(before.active_count, 2);
        assert_eq!(at_boundary.active_count, 1);
        assert_ne!(before.generation, at_boundary.generation);
        assert_eq!(
            at_boundary,
            store
                .evidence_pin_protection(20_000_000)
                .await
                .expect("repeat generation")
        );
        assert!(at_boundary.generation.starts_with("pins:sha256:"));
        assert_ne!(at_boundary.generation, "pins:none");
    }
}
