use super::*;

#[async_trait::async_trait]
impl ScanRepository for SqliteStore {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            INSERT INTO scan_snapshots (id, library_id, root, status)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(id.to_string())
        .bind(library_id.to_string())
        .bind(root)
        .bind(ScanStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            UPDATE scan_snapshots
            SET
                status = ?2,
                error = ?3,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT id, library_id, root, started_at, completed_at, status, error
            FROM scan_snapshots
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_scan_snapshot).transpose()
    }

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO directory_snapshots (
                scan_id, uri, etag, modified_at, child_count
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(scan_id, uri) DO UPDATE SET
                etag = excluded.etag,
                modified_at = excluded.modified_at,
                child_count = excluded.child_count
            "#,
        )
        .bind(snapshot.scan_id.to_string())
        .bind(&snapshot.uri)
        .bind(&snapshot.etag)
        .bind(&snapshot.modified_at)
        .bind(u64_to_i64(snapshot.child_count)?)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT scan_id, uri, etag, modified_at, child_count
            FROM directory_snapshots
            WHERE scan_id = ?1
            ORDER BY uri ASC
            "#,
        )
        .bind(scan_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_directory_snapshot).collect()
    }

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO source_states (
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(library_id, uri) DO UPDATE SET
                source_id = excluded.source_id,
                size_bytes = excluded.size_bytes,
                modified_at = excluded.modified_at,
                etag = excluded.etag,
                fingerprint = excluded.fingerprint,
                last_seen_scan_id = excluded.last_seen_scan_id,
                tombstoned = excluded.tombstoned,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(state.library_id.to_string())
        .bind(state.source_id.map(|id| id.to_string()))
        .bind(&state.uri)
        .bind(optional_u64_to_i64(state.size_bytes)?)
        .bind(&state.modified_at)
        .bind(&state.etag)
        .bind(&state.fingerprint)
        .bind(state.last_seen_scan_id.to_string())
        .bind(bool_to_i64(state.tombstoned))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            FROM source_states
            WHERE library_id = ?1 AND uri = ?2
            "#,
        )
        .bind(library_id.to_string())
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_source_state).transpose()
    }

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            FROM source_states
            WHERE library_id = ?1
            ORDER BY uri ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_source_state).collect()
    }
}
