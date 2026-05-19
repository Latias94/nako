use std::fmt::Display;

use super::*;

#[async_trait::async_trait]
impl CatalogRepository for SqliteStore {
    async fn replace_item_catalog_graph(
        &self,
        item_id: MediaItemId,
        replacement: &CatalogItemGraphReplacement,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        replace_item_catalog_graph_tx(&mut transaction, item_id, replacement).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn commit_item_projection(&self, commit: &CatalogItemProjectionCommit) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        replace_item_catalog_graph_tx(&mut transaction, commit.search.item_id, &commit.graph)
            .await?;
        upsert_search_projection_tx(&mut transaction, &commit.search).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn upsert_search_projection(&self, projection: &CatalogSearchProjection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_search_projection_tx(&mut transaction, projection).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn upsert_person(&self, person: &Person) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_person_tx(&mut transaction, person).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>> {
        let row = sqlx::query("SELECT id, name, sort_name, overview FROM people WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let external_ids = self
            .list_entity_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Person>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT p.id, p.name, p.sort_name, p.overview
            FROM people p
            JOIN person_external_ids e ON e.person_id = p.id
            WHERE e.provider = ?1 AND e.provider_key = ?2 AND e.value = ?3
            ORDER BY p.name ASC, p.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, sort_name, overview
            FROM people
            WHERE name = ?1
            ORDER BY name ASC, id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, sort_name, overview
            FROM people
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut people = Vec::with_capacity(rows.len());

        for row in rows {
            let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_entity_external_ids("person_external_ids", "person_id", id)
                .await?;
            people.push(row_to_person(row, external_ids)?);
        }

        Ok(people)
    }

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_item_credit_tx(&mut transaction, credit).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_credits WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, person_id, role, role_key, character, sort_order
            FROM item_credits
            WHERE item_id = ?1
            ORDER BY COALESCE(sort_order, 2147483647), role ASC, person_id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, person_id, role, role_key, character, sort_order
            FROM item_credits
            WHERE person_id = ?1
            ORDER BY role ASC, COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(person_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                mi.id,
                mi.kind,
                mi.parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json
            FROM media_items mi
            JOIN item_credits ic ON ic.item_id = mi.id
            WHERE ic.person_id = ?1
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(person_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_genre(&self, genre: &Genre) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_genre_tx(&mut transaction, genre).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>> {
        let row = sqlx::query("SELECT id, name, source, source_key FROM genres WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_genre).transpose()
    }

    async fn find_genre_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Genre>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM genres
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_genre).transpose()
    }

    async fn list_genres(&self, page: PageRequest) -> Result<Vec<Genre>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM genres
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_genre).collect()
    }

    async fn upsert_item_genre(&self, item_genre: &ItemGenre) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_item_genre_tx(&mut transaction, item_genre).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_genres WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
        let rows = sqlx::query(
            "SELECT item_id, genre_id FROM item_genres WHERE item_id = ?1 ORDER BY genre_id ASC",
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_genre).collect()
    }

    async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                mi.id,
                mi.kind,
                mi.parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json
            FROM media_items mi
            JOIN item_genres ig ON ig.item_id = mi.id
            WHERE ig.genre_id = ?1
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(genre_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_tag(&self, tag: &Tag) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_tag_tx(&mut transaction, tag).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>> {
        let row = sqlx::query("SELECT id, name, source, source_key FROM tags WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_tag).transpose()
    }

    async fn find_tag_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Tag>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM tags
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_tag).transpose()
    }

    async fn list_tags(&self, page: PageRequest) -> Result<Vec<Tag>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM tags
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_tag).collect()
    }

    async fn upsert_item_tag(&self, item_tag: &ItemTag) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_item_tag_tx(&mut transaction, item_tag).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_tags WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>> {
        let rows = sqlx::query(
            "SELECT item_id, tag_id FROM item_tags WHERE item_id = ?1 ORDER BY tag_id ASC",
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_tag).collect()
    }

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                mi.id,
                mi.kind,
                mi.parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json
            FROM media_items mi
            JOIN item_tags it ON it.item_id = mi.id
            WHERE it.tag_id = ?1
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(tag_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_collection(&self, collection: &Collection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_collection_tx(&mut transaction, collection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        let row = sqlx::query(
            "SELECT id, name, overview, source, source_key FROM collections WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_entity_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn find_collection_by_external_id(
        &self,
        external_id: &ExternalId,
    ) -> Result<Option<Collection>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT c.id, c.name, c.overview, c.source, c.source_key
            FROM collections c
            JOIN collection_external_ids e ON e.collection_id = c.id
            WHERE e.provider = ?1 AND e.provider_key = ?2 AND e.value = ?3
            ORDER BY c.name ASC, c.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn find_collection_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Collection>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, overview, source, source_key
            FROM collections
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, overview, source, source_key
            FROM collections
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut collections = Vec::with_capacity(rows.len());

        for row in rows {
            let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_entity_external_ids("collection_external_ids", "collection_id", id)
                .await?;
            collections.push(row_to_collection(row, external_ids)?);
        }

        Ok(collections)
    }

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_collection_item_tx(&mut transaction, item).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM collection_items WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id, item_id, sort_order
            FROM collection_items
            WHERE item_id = ?1
            ORDER BY COALESCE(sort_order, 2147483647), collection_id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn list_collection_items(
        &self,
        collection_id: CollectionId,
    ) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id, item_id, sort_order
            FROM collection_items
            WHERE collection_id = ?1
            ORDER BY COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(collection_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn upsert_studio(&self, studio: &Studio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_studio_tx(&mut transaction, studio).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>> {
        let row = sqlx::query("SELECT id, name, source, source_key FROM studios WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_entity_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Studio>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT s.id, s.name, s.source, s.source_key
            FROM studios s
            JOIN studio_external_ids e ON e.studio_id = s.id
            WHERE e.provider = ?1 AND e.provider_key = ?2 AND e.value = ?3
            ORDER BY s.name ASC, s.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Studio>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM studios
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM studios
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut studios = Vec::with_capacity(rows.len());

        for row in rows {
            let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_entity_external_ids("studio_external_ids", "studio_id", id)
                .await?;
            studios.push(row_to_studio(row, external_ids)?);
        }

        Ok(studios)
    }

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_item_studio_tx(&mut transaction, item_studio).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_studios WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
        let rows = sqlx::query(
            "SELECT item_id, studio_id FROM item_studios WHERE item_id = ?1 ORDER BY studio_id ASC",
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_studio).collect()
    }

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_image_asset_tx(&mut transaction, image).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            FROM image_assets
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_image_asset).transpose()
    }

    async fn find_image_asset_by_source(
        &self,
        owner: &ImageOwner,
        kind: &ImageKind,
        source_uri: &str,
    ) -> Result<Option<ImageAsset>> {
        let (owner_kind, owner_id) = image_owner_to_parts(owner);
        let (kind, kind_key) = image_kind_to_parts(kind);
        let row = sqlx::query(
            r#"
            SELECT
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            FROM image_assets
            WHERE owner_kind = ?1 AND owner_id = ?2 AND kind = ?3
                AND kind_key = ?4 AND source_uri = ?5
            LIMIT 1
            "#,
        )
        .bind(owner_kind)
        .bind(owner_id)
        .bind(kind)
        .bind(kind_key)
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_image_asset).transpose()
    }

    async fn list_item_images(&self, item_id: MediaItemId) -> Result<Vec<ImageAsset>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            FROM image_assets
            WHERE owner_kind = 'item' AND owner_id = ?1
            ORDER BY selected DESC, kind ASC, id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_image_asset).collect()
    }
}

