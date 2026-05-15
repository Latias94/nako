use super::*;

#[async_trait::async_trait]
impl JobRepository for SqliteStore {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(job.id.to_string())
        .bind(job.kind.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(job.resource_class)
        .bind(job.library_id.map(|id| id.to_string()))
        .bind(job.source_id.map(|id| id.to_string()))
        .bind(job.input_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(job.id).await
    }

    async fn start_job(&self, id: JobId) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                started_at = COALESCE(started_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                completed_at = NULL,
                error = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                summary_json = ?3,
                error = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Succeeded.as_str())
        .bind(summary_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                error = ?3,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn get_job(&self, id: JobId) -> Result<Option<Job>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json,
                summary_json,
                error,
                queued_at,
                started_at,
                completed_at
            FROM jobs
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_job).transpose()
    }
}

#[async_trait::async_trait]
impl EventOutboxRepository for SqliteStore {
    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord> {
        let subject_kind = event.subject.kind();
        let subject_id = event.subject.id();

        sqlx::query(
            r#"
            INSERT INTO event_outbox (
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(kind, idempotency_key) DO NOTHING
            "#,
        )
        .bind(event.id.to_string())
        .bind(event.kind.as_str())
        .bind(subject_kind)
        .bind(subject_id)
        .bind(event.library_id.map(|id| id.to_string()))
        .bind(event.source_id.map(|id| id.to_string()))
        .bind(&event.idempotency_key)
        .bind(&event.payload_json)
        .bind(OutboxEventStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_outbox_event_by_idempotency_key(event.kind, &event.idempotency_key)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "outbox event was not found after enqueue for key {}",
                    event.idempotency_key
                ),
            })
    }

    async fn get_outbox_event(&self, id: EventId) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                occurred_at,
                updated_at,
                next_attempt_at
            FROM event_outbox
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_outbox_event).transpose()
    }

    async fn find_outbox_event_by_idempotency_key(
        &self,
        kind: DomainEventKind,
        idempotency_key: &str,
    ) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                occurred_at,
                updated_at,
                next_attempt_at
            FROM event_outbox
            WHERE kind = ?1 AND idempotency_key = ?2
            "#,
        )
        .bind(kind.as_str())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_outbox_event).transpose()
    }

    async fn list_outbox_events(&self, page: PageRequest) -> Result<Vec<OutboxEventRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                occurred_at,
                updated_at,
                next_attempt_at
            FROM event_outbox
            ORDER BY occurred_at DESC, id DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_outbox_event).collect()
    }
}

