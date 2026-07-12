use super::*;

impl TursoMetadataStore {
    pub async fn dashboard_save(
        &self,
        id: &str,
        name: &str,
        layout: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(now_nanos);
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO dashboards (id, name, layout, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, layout = excluded.layout,
                   updated_at = excluded.updated_at",
                (id, name, layout, millis),
            )
            .await?;
        Ok(())
    }

    pub async fn dashboard_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM dashboards WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn dashboards(&self) -> anyhow::Result<Vec<Dashboard>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, layout, created_at, updated_at
                 FROM dashboards ORDER BY updated_at DESC",
                (),
            )
            .await?;
        let mut dashboards = Vec::new();
        while let Some(row) = rows.next().await? {
            dashboards.push(Self::dashboard_from_row(&row));
        }
        Ok(dashboards)
    }

    fn dashboard_from_row(row: &turso::Row) -> Dashboard {
        Dashboard {
            id: text(row, 0),
            name: text(row, 1),
            layout: text(row, 2),
            created_at_nanos: millis_to_nanos(integer(row, 3)),
            updated_at_nanos: millis_to_nanos(integer(row, 4)),
        }
    }

    pub async fn dashboard(&self, id: &str) -> anyhow::Result<Option<Dashboard>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, layout, created_at, updated_at
                 FROM dashboards WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows.next().await?.map(|row| Self::dashboard_from_row(&row)))
    }

    pub async fn investigation_save(
        &self,
        id: &str,
        name: &str,
        state: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(now_nanos);
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO investigations (id, name, state, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, state = excluded.state,
                   updated_at = excluded.updated_at",
                (id, name, state, millis),
            )
            .await?;
        Ok(())
    }

    pub async fn investigation_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM investigations WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn investigations(&self) -> anyhow::Result<Vec<Investigation>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, state, created_at, updated_at
                 FROM investigations ORDER BY updated_at DESC",
                (),
            )
            .await?;
        let mut investigations = Vec::new();
        while let Some(row) = rows.next().await? {
            investigations.push(Self::investigation_from_row(&row));
        }
        Ok(investigations)
    }

    fn investigation_from_row(row: &turso::Row) -> Investigation {
        Investigation {
            id: text(row, 0),
            name: text(row, 1),
            state: text(row, 2),
            created_at_nanos: millis_to_nanos(integer(row, 3)),
            updated_at_nanos: millis_to_nanos(integer(row, 4)),
        }
    }

    pub async fn investigation(&self, id: &str) -> anyhow::Result<Option<Investigation>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, state, created_at, updated_at
                 FROM investigations WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::investigation_from_row(&row)))
    }

    pub async fn saved_view_save(
        &self,
        id: &str,
        name: &str,
        page: &str,
        state: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(now_nanos);
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO saved_views (id, name, page, state, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, page = excluded.page, state = excluded.state,
                   updated_at = excluded.updated_at",
                (id, name, page, state, millis),
            )
            .await?;
        Ok(())
    }

    pub async fn saved_view_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM saved_views WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn saved_views(&self, page: Option<&str>) -> anyhow::Result<Vec<SavedView>> {
        let conn = self.conn.lock().await;
        let mut saved_views = Vec::new();
        if let Some(page) = page {
            let mut rows = conn
                .query(
                    "SELECT id, name, page, state, created_at, updated_at
                     FROM saved_views WHERE page = ?1 ORDER BY updated_at DESC",
                    (page,),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                saved_views.push(Self::saved_view_from_row(&row));
            }
        } else {
            let mut rows = conn
                .query(
                    "SELECT id, name, page, state, created_at, updated_at
                     FROM saved_views ORDER BY updated_at DESC",
                    (),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                saved_views.push(Self::saved_view_from_row(&row));
            }
        }
        Ok(saved_views)
    }

    fn saved_view_from_row(row: &turso::Row) -> SavedView {
        SavedView {
            id: text(row, 0),
            name: text(row, 1),
            page: text(row, 2),
            state: text(row, 3),
            created_at_nanos: millis_to_nanos(integer(row, 4)),
            updated_at_nanos: millis_to_nanos(integer(row, 5)),
        }
    }

    pub async fn saved_view(&self, id: &str) -> anyhow::Result<Option<SavedView>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, page, state, created_at, updated_at
                 FROM saved_views WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::saved_view_from_row(&row)))
    }
}
