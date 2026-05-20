CREATE TABLE IF NOT EXISTS libraries (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    roots_json jsonb NOT NULL,
    options_json jsonb NOT NULL,
    domain text NOT NULL,
    preset text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE TABLE IF NOT EXISTS media_items (
    id uuid PRIMARY KEY NOT NULL,
    kind text NOT NULL,
    parent_id uuid REFERENCES media_items(id) ON DELETE SET NULL,
    title text NOT NULL,
    original_title text,
    sort_title text,
    overview text,
    release_date text,
    metadata_json jsonb,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS media_items_parent_id_idx
    ON media_items(parent_id);
CREATE INDEX IF NOT EXISTS media_items_kind_idx
    ON media_items(kind);

CREATE TABLE IF NOT EXISTS media_item_external_ids (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    value text NOT NULL,
    PRIMARY KEY (item_id, provider, provider_key, value)
);

CREATE INDEX IF NOT EXISTS media_item_external_ids_lookup_idx
    ON media_item_external_ids(provider, provider_key, value);

CREATE TABLE IF NOT EXISTS media_sources (
    id uuid PRIMARY KEY NOT NULL,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    locator text NOT NULL,
    file_name text NOT NULL,
    size_bytes bigint,
    fingerprint text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS media_sources_library_locator_idx
    ON media_sources(library_id, locator);
CREATE INDEX IF NOT EXISTS media_sources_library_id_idx
    ON media_sources(library_id);
CREATE INDEX IF NOT EXISTS media_sources_item_id_idx
    ON media_sources(item_id);

CREATE TABLE IF NOT EXISTS library_item_states (
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provisional boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (library_id, item_id)
);

CREATE INDEX IF NOT EXISTS library_item_states_item_id_idx
    ON library_item_states(item_id);
CREATE INDEX IF NOT EXISTS library_item_states_library_provisional_idx
    ON library_item_states(library_id, provisional);

CREATE TABLE IF NOT EXISTS jobs (
    id uuid PRIMARY KEY NOT NULL,
    kind text NOT NULL,
    status text NOT NULL,
    resource_class text NOT NULL,
    library_id uuid REFERENCES libraries(id) ON DELETE SET NULL,
    source_id uuid,
    input_json text,
    summary_json text,
    error text,
    queued_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    started_at timestamptz,
    completed_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    worker_id uuid,
    run_token uuid,
    heartbeat_at timestamptz,
    lease_expires_at timestamptz,
    cancel_requested_at timestamptz,
    cancel_reason text
);

CREATE INDEX IF NOT EXISTS jobs_status_idx ON jobs(status);
CREATE INDEX IF NOT EXISTS jobs_kind_idx ON jobs(kind);
CREATE INDEX IF NOT EXISTS jobs_library_id_idx ON jobs(library_id);
CREATE INDEX IF NOT EXISTS jobs_source_id_idx ON jobs(source_id);
CREATE INDEX IF NOT EXISTS jobs_lease_claim_idx
    ON jobs(status, kind, resource_class, library_id, source_id, queued_at, id);
CREATE INDEX IF NOT EXISTS jobs_lease_expiry_idx
    ON jobs(status, lease_expires_at);

CREATE TABLE IF NOT EXISTS media_source_probes (
    source_id uuid PRIMARY KEY NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    duration_ms bigint,
    container text,
    bit_rate bigint,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE TABLE IF NOT EXISTS media_streams (
    source_id uuid NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    stream_index bigint NOT NULL,
    kind text NOT NULL,
    kind_key text NOT NULL DEFAULT '',
    codec text,
    language text,
    duration_ms bigint,
    bit_rate bigint,
    width bigint,
    height bigint,
    channels bigint,
    sample_rate bigint,
    PRIMARY KEY (source_id, stream_index)
);

CREATE INDEX IF NOT EXISTS media_streams_source_id_idx
    ON media_streams(source_id);
CREATE INDEX IF NOT EXISTS media_streams_kind_idx
    ON media_streams(kind, kind_key);

CREATE TABLE IF NOT EXISTS scan_snapshots (
    id uuid PRIMARY KEY NOT NULL,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    root text NOT NULL,
    started_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    completed_at timestamptz,
    status text NOT NULL,
    error text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS scan_snapshots_library_id_idx
    ON scan_snapshots(library_id);
CREATE INDEX IF NOT EXISTS scan_snapshots_status_idx
    ON scan_snapshots(status);

CREATE TABLE IF NOT EXISTS directory_snapshots (
    scan_id uuid NOT NULL REFERENCES scan_snapshots(id) ON DELETE CASCADE,
    uri text NOT NULL,
    etag text,
    modified_at text,
    child_count bigint NOT NULL,
    PRIMARY KEY (scan_id, uri)
);

CREATE TABLE IF NOT EXISTS source_states (
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_id uuid REFERENCES media_sources(id) ON DELETE SET NULL,
    uri text NOT NULL,
    size_bytes bigint,
    modified_at text,
    etag text,
    fingerprint text,
    last_seen_scan_id uuid NOT NULL REFERENCES scan_snapshots(id) ON DELETE CASCADE,
    tombstoned boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (library_id, uri)
);

CREATE INDEX IF NOT EXISTS source_states_source_id_idx
    ON source_states(source_id);
CREATE INDEX IF NOT EXISTS source_states_fingerprint_idx
    ON source_states(fingerprint);
CREATE INDEX IF NOT EXISTS source_states_scan_id_idx
    ON source_states(last_seen_scan_id);

CREATE TABLE IF NOT EXISTS search_documents (
    item_id uuid PRIMARY KEY NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    projection_version bigint NOT NULL DEFAULT 1,
    title text NOT NULL,
    body text NOT NULL,
    aliases_json jsonb NOT NULL DEFAULT '[]'::jsonb,
    facets_json jsonb NOT NULL,
    facets_text text NOT NULL,
    sort_keys_json jsonb NOT NULL DEFAULT '[]'::jsonb,
    provider_identifiers_json jsonb NOT NULL DEFAULT '[]'::jsonb,
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS search_documents_title_idx
    ON search_documents(title);

CREATE TABLE IF NOT EXISTS local_inference_evidence (
    id uuid PRIMARY KEY NOT NULL,
    source_id uuid NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    inferred_kind text NOT NULL,
    inferred_title text,
    inferred_year bigint,
    inferred_season bigint,
    inferred_episode bigint,
    confidence_milli bigint,
    evidence_source text NOT NULL,
    evidence_source_key text NOT NULL DEFAULT '',
    evidence_value text NOT NULL,
    inference_version text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS local_inference_evidence_source_id_idx
    ON local_inference_evidence(source_id, inference_version);
CREATE UNIQUE INDEX IF NOT EXISTS local_inference_evidence_snapshot_idx
    ON local_inference_evidence(
        source_id,
        evidence_source,
        evidence_source_key,
        inference_version
    );

CREATE TABLE IF NOT EXISTS source_duplicate_relationships (
    id uuid PRIMARY KEY NOT NULL,
    source_id uuid NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    duplicate_source_id uuid NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    evidence_kind text NOT NULL,
    evidence_kind_key text NOT NULL DEFAULT '',
    evidence_value text,
    status text NOT NULL,
    confidence_milli bigint,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (source_id <> duplicate_source_id)
);

CREATE INDEX IF NOT EXISTS source_duplicate_relationships_source_idx
    ON source_duplicate_relationships(source_id, duplicate_source_id, status);
CREATE INDEX IF NOT EXISTS source_duplicate_relationships_duplicate_idx
    ON source_duplicate_relationships(duplicate_source_id, source_id, status);

CREATE TABLE IF NOT EXISTS ingestion_failures (
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    phase text NOT NULL,
    target_uri text NOT NULL,
    target_kind text NOT NULL,
    job_id uuid REFERENCES jobs(id) ON DELETE SET NULL,
    scan_id uuid REFERENCES scan_snapshots(id) ON DELETE SET NULL,
    source_id uuid REFERENCES media_sources(id) ON DELETE SET NULL,
    failure_class text NOT NULL,
    status text NOT NULL,
    message text NOT NULL,
    retryable boolean NOT NULL,
    attempts bigint NOT NULL,
    first_failed_at_ms bigint NOT NULL,
    last_failed_at_ms bigint NOT NULL,
    resolved_at_ms bigint,
    ignored_at_ms bigint,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (library_id, phase, target_uri)
);

CREATE INDEX IF NOT EXISTS ingestion_failures_library_status_idx
    ON ingestion_failures(library_id, status, phase, target_uri);
CREATE INDEX IF NOT EXISTS ingestion_failures_job_idx
    ON ingestion_failures(job_id);
CREATE INDEX IF NOT EXISTS ingestion_failures_scan_idx
    ON ingestion_failures(scan_id);
CREATE INDEX IF NOT EXISTS ingestion_failures_source_idx
    ON ingestion_failures(source_id);

CREATE TABLE IF NOT EXISTS metadata_field_locks (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    field text NOT NULL,
    locked boolean NOT NULL DEFAULT true,
    source text NOT NULL,
    source_key text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (item_id, field)
);

CREATE INDEX IF NOT EXISTS metadata_field_locks_item_id_idx
    ON metadata_field_locks(item_id);

CREATE TABLE IF NOT EXISTS provider_raw_responses (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    body_json text NOT NULL,
    fetched_at text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (item_id, provider, provider_key)
);

CREATE INDEX IF NOT EXISTS provider_raw_responses_item_id_idx
    ON provider_raw_responses(item_id);
CREATE INDEX IF NOT EXISTS provider_raw_responses_lookup_idx
    ON provider_raw_responses(provider, provider_key, item_id);

CREATE TABLE IF NOT EXISTS provider_subjects (
    id uuid PRIMARY KEY NOT NULL,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    subject_kind text NOT NULL,
    subject_kind_key text NOT NULL DEFAULT '',
    subject_key text NOT NULL,
    title text,
    release_year bigint,
    locale text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS provider_subjects_lookup_idx
    ON provider_subjects(provider, provider_key, subject_kind, subject_kind_key, subject_key);
CREATE INDEX IF NOT EXISTS provider_subjects_provider_idx
    ON provider_subjects(provider, provider_key);

CREATE TABLE IF NOT EXISTS provider_mappings (
    id uuid PRIMARY KEY NOT NULL,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    subject_id uuid NOT NULL REFERENCES provider_subjects(id) ON DELETE CASCADE,
    status text NOT NULL,
    confidence_milli bigint,
    source text NOT NULL,
    source_key text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(item_id, subject_id)
);

CREATE INDEX IF NOT EXISTS provider_mappings_item_id_idx
    ON provider_mappings(item_id, status);
CREATE INDEX IF NOT EXISTS provider_mappings_subject_id_idx
    ON provider_mappings(subject_id, status);

CREATE TABLE IF NOT EXISTS metadata_provider_attempts (
    id uuid PRIMARY KEY NOT NULL,
    job_id uuid NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_key text,
    status text NOT NULL,
    matched_by text,
    started_at text NOT NULL,
    finished_at text NOT NULL,
    error_class text,
    message text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS metadata_provider_attempts_job_id_idx
    ON metadata_provider_attempts(job_id, started_at);
CREATE INDEX IF NOT EXISTS metadata_provider_attempts_item_id_idx
    ON metadata_provider_attempts(item_id, started_at);

CREATE TABLE IF NOT EXISTS people (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    sort_name text,
    overview text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS people_name_idx
    ON people(name);

CREATE TABLE IF NOT EXISTS person_external_ids (
    person_id uuid NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    value text NOT NULL,
    PRIMARY KEY (person_id, provider, provider_key, value)
);

CREATE INDEX IF NOT EXISTS person_external_ids_lookup_idx
    ON person_external_ids(provider, provider_key, value);

CREATE TABLE IF NOT EXISTS item_credits (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    person_id uuid NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role text NOT NULL,
    role_key text NOT NULL DEFAULT '',
    character text NOT NULL DEFAULT '',
    sort_order bigint,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (item_id, person_id, role, role_key, character)
);

CREATE INDEX IF NOT EXISTS item_credits_person_id_idx
    ON item_credits(person_id);
CREATE INDEX IF NOT EXISTS item_credits_item_id_idx
    ON item_credits(item_id);

CREATE TABLE IF NOT EXISTS genres (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    source text NOT NULL,
    source_key text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS genres_name_source_idx
    ON genres(name, source, source_key);

CREATE TABLE IF NOT EXISTS item_genres (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    genre_id uuid NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, genre_id)
);

CREATE INDEX IF NOT EXISTS item_genres_genre_id_idx
    ON item_genres(genre_id);

CREATE TABLE IF NOT EXISTS tags (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    source text NOT NULL,
    source_key text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS tags_name_source_idx
    ON tags(name, source, source_key);

CREATE TABLE IF NOT EXISTS item_tags (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    tag_id uuid NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE INDEX IF NOT EXISTS item_tags_tag_id_idx
    ON item_tags(tag_id);

CREATE TABLE IF NOT EXISTS collections (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    overview text,
    source text NOT NULL,
    source_key text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS collections_name_source_idx
    ON collections(name, source, source_key);

CREATE TABLE IF NOT EXISTS collection_external_ids (
    collection_id uuid NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    value text NOT NULL,
    PRIMARY KEY (collection_id, provider, provider_key, value)
);

CREATE INDEX IF NOT EXISTS collection_external_ids_lookup_idx
    ON collection_external_ids(provider, provider_key, value);

CREATE TABLE IF NOT EXISTS collection_items (
    collection_id uuid NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    sort_order bigint,
    PRIMARY KEY (collection_id, item_id)
);

CREATE INDEX IF NOT EXISTS collection_items_item_id_idx
    ON collection_items(item_id);

CREATE TABLE IF NOT EXISTS studios (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    source text NOT NULL,
    source_key text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS studios_name_source_idx
    ON studios(name, source, source_key);

CREATE TABLE IF NOT EXISTS studio_external_ids (
    studio_id uuid NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    value text NOT NULL,
    PRIMARY KEY (studio_id, provider, provider_key, value)
);

CREATE INDEX IF NOT EXISTS studio_external_ids_lookup_idx
    ON studio_external_ids(provider, provider_key, value);

CREATE TABLE IF NOT EXISTS item_studios (
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    studio_id uuid NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, studio_id)
);

CREATE INDEX IF NOT EXISTS item_studios_studio_id_idx
    ON item_studios(studio_id);

CREATE TABLE IF NOT EXISTS image_assets (
    id uuid PRIMARY KEY NOT NULL,
    owner_kind text NOT NULL,
    owner_id uuid NOT NULL,
    kind text NOT NULL,
    kind_key text NOT NULL DEFAULT '',
    source_uri text NOT NULL,
    provider text NOT NULL,
    provider_key text NOT NULL DEFAULT '',
    cache_uri text,
    width bigint,
    height bigint,
    language text,
    selected boolean NOT NULL DEFAULT false,
    content_hash text,
    etag text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS image_assets_source_idx
    ON image_assets(owner_kind, owner_id, kind, kind_key, source_uri);
CREATE INDEX IF NOT EXISTS image_assets_owner_idx
    ON image_assets(owner_kind, owner_id, selected);

CREATE TABLE IF NOT EXISTS user_playback_states (
    principal_id text NOT NULL,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    source_id uuid REFERENCES media_sources(id) ON DELETE SET NULL,
    resume_position_ms bigint,
    duration_ms bigint,
    watched boolean NOT NULL DEFAULT false,
    watched_at_ms bigint,
    last_played_at_ms bigint,
    updated_at_ms bigint NOT NULL,
    version bigint NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (principal_id, item_id),
    CHECK (length(principal_id) > 0),
    CHECK (resume_position_ms IS NULL OR resume_position_ms >= 0),
    CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS user_playback_states_continue_watching_idx
    ON user_playback_states(principal_id, watched, last_played_at_ms DESC, item_id);
CREATE INDEX IF NOT EXISTS user_playback_states_source_id_idx
    ON user_playback_states(source_id);

CREATE TABLE IF NOT EXISTS transcode_sessions (
    id uuid PRIMARY KEY NOT NULL,
    source_id uuid NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    kind text NOT NULL,
    request_key text NOT NULL,
    output_path text NOT NULL,
    state text NOT NULL,
    failure_category text,
    failure_message text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    started_at timestamptz,
    completed_at timestamptz
);

CREATE INDEX IF NOT EXISTS transcode_sessions_source_idx
    ON transcode_sessions(source_id);
CREATE INDEX IF NOT EXISTS transcode_sessions_request_idx
    ON transcode_sessions(source_id, kind, request_key, updated_at);
CREATE INDEX IF NOT EXISTS transcode_sessions_state_idx
    ON transcode_sessions(state);
CREATE UNIQUE INDEX IF NOT EXISTS transcode_sessions_active_request_idx
    ON transcode_sessions(source_id, kind, request_key)
    WHERE state IN ('planned', 'starting', 'running', 'cancel_requested');

CREATE TABLE IF NOT EXISTS event_outbox (
    id uuid PRIMARY KEY NOT NULL,
    kind text NOT NULL,
    subject_kind text NOT NULL,
    subject_id uuid NOT NULL,
    library_id uuid REFERENCES libraries(id) ON DELETE SET NULL,
    source_id uuid REFERENCES media_sources(id) ON DELETE SET NULL,
    idempotency_key text NOT NULL,
    payload_json text NOT NULL,
    status text NOT NULL DEFAULT 'pending',
    attempts bigint NOT NULL DEFAULT 0,
    last_error text,
    occurred_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    next_attempt_at text,
    UNIQUE(kind, idempotency_key)
);

CREATE INDEX IF NOT EXISTS event_outbox_status_idx
    ON event_outbox(status, occurred_at);
CREATE INDEX IF NOT EXISTS event_outbox_subject_idx
    ON event_outbox(subject_kind, subject_id, occurred_at);
CREATE INDEX IF NOT EXISTS event_outbox_library_idx
    ON event_outbox(library_id, occurred_at);
CREATE INDEX IF NOT EXISTS event_outbox_source_idx
    ON event_outbox(source_id, occurred_at);

CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    url text NOT NULL,
    secret_env text,
    subscribed_event_kinds_json jsonb NOT NULL,
    timeout_ms bigint NOT NULL,
    max_attempts bigint NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS webhook_endpoints_status_idx
    ON webhook_endpoints(status);

CREATE TABLE IF NOT EXISTS webhook_delivery_attempts (
    id uuid PRIMARY KEY NOT NULL,
    endpoint_id uuid NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event_id uuid NOT NULL REFERENCES event_outbox(id) ON DELETE CASCADE,
    attempt_number bigint NOT NULL,
    status text NOT NULL,
    http_status bigint,
    error text,
    requested_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    completed_at timestamptz,
    next_retry_at text,
    UNIQUE(endpoint_id, event_id, attempt_number)
);

CREATE INDEX IF NOT EXISTS webhook_delivery_attempts_event_idx
    ON webhook_delivery_attempts(event_id, requested_at);
CREATE INDEX IF NOT EXISTS webhook_delivery_attempts_endpoint_idx
    ON webhook_delivery_attempts(endpoint_id, requested_at);
CREATE INDEX IF NOT EXISTS webhook_delivery_attempts_status_idx
    ON webhook_delivery_attempts(status, next_retry_at);

CREATE TABLE IF NOT EXISTS automation_providers (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    base_url text NOT NULL,
    secret_env text,
    capabilities_json jsonb NOT NULL,
    timeout_ms bigint NOT NULL,
    max_attempts bigint NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS automation_providers_status_idx
    ON automation_providers(status);

CREATE TABLE IF NOT EXISTS automation_artifacts (
    id uuid PRIMARY KEY NOT NULL,
    job_id uuid NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    provider_id uuid NOT NULL REFERENCES automation_providers(id) ON DELETE CASCADE,
    capability text NOT NULL,
    kind text NOT NULL,
    library_id uuid REFERENCES libraries(id) ON DELETE SET NULL,
    item_id uuid REFERENCES media_items(id) ON DELETE SET NULL,
    source_id uuid REFERENCES media_sources(id) ON DELETE SET NULL,
    artifact_json text NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    accepted_at timestamptz
);

CREATE INDEX IF NOT EXISTS automation_artifacts_job_idx
    ON automation_artifacts(job_id, created_at);
CREATE INDEX IF NOT EXISTS automation_artifacts_item_idx
    ON automation_artifacts(item_id, created_at);
CREATE INDEX IF NOT EXISTS automation_artifacts_status_idx
    ON automation_artifacts(status, created_at);

CREATE TABLE IF NOT EXISTS addon_registrations (
    id uuid PRIMARY KEY NOT NULL,
    manifest_id text NOT NULL UNIQUE,
    name text NOT NULL,
    version text NOT NULL,
    protocol_version text NOT NULL,
    base_url text NOT NULL,
    manifest_json text NOT NULL,
    granted_scopes_json jsonb NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS addon_registrations_status_idx
    ON addon_registrations(status, created_at);

CREATE TABLE IF NOT EXISTS addon_tokens (
    id uuid PRIMARY KEY NOT NULL,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    label text NOT NULL,
    token_prefix text NOT NULL,
    token_hash text NOT NULL UNIQUE,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    rotated_at timestamptz,
    revoked_at timestamptz,
    last_used_at timestamptz
);

CREATE INDEX IF NOT EXISTS addon_tokens_addon_status_idx
    ON addon_tokens(addon_id, status, created_at);

CREATE TABLE IF NOT EXISTS addon_grants (
    id uuid PRIMARY KEY NOT NULL,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    permission text NOT NULL,
    library_id uuid REFERENCES libraries(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(addon_id, permission, library_id)
);

CREATE INDEX IF NOT EXISTS addon_grants_addon_idx
    ON addon_grants(addon_id, permission, library_id);
CREATE UNIQUE INDEX IF NOT EXISTS addon_grants_unique_scope_idx
    ON addon_grants(addon_id, permission, COALESCE(library_id::text, ''));

CREATE TABLE IF NOT EXISTS addon_side_effects (
    id uuid PRIMARY KEY NOT NULL,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    token_id uuid NOT NULL REFERENCES addon_tokens(id) ON DELETE CASCADE,
    permission text NOT NULL,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    target_kind text NOT NULL,
    target_id text NOT NULL,
    idempotency_key text NOT NULL,
    provenance_json text NOT NULL,
    payload_json text NOT NULL,
    validation_status text NOT NULL,
    safe_error_code text,
    apply_status text NOT NULL DEFAULT 'pending',
    apply_error_code text,
    applied_item_id uuid REFERENCES media_items(id) ON DELETE SET NULL,
    applied_source text,
    apply_report_json text,
    applied_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(addon_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS addon_side_effects_addon_created_idx
    ON addon_side_effects(addon_id, created_at, id);
CREATE INDEX IF NOT EXISTS addon_side_effects_library_target_idx
    ON addon_side_effects(library_id, target_kind, target_id, created_at);
CREATE INDEX IF NOT EXISTS addon_side_effects_apply_status_idx
    ON addon_side_effects(apply_status, created_at, id);

CREATE TABLE IF NOT EXISTS vfs_cache_objects (
    uri text PRIMARY KEY NOT NULL,
    scheme text NOT NULL,
    kind text NOT NULL,
    len bigint,
    modified_at text,
    etag text,
    fingerprint text,
    capabilities_bits bigint NOT NULL,
    fetched_at_ms bigint NOT NULL,
    fresh_until_ms bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (len IS NULL OR len >= 0),
    CHECK (capabilities_bits >= 0)
);

CREATE INDEX IF NOT EXISTS vfs_cache_objects_scheme_idx
    ON vfs_cache_objects(scheme, fresh_until_ms);

CREATE TABLE IF NOT EXISTS vfs_cache_listings (
    uri text PRIMARY KEY NOT NULL REFERENCES vfs_cache_objects(uri) ON DELETE CASCADE,
    scheme text NOT NULL,
    fetched_at_ms bigint NOT NULL,
    fresh_until_ms bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX IF NOT EXISTS vfs_cache_listings_scheme_idx
    ON vfs_cache_listings(scheme, fresh_until_ms);

CREATE TABLE IF NOT EXISTS vfs_cache_listing_entries (
    listing_uri text NOT NULL REFERENCES vfs_cache_listings(uri) ON DELETE CASCADE,
    entry_uri text NOT NULL REFERENCES vfs_cache_objects(uri) ON DELETE CASCADE,
    sort_order bigint NOT NULL,
    PRIMARY KEY (listing_uri, entry_uri)
);

CREATE INDEX IF NOT EXISTS vfs_cache_listing_entries_entry_idx
    ON vfs_cache_listing_entries(entry_uri);

CREATE TABLE IF NOT EXISTS vfs_cache_failures (
    uri text NOT NULL,
    scheme text NOT NULL,
    operation text NOT NULL,
    failed_at_ms bigint NOT NULL,
    failure_count bigint NOT NULL,
    error text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (uri, operation),
    CHECK (failure_count >= 0)
);

CREATE INDEX IF NOT EXISTS vfs_cache_failures_scheme_idx
    ON vfs_cache_failures(scheme, operation, failed_at_ms);

CREATE TABLE IF NOT EXISTS staging_manifest_records (
    id uuid PRIMARY KEY NOT NULL,
    source_uri text NOT NULL,
    source_scheme text NOT NULL,
    purpose text NOT NULL,
    local_path text NOT NULL UNIQUE,
    size_bytes bigint,
    etag text,
    fingerprint text,
    state text NOT NULL,
    created_at_ms bigint NOT NULL,
    updated_at_ms bigint NOT NULL,
    last_accessed_at_ms bigint NOT NULL,
    expires_at_ms bigint,
    active_leases bigint NOT NULL DEFAULT 0,
    validation_error text,
    CHECK (size_bytes IS NULL OR size_bytes >= 0),
    CHECK (active_leases >= 0)
);

CREATE INDEX IF NOT EXISTS idx_staging_manifest_source_purpose
    ON staging_manifest_records(source_uri, purpose);
CREATE INDEX IF NOT EXISTS idx_staging_manifest_cleanup
    ON staging_manifest_records(state, active_leases, expires_at_ms, last_accessed_at_ms);
