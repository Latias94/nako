use super::*;

#[async_trait::async_trait]
impl SearchIndex for SqliteStore {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        crate::catalog::upsert_search_projection_tx(
            &mut transaction,
            &CatalogSearchProjection {
                item_id: document.item_id,
                title: document.title,
                body: document.body,
                facets: document.facets,
            },
        )
        .await?;
        transaction.commit().await.map_err(database_error)
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