#[async_trait::async_trait]
impl AutomationRepository for SqliteStore {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord> {
        let capabilities_json =
            serde_json::to_string(&provider.capabilities).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO automation_providers (
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                base_url = excluded.base_url,
                secret_env = excluded.secret_env,
                capabilities_json = excluded.capabilities_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(provider.id.to_string())
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(&provider.secret_env)
        .bind(capabilities_json)
        .bind(u64_to_i64(provider.timeout_ms)?)
        .bind(u32_to_i64(provider.max_attempts))
        .bind(provider.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_provider(provider.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "automation provider {} was not found after upsert",
                    provider.id
                ),
            })
    }

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM automation_providers
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_automation_provider).transpose()
    }

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM automation_providers
            WHERE status = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(AutomationProviderStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_provider).collect()
    }

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord> {
        sqlx::query(
            r#"
            INSERT INTO automation_artifacts (
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(artifact.id.to_string())
        .bind(artifact.job_id.to_string())
        .bind(artifact.provider_id.to_string())
        .bind(artifact.capability.as_str())
        .bind(artifact.kind.as_str())
        .bind(artifact.library_id.map(|id| id.to_string()))
        .bind(artifact.item_id.map(|id| id.to_string()))
        .bind(artifact.source_id.map(|id| id.to_string()))
        .bind(artifact.artifact_json)
        .bind(AutomationArtifactStatus::Proposed.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(artifact.id).await
    }

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord> {
        let query = if status == AutomationArtifactStatus::Accepted {
            r#"
            UPDATE automation_artifacts
            SET
                status = ?2,
                accepted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#
        } else {
            r#"
            UPDATE automation_artifacts
            SET
                status = ?2,
                accepted_at = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#
        };

        sqlx::query(query)
            .bind(id.to_string())
            .bind(status.as_str())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(id).await
    }

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status,
                created_at,
                updated_at,
                accepted_at
            FROM automation_artifacts
            WHERE job_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(job_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status,
                created_at,
                updated_at,
                accepted_at
            FROM automation_artifacts
            WHERE item_id = ?1
            ORDER BY created_at DESC, id DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }
}

#[async_trait::async_trait]
impl WebhookRepository for SqliteStore {
    async fn upsert_webhook_endpoint(
        &self,
        endpoint: NewWebhookEndpoint,
    ) -> Result<WebhookEndpointRecord> {
        let event_kinds_json =
            serde_json::to_string(&endpoint.subscribed_event_kinds).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO webhook_endpoints (
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                secret_env = excluded.secret_env,
                subscribed_event_kinds_json = excluded.subscribed_event_kinds_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(endpoint.id.to_string())
        .bind(&endpoint.name)
        .bind(&endpoint.url)
        .bind(&endpoint.secret_env)
        .bind(event_kinds_json)
        .bind(u64_to_i64(endpoint.timeout_ms)?)
        .bind(u32_to_i64(endpoint.max_attempts))
        .bind(endpoint.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_endpoint(endpoint.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "webhook endpoint {} was not found after upsert",
                    endpoint.id
                ),
            })
    }

    async fn get_webhook_endpoint(
        &self,
        id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpointRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM webhook_endpoints
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_webhook_endpoint).transpose()
    }

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM webhook_endpoints
            WHERE status = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(WebhookEndpointStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_webhook_endpoint).collect()
    }

    async fn create_webhook_delivery_attempt(
        &self,
        attempt: NewWebhookDeliveryAttempt,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            INSERT INTO webhook_delivery_attempts (
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(attempt.id.to_string())
        .bind(attempt.endpoint_id.to_string())
        .bind(attempt.event_id.to_string())
        .bind(u32_to_i64(attempt.attempt_number))
        .bind(WebhookDeliveryStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_delivery_attempt_or_not_found(attempt.id)
            .await
    }

    async fn set_webhook_delivery_attempt_result(
        &self,
        id: WebhookDeliveryAttemptId,
        status: WebhookDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            UPDATE webhook_delivery_attempts
            SET
                status = ?2,
                http_status = ?3,
                error = ?4,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                next_retry_at = ?5
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(http_status.map(i64::from))
        .bind(error)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_delivery_attempt_or_not_found(id).await
    }

    async fn list_webhook_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<WebhookDeliveryAttemptRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status,
                http_status,
                error,
                requested_at,
                completed_at,
                next_retry_at
            FROM webhook_delivery_attempts
            WHERE event_id = ?1
            ORDER BY attempt_number ASC, requested_at ASC, id ASC
            "#,
        )
        .bind(event_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_webhook_delivery_attempt)
            .collect()
    }
}

#[async_trait::async_trait]
impl AddonRepository for SqliteStore {
    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord> {
        let granted_scopes_json =
            serde_json::to_string(&addon.granted_scopes).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO addon_registrations (
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                manifest_id = excluded.manifest_id,
                name = excluded.name,
                version = excluded.version,
                protocol_version = excluded.protocol_version,
                base_url = excluded.base_url,
                manifest_json = excluded.manifest_json,
                granted_scopes_json = excluded.granted_scopes_json,
                status = excluded.status,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(addon.id.to_string())
        .bind(&addon.manifest_id)
        .bind(&addon.name)
        .bind(&addon.version)
        .bind(&addon.protocol_version)
        .bind(&addon.base_url)
        .bind(&addon.manifest_json)
        .bind(granted_scopes_json)
        .bind(addon.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_registration(addon.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon registration {} was not found after upsert", addon.id),
            })
    }

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status,
                created_at,
                updated_at
            FROM addon_registrations
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status,
                created_at,
                updated_at
            FROM addon_registrations
            WHERE manifest_id = ?1
            "#,
        )
        .bind(manifest_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"
                SELECT
                    id,
                    manifest_id,
                    name,
                    version,
                    protocol_version,
                    base_url,
                    manifest_json,
                    granted_scopes_json,
                    status,
                    created_at,
                    updated_at
                FROM addon_registrations
                WHERE status = ?1
                ORDER BY created_at ASC, id ASC
                "#,
            )
            .bind(status.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?
        } else {
            sqlx::query(
                r#"
                SELECT
                    id,
                    manifest_id,
                    name,
                    version,
                    protocol_version,
                    base_url,
                    manifest_json,
                    granted_scopes_json,
                    status,
                    created_at,
                    updated_at
                FROM addon_registrations
                ORDER BY created_at ASC, id ASC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?
        };

        rows.into_iter().map(row_to_addon_registration).collect()
    }
}