pub(crate) async fn replace_item_catalog_graph_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item_id: MediaItemId,
    replacement: &CatalogItemGraphReplacement,
) -> Result<()> {
    sqlx::query("DELETE FROM item_credits WHERE item_id = ?1")
        .bind(item_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_genres WHERE item_id = ?1")
        .bind(item_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_tags WHERE item_id = ?1")
        .bind(item_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM collection_items WHERE item_id = ?1")
        .bind(item_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_studios WHERE item_id = ?1")
        .bind(item_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for person in &replacement.people {
        upsert_person_tx(transaction, person).await?;
    }
    for credit in &replacement.credits {
        upsert_item_credit_tx(transaction, credit).await?;
    }
    for genre in &replacement.genres {
        upsert_genre_tx(transaction, genre).await?;
    }
    for item_genre in &replacement.item_genres {
        upsert_item_genre_tx(transaction, item_genre).await?;
    }
    for tag in &replacement.tags {
        upsert_tag_tx(transaction, tag).await?;
    }
    for item_tag in &replacement.item_tags {
        upsert_item_tag_tx(transaction, item_tag).await?;
    }
    for collection in &replacement.collections {
        upsert_collection_tx(transaction, collection).await?;
    }
    for collection_item in &replacement.collection_items {
        upsert_collection_item_tx(transaction, collection_item).await?;
    }
    for studio in &replacement.studios {
        upsert_studio_tx(transaction, studio).await?;
    }
    for item_studio in &replacement.item_studios {
        upsert_item_studio_tx(transaction, item_studio).await?;
    }
    for image in &replacement.images {
        upsert_image_asset_tx(transaction, image).await?;
    }

    Ok(())
}

pub(crate) async fn upsert_search_projection_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    projection: &CatalogSearchProjection,
) -> Result<()> {
    let facets_json = serde_json::to_string(&projection.facets).map_err(database_error)?;
    let facets_text = projection.facets.join(" ");

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
    .bind(projection.item_id.to_string())
    .bind(&projection.title)
    .bind(&projection.body)
    .bind(facets_json)
    .bind(facets_text)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

impl SqliteStore {
    pub(crate) async fn list_entity_external_ids<T>(
        &self,
        table: &str,
        owner_column: &str,
        owner_id: T,
    ) -> Result<Vec<ExternalId>>
    where
        T: Display,
    {
        let query = format!(
            "SELECT provider, provider_key, value FROM {table} WHERE {owner_column} = ?1 ORDER BY provider ASC, provider_key ASC, value ASC"
        );
        let rows = sqlx::query(&query)
            .bind(owner_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(ExternalId {
                    provider: provider_from_parts(
                        row_get(&row, "provider")?,
                        row_get(&row, "provider_key")?,
                    ),
                    value: row_get(&row, "value")?,
                })
            })
            .collect()
    }
}

async fn upsert_person_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    person: &Person,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO people (id, name, sort_name, overview)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            sort_name = excluded.sort_name,
            overview = excluded.overview,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(person.id.to_string())
    .bind(&person.name)
    .bind(&person.sort_name)
    .bind(&person.overview)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM person_external_ids WHERE person_id = ?1")
        .bind(person.id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    insert_external_ids(
        transaction,
        "person_external_ids",
        "person_id",
        person.id,
        &person.external_ids,
    )
    .await
}

async fn upsert_item_credit_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    credit: &ItemCredit,
) -> Result<()> {
    let (role, role_key) = credit_role_to_parts(&credit.role);
    sqlx::query(
        r#"
        INSERT INTO item_credits (
            item_id, person_id, role, role_key, character, sort_order
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(item_id, person_id, role, role_key, character) DO UPDATE SET
            sort_order = excluded.sort_order,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(credit.item_id.to_string())
    .bind(credit.person_id.to_string())
    .bind(role)
    .bind(role_key)
    .bind(credit.character.clone().unwrap_or_default())
    .bind(optional_u32_to_i64(credit.sort_order))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_genre_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    genre: &Genre,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&genre.source);
    sqlx::query(
        r#"
        INSERT INTO genres (id, name, source, source_key)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(genre.id.to_string())
    .bind(&genre.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_item_genre_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item_genre: &ItemGenre,
) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO item_genres (item_id, genre_id) VALUES (?1, ?2)")
        .bind(item_genre.item_id.to_string())
        .bind(item_genre.genre_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn upsert_tag_tx(transaction: &mut sqlx::Transaction<'_, Sqlite>, tag: &Tag) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&tag.source);
    sqlx::query(
        r#"
        INSERT INTO tags (id, name, source, source_key)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(tag.id.to_string())
    .bind(&tag.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_item_tag_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item_tag: &ItemTag,
) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)")
        .bind(item_tag.item_id.to_string())
        .bind(item_tag.tag_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn upsert_collection_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    collection: &Collection,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&collection.source);

    sqlx::query(
        r#"
        INSERT INTO collections (id, name, overview, source, source_key)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            overview = excluded.overview,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(collection.id.to_string())
    .bind(&collection.name)
    .bind(&collection.overview)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM collection_external_ids WHERE collection_id = ?1")
        .bind(collection.id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    insert_external_ids(
        transaction,
        "collection_external_ids",
        "collection_id",
        collection.id,
        &collection.external_ids,
    )
    .await
}

async fn upsert_collection_item_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item: &CollectionItem,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO collection_items (collection_id, item_id, sort_order)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(collection_id, item_id) DO UPDATE SET
            sort_order = excluded.sort_order
        "#,
    )
    .bind(item.collection_id.to_string())
    .bind(item.item_id.to_string())
    .bind(optional_u32_to_i64(item.sort_order))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_studio_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    studio: &Studio,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&studio.source);

    sqlx::query(
        r#"
        INSERT INTO studios (id, name, source, source_key)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(studio.id.to_string())
    .bind(&studio.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM studio_external_ids WHERE studio_id = ?1")
        .bind(studio.id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    insert_external_ids(
        transaction,
        "studio_external_ids",
        "studio_id",
        studio.id,
        &studio.external_ids,
    )
    .await
}

async fn upsert_item_studio_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item_studio: &ItemStudio,
) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO item_studios (item_id, studio_id) VALUES (?1, ?2)")
        .bind(item_studio.item_id.to_string())
        .bind(item_studio.studio_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn upsert_image_asset_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    image: &ImageAsset,
) -> Result<()> {
    let (owner_kind, owner_id) = image_owner_to_parts(&image.owner);
    let (kind, kind_key) = image_kind_to_parts(&image.kind);
    let (provider, provider_key) = provider_to_parts(&image.provider);

    sqlx::query(
        r#"
        INSERT INTO image_assets (
            id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
            provider_key, cache_uri, width, height, language, selected,
            content_hash, etag
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(id) DO UPDATE SET
            owner_kind = excluded.owner_kind,
            owner_id = excluded.owner_id,
            kind = excluded.kind,
            kind_key = excluded.kind_key,
            source_uri = excluded.source_uri,
            provider = excluded.provider,
            provider_key = excluded.provider_key,
            cache_uri = excluded.cache_uri,
            width = excluded.width,
            height = excluded.height,
            language = excluded.language,
            selected = excluded.selected,
            content_hash = excluded.content_hash,
            etag = excluded.etag,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        "#,
    )
    .bind(image.id.to_string())
    .bind(owner_kind)
    .bind(owner_id)
    .bind(kind)
    .bind(kind_key)
    .bind(&image.source_uri)
    .bind(provider)
    .bind(provider_key)
    .bind(&image.cache_uri)
    .bind(optional_u32_to_i64(image.width))
    .bind(optional_u32_to_i64(image.height))
    .bind(&image.language)
    .bind(bool_to_i64(image.selected))
    .bind(&image.content_hash)
    .bind(&image.etag)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}
