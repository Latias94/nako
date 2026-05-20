use super::{SqliteStore, codec::*};
use taru_core::*;
use taru_search::{SearchDocument, SearchHit, SearchIndex, SearchQuery};

#[async_trait::async_trait]
impl SearchIndex for SqliteStore {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let mut projection =
            CatalogSearchProjection::new(document.item_id, document.title, document.body);
        projection.projection_version = document.projection_version;
        projection.aliases = document.aliases;
        projection.browse_facets = document.browse_facets;

        crate::sqlite::catalog::upsert_search_projection_tx(&mut transaction, &projection).await?;
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
            SELECT item_id, title, body, aliases_json, facets_json, facets_text
            FROM search_documents
            ORDER BY title ASC, item_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let needle = query.query.trim().to_lowercase();
        let required_facets = query
            .facet_labels()
            .into_iter()
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
            let aliases_json: String = row_get(&row, "aliases_json")?;
            let aliases: Vec<String> =
                serde_json::from_str(&aliases_json).map_err(database_error)?;
            let facets_text: String = row_get(&row, "facets_text")?;
            let haystack =
                format!("{title} {body} {} {facets_text}", aliases.join(" ")).to_lowercase();

            if !needle.is_empty() && !haystack.contains(&needle) {
                continue;
            }

            let facets_json: String = row_get(&row, "facets_json")?;
            let document_facets = serde_json::from_str::<Vec<String>>(&facets_json)
                .map_err(database_error)?
                .into_iter()
                .map(|facet| facet.to_lowercase())
                .collect::<Vec<_>>();
            if required_facets.iter().any(|facet| {
                !document_facets
                    .iter()
                    .any(|document_facet| document_facet == facet)
            }) {
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
