use super::*;

#[async_trait::async_trait]
impl SearchIndex for SqliteStore {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        let facets_json = serde_json::to_string(&document.facets).map_err(database_error)?;
        let facets_text = document.facets.join(" ");

        sqlx::query(
            r#"
            INSERT INTO search_documents (
                item_id, title, body, facets_json, facets_text
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(item_id) DO UPDATE SET
                title = excluded.title,
                body = excluded.body,
                facets_json = excluded.facets_json,
                facets_text = excluded.facets_text,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(document.item_id.to_string())
        .bind(document.title)
        .bind(document.body)
        .bind(facets_json)
        .bind(facets_text)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn delete(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM search_documents WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, title, body, facets_json, facets_text
            FROM search_documents
            ORDER BY title ASC, item_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let needle = query.query.trim().to_lowercase();
        let required_facets = query
            .facets
            .iter()
            .map(|facet| facet.to_lowercase())
            .collect::<Vec<_>>();
        let offset = query.offset as usize;
        let limit = if query.limit == 0 {
            PageRequest::DEFAULT_LIMIT as usize
        } else {
            query.limit.min(PageRequest::MAX_LIMIT) as usize
        };

        let mut hits = Vec::new();

        for row in rows {
            let title: String = row_get(&row, "title")?;
            let body: String = row_get(&row, "body")?;
            let facets_text: String = row_get(&row, "facets_text")?;
            let haystack = format!("{title} {body} {facets_text}").to_lowercase();

            if !needle.is_empty() && !haystack.contains(&needle) {
                continue;
            }

            let facet_haystack = facets_text.to_lowercase();
            if required_facets
                .iter()
                .any(|facet| !facet_haystack.contains(facet))
            {
                continue;
            }

            let score = if !needle.is_empty() && title.to_lowercase().contains(&needle) {
                1.0
            } else if !needle.is_empty() && body.to_lowercase().contains(&needle) {
                0.7
            } else {
                0.5
            };

            hits.push(SearchHit {
                item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
                score,
            });
        }

        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.item_id.cmp(&right.item_id))
        });

        Ok(hits.into_iter().skip(offset).take(limit).collect())
    }
}
