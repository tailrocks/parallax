//! Dated evidence claim coverage rows (plans 121/124 residual).
//!
//! Product claims require measured, appendable coverage rows. Wording is
//! stored verbatim so agent projections cannot invent stronger language.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceClaimRow {
    pub domain: String,
    pub claim_key: String,
    pub level: String,
    pub measured_at_nanos: u128,
    pub coverage_numerator: Option<u64>,
    pub coverage_denominator: Option<u64>,
    pub wording: String,
}

impl TursoMetadataStore {
    /// Upsert one claim coverage row keyed by `(domain, claim_key)`.
    pub async fn upsert_evidence_claim(&self, row: &EvidenceClaimRow) -> anyhow::Result<()> {
        if row.domain.trim().is_empty()
            || row.claim_key.trim().is_empty()
            || row.level.trim().is_empty()
            || row.wording.trim().is_empty()
        {
            anyhow::bail!("domain, claim_key, level, and wording are required");
        }
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO evidence_claim_rows
               (domain, claim_key, level, measured_at, coverage_numerator,
                coverage_denominator, wording)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(domain, claim_key) DO UPDATE SET
               level = excluded.level,
               measured_at = excluded.measured_at,
               coverage_numerator = excluded.coverage_numerator,
               coverage_denominator = excluded.coverage_denominator,
               wording = excluded.wording",
            (
                row.domain.as_str(),
                row.claim_key.as_str(),
                row.level.as_str(),
                nanos_to_millis(row.measured_at_nanos),
                row.coverage_numerator
                    .map(|n| i64::try_from(n).unwrap_or(i64::MAX)),
                row.coverage_denominator
                    .map(|n| i64::try_from(n).unwrap_or(i64::MAX)),
                row.wording.as_str(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn evidence_claim(
        &self,
        domain: &str,
        claim_key: &str,
    ) -> anyhow::Result<Option<EvidenceClaimRow>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT domain, claim_key, level, measured_at, coverage_numerator,
                        coverage_denominator, wording
                 FROM evidence_claim_rows
                 WHERE domain = ?1 AND claim_key = ?2",
                (domain, claim_key),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| decode_claim_row(&row))
            .transpose()
    }

    pub async fn count_evidence_claims(&self, domain: Option<&str>) -> anyhow::Result<u64> {
        let conn = self.conn.lock().await;
        let mut rows = if let Some(domain) = domain {
            conn.query(
                "SELECT COUNT(*) FROM evidence_claim_rows WHERE domain = ?1",
                (domain,),
            )
            .await?
        } else {
            conn.query("SELECT COUNT(*) FROM evidence_claim_rows", ())
                .await?
        };
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing count row"))?;
        Ok(u64::try_from(integer(&row, 0)).unwrap_or(0))
    }
}

fn decode_claim_row(row: &turso::Row) -> anyhow::Result<EvidenceClaimRow> {
    Ok(EvidenceClaimRow {
        domain: text(row, 0),
        claim_key: text(row, 1),
        level: text(row, 2),
        measured_at_nanos: millis_to_nanos(integer(row, 3)),
        coverage_numerator: opt_integer(row, 4).and_then(|n| u64::try_from(n).ok()),
        coverage_denominator: opt_integer(row, 5).and_then(|n| u64::try_from(n).ok()),
        wording: text(row, 6),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn claim_rows_upsert_and_count_by_domain() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("store");
        let row = EvidenceClaimRow {
            domain: "ci_evidence".into(),
            claim_key: "rest_backfill_rate_aware".into(),
            level: "fixture_proven".into(),
            measured_at_nanos: 5_000_000,
            coverage_numerator: Some(2),
            coverage_denominator: Some(2),
            wording: "REST backfill is rate-aware and cursor-safe".into(),
        };
        store.upsert_evidence_claim(&row).await.expect("insert");
        store
            .upsert_evidence_claim(&EvidenceClaimRow {
                level: "live_proven".into(),
                measured_at_nanos: 6_000_000,
                ..row.clone()
            })
            .await
            .expect("update");
        let loaded = store
            .evidence_claim("ci_evidence", "rest_backfill_rate_aware")
            .await
            .expect("load")
            .expect("present");
        assert_eq!(loaded.level, "live_proven");
        assert_eq!(
            store
                .count_evidence_claims(Some("ci_evidence"))
                .await
                .expect("count"),
            1
        );
    }
}
