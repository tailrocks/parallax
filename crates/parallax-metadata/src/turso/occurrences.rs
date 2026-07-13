use super::{IssueOccurrence, nanos_to_millis};

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
            "INSERT INTO issue_occurrences (occurrence_id, fingerprint, observed_at)
             VALUES (?1, ?2, ?3) ON CONFLICT(occurrence_id) DO NOTHING",
            (
                occurrence.occurrence_id.as_ref(),
                occurrence.fingerprint,
                millis,
            ),
        )
        .await?;
    Ok(claimed > 0)
}
