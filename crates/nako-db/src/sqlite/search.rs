use super::{SqliteStore, codec::*};
use crate::accessible_search::AccessibleSearchIndex;
use nako_core::*;
use nako_search::{
    SearchDocument, SearchEvaluationDocument, SearchHit, SearchIndex, SearchQuery,
    evaluate_search_documents,
};
use sqlx::{QueryBuilder, Sqlite, sqlite::SqliteRow};

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
        let rows = sqlx::query(search_documents_select_sql())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        evaluate_search_rows(query, rows)
    }
}

#[async_trait::async_trait]
impl AccessibleSearchIndex for SqliteStore {
    async fn search_accessible(
        &self,
        principal: &AuthenticatedPrincipal,
        query: SearchQuery,
    ) -> Result<Vec<SearchHit>> {
        let mut sql = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT item_id, projection_version, title, body, aliases_json, facets_json
            FROM search_documents
            WHERE 1 = 1
            "#,
        );
        crate::sqlite::access::push_media_item_access_filter(
            &mut sql,
            principal,
            "search_documents.item_id",
        );
        sql.push(
            r#"
            ORDER BY title ASC, item_id ASC
            "#,
        );

        let rows = sql
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        evaluate_search_rows(query, rows)
    }
}

fn search_documents_select_sql() -> &'static str {
    r#"
            SELECT item_id, projection_version, title, body, aliases_json, facets_json
            FROM search_documents
            ORDER BY title ASC, item_id ASC
            "#
}

fn evaluate_search_rows(query: SearchQuery, rows: Vec<SqliteRow>) -> Result<Vec<SearchHit>> {
    let documents = rows
        .into_iter()
        .map(search_evaluation_document_from_row)
        .collect::<Result<Vec<_>>>()?;

    Ok(evaluate_search_documents(&query, documents))
}

fn search_evaluation_document_from_row(row: SqliteRow) -> Result<SearchEvaluationDocument> {
    let aliases_json: String = row_get(&row, "aliases_json")?;
    let facets_json: String = row_get(&row, "facets_json")?;

    Ok(SearchEvaluationDocument::from_facet_labels(
        parse_id(row_get::<String>(&row, "item_id")?)?,
        i64_to_u16(row_get::<i64>(&row, "projection_version")?)?,
        row_get::<String>(&row, "title")?,
        row_get::<String>(&row, "body")?,
        serde_json::from_str(&aliases_json).map_err(database_error)?,
        serde_json::from_str(&facets_json).map_err(database_error)?,
    ))
}

fn i64_to_u16(value: i64) -> Result<u16> {
    u16::try_from(value).map_err(|err| NakoError::Database {
        message: format!("SQLite integer cannot be converted to u16: {err}"),
    })
}
