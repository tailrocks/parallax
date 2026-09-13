use std::collections::BTreeMap;

use super::merge_tags;
use super::row::text;
use super::values::BUCKET_MILLIS;
use super::{IssueOccurrence, nanos_to_millis};

pub(super) async fn upsert_issue_occurrences(
    conn: &mut turso::Connection,
    occurrences: &[IssueOccurrence<'_>],
) -> anyhow::Result<()> {
    let tx = conn
        .transaction_with_behavior(turso::transaction::TransactionBehavior::Immediate)
        .await?;
    prune_occurrence_ledger(&tx, occurrences).await?;
    // Issues that received at least one insert, in first-seen order, so
    // tag merge can SELECT once per issue after all inserts.
    let mut tag_order: Vec<(&str, &str)> = Vec::new();
    let mut tag_attrs: BTreeMap<(&str, &str), Vec<&serde_json::Value>> = BTreeMap::new();

    for occurrence in occurrences {
        let millis = nanos_to_millis(occurrence.ts_nanos);
        if !claim_occurrence(&tx, occurrence, millis).await? {
            continue;
        }
        // Plan 111: never persist raw title/culprit — sanitize at the write
        // boundary so every caller (worker, tests, migrations) is covered.
        let safe_title = parallax_redaction::sanitize_text(occurrence.title.as_str());
        let safe_culprit = occurrence
            .culprit
            .as_deref()
            .map(parallax_redaction::sanitize_text);
        tx.execute(
            "INSERT INTO issues
                       (service, fingerprint, title, error_type, culprit,
                        first_seen, last_seen, event_count, last_trace_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 1, ?7)
                     ON CONFLICT(service, fingerprint) DO UPDATE SET
                       title = excluded.title,
                       error_type = excluded.error_type,
                       culprit = COALESCE(excluded.culprit, culprit),
                       first_seen = MIN(first_seen, excluded.first_seen),
                       last_seen = MAX(last_seen, excluded.last_seen),
                       event_count = event_count + 1,
                       last_trace_id = COALESCE(excluded.last_trace_id, last_trace_id),
                       -- Recurrence of a resolved issue is a regression, not a
                       -- silent reopen. Keep resolved_at as last-resolved time.
                       -- (All RHS expressions read the pre-update row.)
                       status = CASE WHEN status = 'resolved' THEN 'regressed' ELSE status END",
            (
                occurrence.service,
                occurrence.fingerprint,
                safe_title.as_str(),
                occurrence.error_type,
                safe_culprit,
                millis,
                occurrence.trace_id.map(str::to_string),
            ),
        )
        .await?;
        tx.execute(
            "INSERT INTO issue_buckets (service, fingerprint, bucket_ts, count)
                 VALUES (?1, ?2, ?3, 1)
                 ON CONFLICT(service, fingerprint, bucket_ts) DO UPDATE SET count = count + 1",
            (
                occurrence.service,
                occurrence.fingerprint,
                millis / BUCKET_MILLIS * BUCKET_MILLIS,
            ),
        )
        .await?;
        let issue_key = (occurrence.service, occurrence.fingerprint);
        if !tag_attrs.contains_key(&issue_key) {
            tag_order.push(issue_key);
        }
        tag_attrs
            .entry(issue_key)
            .or_default()
            .push(occurrence.attributes);
    }

    for (service, fingerprint) in tag_order {
        let attrs = tag_attrs
            .remove(&(service, fingerprint))
            .ok_or_else(|| anyhow::anyhow!("tag attrs missing for ordered issue"))?;
        // Tag cache: read-merge-write under the same connection lock. The
        // SELECT's statement must be dropped before the UPDATE — an UPDATE
        // executed while another statement is open on the same turso
        // connection reports success but does not persist.
        let existing = {
            let mut rows = tx
                .query(
                    "SELECT tags FROM issues WHERE service = ?1 AND fingerprint = ?2",
                    (service, fingerprint),
                )
                .await?;
            rows.next().await?.map(|row| text(&row, 0))
        };
        if let Some(existing) = existing {
            let mut merged = existing;
            for attributes in attrs {
                merged = merge_tags(&merged, attributes);
            }
            tx.execute(
                "UPDATE issues SET tags = ?1 WHERE service = ?2 AND fingerprint = ?3",
                (merged, service, fingerprint),
            )
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

pub(super) const OCCURRENCE_RETENTION_MILLIS: i64 = 30 * 24 * 60 * 60 * 1_000;

pub(super) async fn prune_occurrence_ledger(
    tx: &turso::transaction::Transaction<'_>,
    occurrences: &[IssueOccurrence<'_>],
) -> anyhow::Result<()> {
    let newest = occurrences
        .iter()
        .map(|occurrence| nanos_to_millis(occurrence.ts_nanos))
        .max()
        .unwrap_or(0);
    tx.execute(
        "DELETE FROM issue_occurrences WHERE observed_at < ?1",
        (newest.saturating_sub(OCCURRENCE_RETENTION_MILLIS),),
    )
    .await?;
    Ok(())
}

pub(super) async fn claim_occurrence(
    tx: &turso::transaction::Transaction<'_>,
    occurrence: &IssueOccurrence<'_>,
    millis: i64,
) -> anyhow::Result<bool> {
    let claimed = tx
        .execute(
            "INSERT INTO issue_occurrences (occurrence_id, service, fingerprint, observed_at)
             VALUES (?1, ?2, ?3, ?4) ON CONFLICT(occurrence_id) DO NOTHING",
            (
                occurrence.occurrence_id.as_ref(),
                occurrence.service,
                occurrence.fingerprint,
                millis,
            ),
        )
        .await?;
    Ok(claimed > 0)
}
