use super::*;

#[async_trait::async_trait]
impl LibraryRepository for SqliteStore {
    async fn upsert_library(&self, library: &Library) -> Result<()> {
        let roots_json = serde_json::to_string(&library.roots).map_err(database_error)?;
        let options_json = serde_json::to_string(&library.options).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO libraries (id, name, roots_json, domain, preset, options_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                roots_json = excluded.roots_json,
                domain = excluded.domain,
                preset = excluded.preset,
                options_json = excluded.options_json,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(library.id.to_string())
        .bind(&library.name)
        .bind(roots_json)
        .bind(library.options.domain.as_str())
        .bind(library.options.preset.as_str())
        .bind(options_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, roots_json, domain, preset, options_json
            FROM libraries
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id = parse_id(row_get::<String>(&row, "id")?)?;
        let roots_json = row_get::<String>(&row, "roots_json")?;
        let roots = serde_json::from_str(&roots_json).map_err(database_error)?;
        let options = row_to_library_options(&row)?;

        Ok(Some(Library {
            id,
            name: row_get(&row, "name")?,
            roots,
            options,
        }))
    }

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, roots_json, domain, preset, options_json
            FROM libraries
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_library).collect()
    }
}
