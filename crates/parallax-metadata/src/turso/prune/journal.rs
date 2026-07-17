use super::super::*;
use parallax_storage::{
    PruneJournal, PruneJournalStep, PruneJournalStepState, PrunePlan, PrunePlanLimits,
};

impl TursoMetadataStore {
    pub async fn create_prune_journal(
        &self,
        plan: &PrunePlan,
        now_nanos: u128,
        limits: PrunePlanLimits,
    ) -> anyhow::Result<PruneJournal> {
        let plan_json = serde_json::to_string(plan)?;
        let now = nanos_to_millis(now_nanos);
        let mut conn = self.conn.lock().await;
        let transaction = conn.transaction().await?;
        transaction
            .execute(
                "INSERT OR IGNORE INTO prune_journals
                 (plan_id, plan_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?3)",
                (plan.plan_id(), plan_json.as_str(), now),
            )
            .await?;

        let mut rows = transaction
            .query(
                "SELECT plan_json FROM prune_journals WHERE plan_id = ?1",
                (plan.plan_id(),),
            )
            .await?;
        let stored_plan_json = rows
            .next()
            .await?
            .map(|row| text(&row, 0))
            .ok_or_else(|| anyhow::anyhow!("created prune journal is missing"))?;
        drop(rows);
        if stored_plan_json != plan_json {
            anyhow::bail!("prune journal plan identity collision");
        }

        for (index, item) in plan.items().iter().enumerate() {
            let index = i64::try_from(index).map_err(|_| anyhow::anyhow!("too many steps"))?;
            let item_json = serde_json::to_string(item)?;
            transaction
                .execute(
                    "INSERT OR IGNORE INTO prune_journal_steps
                     (plan_id, step_index, item_json, state)
                     VALUES (?1, ?2, ?3, 'planned')",
                    (plan.plan_id(), index, item_json),
                )
                .await?;
        }
        let mut rows = transaction
            .query(
                "SELECT COUNT(*) FROM prune_journal_steps WHERE plan_id = ?1",
                (plan.plan_id(),),
            )
            .await?;
        let step_count = rows
            .next()
            .await?
            .map(|row| integer(&row, 0))
            .ok_or_else(|| anyhow::anyhow!("prune journal step count is missing"))?;
        drop(rows);
        if usize::try_from(step_count).ok() != Some(plan.items().len()) {
            anyhow::bail!("prune journal step set does not match immutable plan");
        }
        transaction.commit().await?;
        drop(conn);

        self.prune_journal(plan.plan_id(), limits)
            .await?
            .ok_or_else(|| anyhow::anyhow!("committed prune journal is missing"))
    }

    pub async fn prune_journal(
        &self,
        plan_id: &str,
        limits: PrunePlanLimits,
    ) -> anyhow::Result<Option<PruneJournal>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT plan_json, created_at, updated_at, completed_at
                 FROM prune_journals WHERE plan_id = ?1",
                (plan_id,),
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let plan = PrunePlan::decode(&text(&row, 0), limits)?;
        let created_at_nanos = millis_to_nanos(integer(&row, 1));
        let updated_at_nanos = millis_to_nanos(integer(&row, 2));
        let completed_at_nanos = opt_integer(&row, 3).map(millis_to_nanos);
        drop(rows);

        let mut rows = conn
            .query(
                "SELECT step_index, item_json, state, last_error, completed_at
                 FROM prune_journal_steps WHERE plan_id = ?1 ORDER BY step_index",
                (plan_id,),
            )
            .await?;
        let mut steps = Vec::new();
        while let Some(row) = rows.next().await? {
            steps.push(PruneJournalStep {
                step_index: u32::try_from(integer(&row, 0))?,
                item: serde_json::from_str(&text(&row, 1))?,
                state: parse_step_state(&text(&row, 2))?,
                last_error: opt_text(&row, 3),
                completed_at_nanos: opt_integer(&row, 4).map(millis_to_nanos),
            });
        }
        if steps.len() != plan.items().len()
            || steps
                .iter()
                .zip(plan.items())
                .any(|(step, item)| &step.item != item)
        {
            anyhow::bail!("prune journal steps do not match immutable plan");
        }
        Ok(Some(PruneJournal {
            plan,
            created_at_nanos,
            updated_at_nanos,
            completed_at_nanos,
            steps,
        }))
    }
}

fn parse_step_state(value: &str) -> anyhow::Result<PruneJournalStepState> {
    match value {
        "planned" => Ok(PruneJournalStepState::Planned),
        "executing" => Ok(PruneJournalStepState::Executing),
        "complete" => Ok(PruneJournalStepState::Complete),
        other => anyhow::bail!("unknown prune journal step state {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_storage::{PruneClass, PruneEstimate, PruneItem, PruneSnapshot, PruneStore};

    fn plan() -> PrunePlan {
        PrunePlan::build(
            100,
            PruneSnapshot {
                config_generation: "config".into(),
                protection_generation: "pins".into(),
                catalog_fingerprint: "catalog".into(),
            },
            vec![PruneItem {
                store: PruneStore::Turso,
                class: PruneClass::Issues,
                target: "issues".into(),
                cutoff_nanos: 100,
                estimate: PruneEstimate {
                    rows: Some(2),
                    objects: None,
                    bytes: None,
                },
                exclusions: Vec::new(),
                warnings: Vec::new(),
            }],
            PrunePlanLimits::default(),
        )
        .expect("build plan")
    }

    #[tokio::test]
    async fn journal_creation_is_atomic_idempotent_and_restart_safe() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("metadata.db");
        let plan = plan();
        let store = TursoMetadataStore::open(&path)
            .await
            .expect("open metadata");

        let first = store
            .create_prune_journal(&plan, 12_000_000, PrunePlanLimits::default())
            .await
            .expect("create journal");
        let repeated = store
            .create_prune_journal(&plan, 99_000_000, PrunePlanLimits::default())
            .await
            .expect("repeat journal");
        assert_eq!(first, repeated);
        assert_eq!(first.created_at_nanos, 12_000_000);
        assert_eq!(first.steps.len(), 1);
        assert_eq!(first.steps[0].state, PruneJournalStepState::Planned);
        drop(store);

        let reopened = TursoMetadataStore::open(&path)
            .await
            .expect("reopen metadata");
        assert_eq!(
            reopened
                .prune_journal(plan.plan_id(), PrunePlanLimits::default())
                .await
                .expect("recover journal")
                .expect("journal exists"),
            first
        );
    }

    #[tokio::test]
    async fn recovery_rejects_step_bytes_that_diverge_from_the_plan() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let store = TursoMetadataStore::open(directory.path().join("metadata.db"))
            .await
            .expect("open metadata");
        let plan = plan();
        store
            .create_prune_journal(&plan, 1, PrunePlanLimits::default())
            .await
            .expect("create journal");
        store
            .conn
            .lock()
            .await
            .execute(
                "UPDATE prune_journal_steps SET item_json = '{}'
                 WHERE plan_id = ?1 AND step_index = 0",
                (plan.plan_id(),),
            )
            .await
            .expect("tamper step");

        store
            .prune_journal(plan.plan_id(), PrunePlanLimits::default())
            .await
            .expect_err("tampered step must fail recovery");
    }
}
