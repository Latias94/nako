import type { FetchLike, MediaItemDto } from "@nako/sdk"
import { describe, expect, it, vi } from "vitest"
import {
  ADMIN_DASHBOARD_FIXTURE,
  createAdminDashboardDataSource,
} from "@/src/api/admin/dashboard-data-source"
import {
  ADMIN_ADDON_MANAGER_FIXTURE,
  createAdminAddonManagerDataSource,
} from "@/src/api/admin/addons-data-source"
import {
  ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE,
  ADMIN_GENERATED_ARTIFACT_METADATA_APPLY_PLAN_FIXTURE,
  ADMIN_GENERATED_ARTIFACT_REVIEW_PLAN_FIXTURE,
  ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE,
  ADMIN_LIBRARY_READ_MODEL_FIXTURE,
  ADMIN_LOGS_READ_MODEL_FIXTURE,
  ADMIN_SETTINGS_READ_MODEL_FIXTURE,
  ADMIN_TASKS_READ_MODEL_FIXTURE,
  ADMIN_USERS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
} from "@/src/api/admin/read-models-data-source"
import { createAdminMutationDataSource } from "@/src/api/admin/mutations-data-source"
import {
  createPublicManagementContextDataSource,
  type PublicManagementContextLink,
} from "@/src/api/public/management-context-data-source"
import { createPublicMediaDataSource } from "@/src/api/public/media-data-source"
import {
  resolveManagementContextLink,
  resolveManagementContextLinks,
} from "@/src/shell"

const page = {
  limit: 10,
  offset: 0,
  returned: 1,
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
}

function publicMediaItem(overrides: Partial<MediaItemDto> = {}): MediaItemDto {
  return {
    id: "live-movie",
    kind: "movie",
    parent_id: null,
    metadata: {
      collections: [],
      credits: [],
      external_ids: [],
      genres: ["Sci-Fi"],
      original_title: "Live Original",
      overview: "A mapped public API item.",
      ratings: [{ source: "tmdb", value: "7.86" }],
      release_date: "2026-01-02",
      runtime_minutes: 125,
      sort_title: null,
      studios: [],
      tagline: null,
      tags: [],
      title: "Live Movie",
    },
    ...overrides,
  }
}

function publicLibrary(overrides: Record<string, unknown> = {}) {
  return {
    id: "library-a",
    name: "Live Library",
    roots: ["/media/live"],
    options: {
      domain: "video",
      preset: "movies",
      naming_strategy: "movie",
      scan: {
        max_depth: null,
        realtime_monitor: true,
      },
      metadata_profile: {
        country: null,
        image_providers: [],
        item_kinds: ["movie"],
        language: null,
        local_metadata_policy: "read_only",
        local_readers: [],
        metadata_providers: [],
        refresh_mode: "default",
        scan: {
          addon_scrape: true,
          addon_writeback: false,
          enabled: true,
        },
      },
    },
    ...overrides,
  }
}

function publicUserPlaylist(overrides: Record<string, unknown> = {}) {
  return {
    id: "playlist-live",
    name: "Live Playlist",
    visibility: "private",
    item_count: 1,
    created_at: "2026-05-29T00:00:00Z",
    updated_at: "2026-05-29T01:00:00Z",
    version: 2,
    ...overrides,
  }
}

function managementContextLinksResponse(overrides: Record<string, unknown> = {}) {
  return {
    context: {
      library_id: "library-a",
      item_id: "item-a",
      source_id: "source-a",
      playback_session_id: "session-a",
    },
    links: [managementContextLink()],
    ...overrides,
  }
}

function managementContextLink(overrides: Record<string, unknown> = {}) {
  return {
    action: "scan_library",
    disabled_reason: null,
    enabled: true,
    method: "POST",
    required_access: "library_manage",
    route_name: "library.scan",
    surface: "management",
    target: {
      library_id: "library-a",
      item_id: "item-a",
      source_id: "source-a",
      playback_session_id: "session-a",
    },
    ...overrides,
  }
}

function publicManagementContextLink(
  overrides: Partial<PublicManagementContextLink> = {},
): PublicManagementContextLink {
  return {
    action: "scan_library",
    disabledReason: null,
    enabled: true,
    method: "POST",
    requiredAccess: "library_manage",
    routeName: "library.scan",
    surface: "management",
    target: {
      libraryId: "library-a",
      itemId: "item-a",
      sourceId: "source-a",
      playbackSessionId: "session-a",
    },
    ...overrides,
  }
}

function adminOverviewResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    status: "healthy",
    storage: {
      total_backends: 4,
      ready_backends: 3,
      degraded_backends: 1,
      unavailable_backends: 0,
      backends: [],
    },
    metadata: {
      total_providers: 2,
      available_providers: 2,
      disabled_providers: 0,
      unavailable_providers: 0,
      providers: [],
    },
    runtime: {
      active_tasks: 1,
      completed_tasks: 3,
      failed_tasks: 0,
      succeeded_jobs: 3,
      cancelled_jobs: 0,
      failed_jobs: 0,
      shutdown_requested: false,
    },
    startup: {
      configured_libraries: 7,
      recovered_transcode_sessions: 0,
      recovered_jobs: 0,
      staging_deleted_records: 0,
      staging_deleted_files: 0,
      metadata_raw_cache_deleted: 0,
      metadata_lifecycle_tasks_started: 1,
      artwork_ingest_worker_started: true,
    },
  }
}

function adminJobsResponse() {
  return {
    jobs: [
      {
        id: "job-1",
        kind: "library_scan",
        status: "running",
        resource_class: "library",
        library_id: "library-a",
        source_id: null,
        has_input: true,
        has_summary: false,
        has_error: false,
        queued_at: "2026-05-28T10:00:00Z",
        started_at: "2026-05-28T10:01:00Z",
        completed_at: null,
      },
    ],
    page,
  }
}

function adminPlaybackSessionsResponse() {
  return {
    sessions: [
      {
        id: "session-1",
        principal_id: "user-1",
        source_id: "source-1",
        item_id: "item-1",
        mode: "direct_play",
        state: "playing",
        transcode_session_id: null,
        has_client_capabilities: true,
        active: true,
        terminal: false,
        created_at: "2026-05-28T10:00:00Z",
        updated_at: "2026-05-28T10:10:00Z",
        started_at_ms: 1000,
        ended_at_ms: null,
        last_heartbeat_at_ms: 2000,
      },
      {
        id: "session-2",
        principal_id: "user-2",
        source_id: "source-2",
        item_id: "item-2",
        mode: "hls",
        state: "ended",
        transcode_session_id: "transcode-1",
        has_client_capabilities: true,
        active: false,
        terminal: true,
        created_at: "2026-05-28T09:00:00Z",
        updated_at: "2026-05-28T09:30:00Z",
        started_at_ms: 1000,
        ended_at_ms: 2000,
        last_heartbeat_at_ms: 2000,
      },
    ],
    page: {
      ...page,
      returned: 2,
    },
  }
}

function adminPlaybackRuntimeResponse(status = "ready") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    readiness: {
      status,
      reason: status,
      checks: [],
    },
  }
}

function adminSystemConfigResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
  }
}

function adminReadModelOverviewResponse() {
  return {
    ...adminOverviewResponse(),
    storage: {
      ...adminOverviewResponse().storage,
      backends: [
        {
          library_id: "library-a",
          library_name: "Movies",
          backend_kind: "local",
          status: "ready",
        },
      ],
    },
  }
}

function adminReadModelSystemConfigResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    auth: {
      enabled: true,
      token_env: "NAKO_ADMIN_TOKEN",
    },
    network: {
      exposure_mode: "private_network",
      readiness: {
        status: "ready",
        reason: "ready",
        checks: [],
      },
      external_endpoint: {
        configured: true,
        scheme: "https",
        host_fingerprint: "fp",
      },
      trusted_proxy: {
        headers_enabled: true,
        source_count: 1,
      },
      origins: {
        allowed_origin_count: 2,
        configured: true,
      },
      tunnel_providers: [],
    },
    database: {
      configured_backend_kind: "sqlite",
      active_backend_kind: "sqlite",
      url_scheme: "sqlite",
      runtime_supported: true,
      migrated_on_startup: true,
      capabilities: {
        lifecycle: true,
        libraries: true,
        jobs: true,
        job_leases: true,
        media: true,
        scan_commits: true,
        metadata: true,
        catalog: true,
        playback_state: true,
        playback_sessions: true,
        transcode_sessions: true,
        event_outbox: true,
        addons: true,
        automation: true,
        managed_artwork: true,
        vfs_cache: true,
        webhooks: true,
        search_index: true,
      },
    },
    runtime: {
      listen_addr: "0.0.0.0:8096",
      scan_concurrency: 2,
      probe_concurrency: 4,
      metadata_concurrency: 3,
      remux_concurrency: 2,
      webhook_concurrency: 1,
      remux_timeout_ms: 120000,
    },
    libraries: [
      {
        id: "library-a",
        name: "Movies",
        preset: "movie",
        backend_kind: "local",
        root_scheme: "file",
        has_webdav_password_env: false,
        webdav_timeout_ms: null,
        webdav_max_attempts: null,
      },
    ],
    metadata: {
      raw_cache_retention_ms: 86400000,
      raw_cache_cleanup_on_startup: true,
      raw_cache_cleanup_interval_ms: 3600000,
      maintenance_policies: 1,
      providers: [
        {
          provider: "tmdb",
          enabled: true,
          token_env: null,
          api_key_env: "TMDB_API_KEY",
          has_api_base_url: true,
          has_image_base_url: true,
          language: "zh-CN",
          include_adult: false,
          header_count: 0,
          secret_header_count: 0,
          has_provider_runtime_override: false,
        },
      ],
      runtime: {
        timeout_ms: 10000,
        max_attempts: 3,
        min_interval_ms: 100,
        concurrency: 4,
        user_agent: "nako-test",
        has_proxy: false,
        circuit_breaker_failures: 3,
        circuit_breaker_backoff_ms: 1000,
      },
    },
    transcode: {
      hardware_policy: {},
      cpu_concurrency: 2,
      gpu_concurrency: 1,
    },
    staging: {
      max_bytes: 1024,
      retention_ms: 86400000,
      cleanup_on_startup: true,
    },
    playback: {
      remote_stream_concurrency: 4,
      remote_stage_concurrency: 2,
      transcode_artifact_retention_ms: 86400000,
      transcode_artifact_cleanup_on_startup: true,
      hls_segment_cleanup_enabled: true,
      hls_segment_keep_ms: 60000,
      transcode_throttle_enabled: false,
      transcode_throttle_delay_ms: 0,
    },
    artwork: {
      artifact_root_configured: true,
      fetch_timeout_ms: 10000,
      fetch_max_attempts: 3,
      fetch_max_bytes: 4096,
      fetch_concurrency: 2,
      ingest_worker_enabled: true,
      ingest_worker_idle_ms: 1000,
      fetch_user_agent: "nako-test",
      has_fetch_proxy: false,
      max_width: 3000,
      max_height: 3000,
    },
  }
}

function adminAccessSummaryResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    mode: "single_admin",
    principal: {
      principal_id: "principal-admin",
      display_name: "Admin",
      principal_kind: "local_admin",
    },
    auth: {
      enabled: true,
      token_reference_configured: true,
    },
    readiness: {
      single_admin_mode: "active",
      user_accounts: "active",
      roles: "active",
      library_access_policy: "planned",
    },
    library_access: {
      configured_libraries: 1,
      libraries: [
        {
          library_id: "library-a",
          library_name: "Movies",
          preset: "movie",
          backend_kind: "local",
          access: "manage",
          reason: "single_admin_mode",
        },
      ],
    },
  }
}

function adminAccessUsersResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    users: [
      {
        user_id: "user-1",
        principal_id: "user-1",
        username: "admin",
        display_name: "Admin",
        status: "active",
        roles: ["administrator"],
        bootstrap: true,
        local_password_configured: true,
        created_at_ms: 1000,
        updated_at_ms: 2000,
      },
    ],
    page,
  }
}

function adminEventsResponse() {
  return {
    events: [
      {
        id: "event-1",
        kind: "library.scan.completed",
        subject: "library-a",
        library_id: "library-a",
        source_id: null,
        status: "pending",
        attempts: 1,
        has_payload: true,
        has_error: false,
        occurred_at: "2026-05-28T10:00:00Z",
        updated_at: "2026-05-28T10:01:00Z",
        next_attempt_at: null,
      },
    ],
    page,
  }
}

function adminStorageStagingResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    summary: {
      configured_max_bytes: 1000,
      used_manifest_bytes: 250,
      cleanup_on_startup: true,
      retention_ms: 86400000,
      startup_deleted_records: 0,
      startup_deleted_files: 0,
      process_cached_backends: 0,
      vfs_cache: {
        object_count: 0,
        listing_count: 0,
        failure_count: 0,
        stale_object_count: 0,
        stale_listing_count: 0,
        last_failure_at_ms: null,
      },
    },
    records: [],
    page,
  }
}

function adminAcquisitionIntakeCandidatesResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    candidates: [
      {
        id: "candidate-1",
        target_library_id: "library-a",
        source_kind: "watch_folder",
        custom_source_kind: false,
        source_scheme: "file",
        source_ref_redacted: "file://<redacted>/Movie.mkv",
        source_key_fingerprint: "sha256:candidate-1",
        has_display_name: true,
        has_intended_locator: true,
        size_bytes: 123456789,
        has_fingerprint: true,
        managed_import_artifact_id: "artifact-1",
        state: "ready",
        has_diagnostics: true,
        first_seen_at_ms: 1710468000000,
        last_seen_at_ms: 1710468300000,
        created_at_ms: 1710468000000,
        updated_at_ms: 1710468300000,
        intended_locator: "file:///mnt/private/raw/Movie.mkv",
        prompt_body: "unsafe prompt body",
      },
    ],
    page,
  }
}

function adminGeneratedArtifactProposalsResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    proposals: [
      {
        id: "artifact-live",
        kind: "metadata_suggestion",
        capability: "item_metadata_suggest",
        status: "pending_review",
        target: {
          kind: "media_item",
          library_id: "library-a",
          item_id: "item-live",
          source_id: "source-live",
          local_path: "F:\\private\\source\\Movie.mkv",
          source_locator: "file:///mnt/private/source/Movie.mkv",
        },
        provenance: {
          provider_id: "provider-live",
          provider_name: "Live Automation Provider",
          job_id: "job-live",
          capability: "item_metadata_suggest",
          idempotency_key_fingerprint: "sha256:idempotency-live",
          prompt_fingerprint: "sha256:prompt-live",
          attempt_count: 2,
          artifact_created_at: "2026-05-29T01:00:00Z",
          raw_prompt: "unsafe prompt body",
          provider_raw_response: "provider secret response",
        },
        payload: {
          valid_json: true,
          shape: "object",
          payload_fingerprint: "sha256:payload-live",
          payload_bytes: 4096,
          object_field_count: 9,
          array_item_count: null,
          has_textual_values: true,
          has_explanation: true,
          confidence_milli: 910,
          raw_payload: {
            title: "unsafe generated payload title",
            secret: "provider-secret",
          },
        },
        readiness: {
          status: "ready",
          actionable: true,
          reasons: ["ready_for_review"],
        },
        created_at: "2026-05-29T01:01:00Z",
        updated_at: "2026-05-29T01:05:00Z",
        accepted_at: null,
        artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function adminMetadataCandidateReviewResponse(reviewId = "review-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    review: adminMetadataCandidateReviewDetail(reviewId),
    application_plan: adminMetadataCandidateReviewApplicationPlan(reviewId, "apply", ["ready"]),
    boundary: adminMetadataCandidateReviewBoundary(),
    raw_provider_response: "provider secret response",
    idempotency_key: "candidate-review:operator-secret",
  }
}

function adminMetadataCandidateReviewListResponse(itemId = "item-live") {
  const newer = adminMetadataCandidateReviewDetail("review-live-newer")
  const older = adminMetadataCandidateReviewDetail("review-live-older")

  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    item_id: itemId,
    reviews: [
      {
        review_id: newer.review_id,
        item_id: itemId,
        source: { provider: "bangumi" },
        source_key: "bangumi:newer",
        status: "pending",
        root: {
          ...newer.root,
          metadata: adminMetadataCandidateSummary("Newer Candidate", {
            raw_overview: "secret candidate overview",
            raw_tags: ["secret-candidate-tag"],
            source_locator: "local:///Private/Live.S01E02.mkv?token=secret",
            source_fingerprint: "sha256-private-list-candidate",
          }),
        },
        related_count: 1,
        relationship_count: 1,
        application_plan: adminMetadataCandidateReviewApplicationPlan(
          newer.review_id,
          "skip",
          ["review_status_not_accepted"],
        ),
        boundary: adminMetadataCandidateReviewBoundary({
          apply_mutation_required: false,
          apply_updates_root_provider_subject: false,
          apply_updates_root_provider_mapping: false,
        }),
        expires_at_ms: null,
        created_at_ms: 100,
        updated_at_ms: 500,
        raw_provider_response: "provider secret response",
      },
      {
        review_id: older.review_id,
        item_id: itemId,
        source: { provider: "bangumi" },
        source_key: "bangumi:older",
        status: "accepted",
        root: older.root,
        related_count: 0,
        relationship_count: 0,
        application_plan: adminMetadataCandidateReviewApplicationPlan(
          older.review_id,
          "apply",
          ["ready"],
        ),
        boundary: adminMetadataCandidateReviewBoundary(),
        expires_at_ms: null,
        created_at_ms: 90,
        updated_at_ms: 300,
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 2,
    },
  }
}

function adminMetadataCandidateReviewQueueResponse() {
  const list = adminMetadataCandidateReviewListResponse("item/unsafe id")

  return {
    admin_api_version: list.admin_api_version,
    public_api_version: list.public_api_version,
    reviews: list.reviews.map((review, index) => ({
      ...review,
      item_id: index === 0 ? "item/unsafe id" : "item-live-other",
    })),
    page: list.page,
    raw_provider_response: "provider secret response",
  }
}

function adminMetadataCandidateReviewApplyResponse(reviewId = "review-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    review_id: reviewId,
    item_id: "item-live",
    applied: true,
    changed: false,
    idempotent_replay: true,
    idempotency_key_fingerprint: "0123456789abcdef",
    plan: adminMetadataCandidateReviewApplicationPlan(
      reviewId,
      "noop",
      ["existing_accepted_mapping"],
    ),
    provider_subject: {
      subject_id: "subject-live",
      provider: "bangumi",
      subject_kind: "subject",
      subject_key: "1437",
      title: "Live Candidate",
      release_year: 2026,
      locale: "zh-CN",
      raw_subject_payload: "provider secret response",
    },
    provider_mapping: {
      mapping_id: "mapping-live",
      item_id: "item-live",
      subject_id: "subject-live",
      status: "accepted",
      confidence_milli: 940,
      source: "user",
      raw_provider_mapping: "provider secret response",
    },
    boundary: adminMetadataCandidateReviewBoundary({
      read_only: false,
      applies_on_read: false,
      apply_mutation_required: true,
    }),
    idempotency_key: "web-metadata-candidate-review-apply:review-unsafe-id:test",
  }
}

function adminMetadataCandidateReviewBatchResponse(
  reviewIds = ["review/unsafe id"],
  status: "queued" | "running" | "completed" | "failed" | "cancelled" = "completed",
) {
  const completed = status === "completed"
  const failed = status === "failed"
  const pending = status === "queued" || status === "running"

  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    batch: {
      id: "candidate-review-batch-live",
      job_id: "candidate-review-batch-job-live",
      status,
      idempotency_key_fingerprint: "batch-key-fingerprint",
      selection: {
        requested_review_count: reviewIds.length,
        selected_review_count: reviewIds.length,
        duplicate_review_count: 0,
        max_review_count: 50,
      },
      summary: {
        requested_count: reviewIds.length,
        returned_count: reviewIds.length,
        max_review_count: 50,
        apply_count: reviewIds.length,
        noop_count: 0,
        skip_count: 0,
      },
      execution_summary: {
        total_item_count: reviewIds.length,
        pending_item_count: pending ? reviewIds.length : 0,
        skipped_item_count: 0,
        blocked_item_count: 0,
        applied_item_count: completed ? reviewIds.length : 0,
        noop_item_count: 0,
        stale_item_count: 0,
        conflict_item_count: 0,
        failed_item_count: failed ? reviewIds.length : 0,
      },
      items: reviewIds.map((reviewId, position) => ({
        review_id: reviewId,
        item_id: "item-live",
        position,
        status: completed ? "applied" : failed ? "failed" : "pending",
        idempotency_key_fingerprint: `row-key-fingerprint-${position}`,
        expected_updated_at_ms: 300,
        provider_subject_id: completed ? "subject-live" : null,
        provider_mapping_id: completed ? "mapping-batch-live" : null,
        error: failed
          ? {
              code: "provider_mapping_conflict",
              message: "Provider Mapping changed before apply",
            }
          : null,
        plan: adminMetadataCandidateReviewApplicationPlan(reviewId, "apply", ["ready"]),
        boundary: adminMetadataCandidateReviewBoundary({
          read_only: false,
          applies_on_read: false,
          apply_mutation_required: true,
        }),
        created_at: "2026-06-02T02:00:00Z",
        updated_at: "2026-06-02T02:01:00Z",
        idempotency_key: "unsafe-item-idempotency-key",
        raw_provider_response: "provider secret response",
      })),
      created_at: "2026-06-02T02:00:00Z",
      updated_at: "2026-06-02T02:01:00Z",
      idempotency_key: "unsafe-batch-idempotency-key",
      raw_provider_response: "provider secret response",
    },
  }
}

function adminMetadataCandidateReviewDetail(reviewId = "review-live") {
  const rootSubject = adminMetadataCandidateSubject("subject", "1437", "Live Candidate")
  const childSubject = adminMetadataCandidateSubject("episode", "1437/1", "Episode One")

  return {
    review_id: reviewId,
    item_id: "item-live",
    source: { provider: "bangumi" },
    source_key: "bangumi:1437",
    status: "accepted",
    root: {
      source: { provider: "bangumi" },
      kind: "series",
      subject: rootSubject,
      metadata: adminMetadataCandidateSummary("Live Candidate", {
        raw_overview: "secret candidate overview",
        raw_tags: ["secret-candidate-tag"],
        source_locator: "local:///Private/Live.S01E01.mkv?token=secret",
        source_fingerprint: "sha256-private-candidate",
      }),
    },
    related: [
      {
        source: { provider: "bangumi" },
        kind: "episode",
        subject: childSubject,
        metadata: adminMetadataCandidateSummary("Episode One", {
          raw_overview: "secret related overview",
        }),
      },
    ],
    relationships: [
      {
        parent_subject: rootSubject,
        child_subject: childSubject,
        kind: "contains",
      },
    ],
    related_count: 1,
    relationship_count: 1,
    expires_at_ms: null,
    created_at_ms: 100,
    updated_at_ms: 300,
  }
}

function adminMetadataCandidateReviewApplicationPlan(
  reviewId: string,
  action: "apply" | "skip" | "noop",
  reasons: string[],
) {
  return {
    review_id: reviewId,
    item_id: "item-live",
    action,
    reasons,
    source: "user",
    root_subject: adminMetadataCandidateSubject("subject", "1437", "Live Candidate"),
    existing_mapping_id: action === "noop" ? "mapping-live" : null,
    existing_mapping_status: action === "noop" ? "accepted" : null,
    raw_provider_response: "provider secret response",
  }
}

function adminMetadataCandidateReviewBoundary(
  overrides: Record<string, boolean> = {},
) {
  return {
    read_only: true,
    applies_on_read: false,
    apply_mutation_required: true,
    apply_updates_root_provider_subject: true,
    apply_updates_root_provider_mapping: true,
    apply_updates_related_provider_subjects: false,
    apply_updates_related_provider_mappings: false,
    updates_canonical_metadata: false,
    updates_hierarchy: false,
    writes_nfo: false,
    writes_library_files: false,
    ...overrides,
  }
}

function adminMetadataCandidateSubject(
  subjectKind: string,
  subjectKey: string,
  title: string,
) {
  return {
    provider: "bangumi",
    subject_kind: subjectKind,
    subject_key: subjectKey,
    title,
    release_year: 2026,
    locale: "zh-CN",
    raw_subject_payload: "provider secret response",
  }
}

function adminMetadataCandidateSummary(
  title: string,
  unsafeFields: Record<string, unknown> = {},
) {
  return {
    title,
    original_title: null,
    sort_title: null,
    release_date: "2026-06-01",
    runtime_minutes: null,
    description_present: true,
    tagline_present: false,
    genre_count: 1,
    tag_count: 1,
    rating_count: 1,
    image_count: 1,
    credit_count: 0,
    collection_count: 0,
    studio_count: 0,
    external_id_count: 1,
    ...unsafeFields,
  }
}

function adminGeneratedArtifactReviewPlanResponse(
  artifactId = "artifact-live",
  decision: "accept" | "reject" = "accept",
) {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    plan: adminGeneratedArtifactAcceptancePlan(artifactId, decision),
  }
}

function adminGeneratedArtifactReviewResponse(
  artifactId = "artifact-live",
  decision: "accept" | "reject" = "accept",
) {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    artifact_id: artifactId,
    decision,
    artifact_status: decision === "accept" ? "accepted" : "rejected",
    accepted_at: decision === "accept" ? "2026-05-29T01:10:00Z" : null,
    idempotent_replay: true,
    plan: adminGeneratedArtifactAcceptancePlan(artifactId, decision),
  }
}

function adminGeneratedArtifactMetadataApplyPlanResponse(artifactId = "artifact-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    plan: adminGeneratedArtifactMetadataApplyPlan(artifactId),
  }
}

function adminGeneratedArtifactMetadataApplyResponse(artifactId = "artifact-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    outcome_id: "metadata-apply-outcome-live",
    artifact_id: artifactId,
    status: "applied",
    applied: true,
    changed: true,
    idempotent_replay: true,
    applied_source: "user",
    plan: adminGeneratedArtifactMetadataApplyPlan(artifactId),
  }
}

function adminGeneratedArtifactMetadataApplyRecoveryResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    summary: {
      returned_entry_count: 1,
      needs_repair_count: 1,
      needs_review_count: 0,
      replay_only_count: 0,
      resolved_count: 0,
    },
    entries: [
      {
        source: "apply_outcome",
        attention: "needs_repair",
        reason: "apply_outcome_failed",
        artifact_id: "artifact/unsafe id",
        outcome_id: "metadata-apply-outcome-live",
        batch_id: null,
        batch_item_status: null,
        outcome_status: "failed",
        item_id: "item-live",
        plan: adminGeneratedArtifactMetadataApplyPlan("artifact/unsafe id"),
        error_code: "target_stale",
        error_message: "target became stale before apply execution",
        created_at: "2026-06-02T12:00:00Z",
        updated_at: "2026-06-02T12:05:00Z",
        idempotency_key: "unsafe-recovery-idempotency",
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function adminGeneratedArtifactMetadataBulkApplyPlanResponse(
  artifactIds = ["artifact-live", "artifact-missing"],
) {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    plan: {
      selection: {
        requested_artifact_count: artifactIds.length,
        selected_artifact_count: artifactIds.length,
        duplicate_artifact_count: 0,
        max_artifact_count: 100,
      },
      summary: {
        planned_artifact_count: 1,
        missing_artifact_count: artifactIds.length > 1 ? 1 : 0,
        ready_artifact_count: 1,
        blocked_artifact_count: 0,
        stale_artifact_count: 0,
        executable_artifact_count: 1,
        apply_field_count: 1,
        skipped_field_count: 1,
        noop_field_count: 1,
        apply_provider_mapping_count: 1,
        skipped_provider_mapping_count: 0,
        noop_provider_mapping_count: 1,
      },
      items: artifactIds.map((artifactId, index) =>
        index === 0
          ? {
              artifact_id: artifactId,
              status: "planned",
              executable: true,
              reasons: ["planned"],
              plan: adminGeneratedArtifactMetadataApplyPlan(artifactId),
            }
          : {
              artifact_id: artifactId,
              status: "missing",
              executable: false,
              reasons: ["missing_artifact"],
              plan: null,
              raw_payload: "unsafe missing artifact payload",
            },
      ),
      raw_prompt: "unsafe prompt body",
    },
  }
}

function adminGeneratedArtifactMetadataBulkApplyBatchResponse(
  artifactIds = ["artifact-live", "artifact-missing"],
) {
  const plan = adminGeneratedArtifactMetadataBulkApplyPlanResponse(artifactIds).plan

  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    batch: {
      id: "bulk-batch-live",
      job_id: "bulk-job-live",
      idempotency_key: "unsafe-bulk-idempotency-key",
      status: "completed",
      selection: plan.selection,
      summary: plan.summary,
      execution_summary: {
        total_item_count: artifactIds.length,
        pending_item_count: 0,
        skipped_item_count: artifactIds.length > 1 ? 1 : 0,
        applied_item_count: 1,
        noop_item_count: 0,
        stale_item_count: 0,
        failed_item_count: 0,
      },
      items: artifactIds.map((artifactId, index) => ({
        artifact_id: artifactId,
        position: index,
        status: index === 0 ? "applied" : "skipped",
        idempotency_key: `unsafe-item-idempotency-${index}`,
        outcome_id: index === 0 ? "metadata-apply-outcome-live" : null,
        error_code: index === 0 ? null : "missing_artifact",
        error_message: index === 0 ? null : "safe missing artifact",
        plan_item: plan.items[index],
        created_at: "2026-06-01T00:00:00Z",
        updated_at: "2026-06-01T00:00:01Z",
      })),
      created_at: "2026-06-01T00:00:00Z",
      updated_at: "2026-06-01T00:00:01Z",
      raw_artifact_json: "unsafe generated payload title",
    },
  }
}

function adminGeneratedArtifactAcceptancePlan(
  artifactId = "artifact-live",
  decision: "accept" | "reject" = "accept",
) {
  return {
    artifact_id: artifactId,
    decision,
    status: "ready",
    action: decision === "accept" ? "accept_generated_artifact" : "reject_generated_artifact",
    reasons: ["ready_for_review"],
    capability: "item_metadata_suggest",
    kind: "metadata_suggestion",
    target: {
      kind: "media_item",
      library_id: "library-a",
      item_id: "item-live",
      source_id: "source-live",
      local_path: "F:\\private\\source\\Movie.mkv",
      source_locator: "file:///mnt/private/source/Movie.mkv",
    },
    payload: {
      valid_json: true,
      shape: "object",
      payload_fingerprint: "sha256:payload-live",
      payload_bytes: 4096,
      object_field_count: 9,
      array_item_count: null,
      has_textual_values: true,
      has_explanation: true,
      confidence_milli: 910,
      raw_payload: {
        title: "unsafe generated payload title",
        secret: "provider-secret",
      },
    },
    readiness: {
      status: "ready",
      actionable: true,
      reasons: ["ready_for_review"],
    },
    boundary: {
      accepted_into_canonical_metadata: false,
      writes_sidecar: false,
      writes_library_files: false,
      applies_immediately: false,
      requires_metadata_authority_apply: decision === "accept",
    },
    raw_prompt: "unsafe prompt body",
    provider_raw_response: "provider secret response",
    artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
  }
}

function adminGeneratedArtifactMetadataApplyPlan(artifactId = "artifact-live") {
  return {
    artifact_id: artifactId,
    status: "ready",
    executable: true,
    reasons: ["accepted_generated_artifact"],
    target: {
      kind: "media_item",
      library_id: "library-a",
      item_id: "item-live",
      source_id: "source-live",
      local_path: "F:\\private\\source\\Movie.mkv",
      source_locator: "file:///mnt/private/source/Movie.mkv",
    },
    payload: {
      valid_json: true,
      shape: "object",
      payload_fingerprint: "sha256:payload-live",
      payload_bytes: 4096,
      object_field_count: 9,
      array_item_count: null,
      has_textual_values: true,
      has_explanation: true,
      confidence_milli: 910,
      raw_payload: {
        title: "unsafe generated payload title",
        secret: "provider-secret",
      },
    },
    fields: [
      {
        field: "title",
        action: "apply",
        reasons: ["incoming_differs"],
        current: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:current-title",
          value_bytes: 12,
          item_count: null,
          raw_value: "unsafe current title",
        },
        incoming: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:incoming-title",
          value_bytes: 16,
          item_count: null,
          raw_value: "unsafe generated payload title",
        },
      },
      {
        field: "overview",
        action: "skip",
        reasons: ["field_locked"],
        current: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:current-overview",
          value_bytes: 32,
          item_count: null,
        },
        incoming: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:incoming-overview",
          value_bytes: 64,
          item_count: null,
        },
      },
      {
        field: "genres",
        action: "noop",
        reasons: ["same_value"],
        current: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:genres",
          value_bytes: null,
          item_count: 2,
        },
        incoming: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:genres",
          value_bytes: null,
          item_count: 2,
        },
      },
    ],
    provider_mappings: adminGeneratedArtifactProviderMappingPlans(),
    apply_field_count: 1,
    skipped_field_count: 1,
    noop_field_count: 1,
    apply_provider_mapping_count: 1,
    skipped_provider_mapping_count: 0,
    noop_provider_mapping_count: 1,
    raw_prompt: "unsafe prompt body",
    provider_raw_response: "provider secret response",
    artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
  }
}

function adminGeneratedArtifactProviderMappingPlans() {
  return [
    {
      subject: {
        provider: "tmdb",
        provider_name: "TMDB",
        subject_kind: "movie",
        subject_kind_name: "Movie",
        subject_key: "tmdb-123",
        title: "Live Movie",
        release_year: 2026,
        locale: "zh-CN",
        raw_subject_payload: "provider secret response",
      },
      action: "apply",
      reasons: ["incoming_provider_subject"],
      confidence_milli: 910,
      existing_mapping_status: null,
      raw_provider_mapping: "provider secret response",
    },
    {
      subject: {
        provider: "tmdb",
        provider_name: "TMDB",
        subject_kind: "collection",
        subject_kind_name: "Collection",
        subject_key: "tmdb-collection-9",
        title: "Live Collection",
        release_year: null,
        locale: "zh-CN",
      },
      action: "noop",
      reasons: ["existing_mapping_same_subject"],
      confidence_milli: 870,
      existing_mapping_status: "accepted",
    },
  ]
}

function adminPlaybackRuntimeFullResponse() {
  return {
    ...adminPlaybackRuntimeResponse(),
    policy: {
      user_policy_rows_supported: true,
      role_policy_rows_supported: true,
      effective_resolution_supported: true,
      library_access_required: true,
      user_policy_overrides_role_policy: false,
      role_policy_merge: "restrictive",
      permissions: ["media_playback"],
    },
    ffmpeg: {
      probe_status: "ok",
      has_probe_error: false,
      hardware_capability_count: 1,
      available_gpu_capabilities: 1,
    },
    hardware: {
      policy: {},
      selection: {
        acceleration: "qsv",
        fallback_used: false,
        reason: "ready",
      },
      capabilities: [],
    },
    transcode: {
      configured_cpu_slots: 2,
      configured_gpu_slots: 1,
      effective_cpu_slots: 2,
      effective_gpu_slots: 1,
      selected_hls_slots: 1,
    },
    remux: {
      max_concurrent_sessions: 2,
      timeout_ms: 120000,
    },
    remote_playback: {
      backend_count: 1,
      stream_permits_available: 4,
      stream_permits_max: 4,
      stage_permits_available: 2,
      stage_permits_max: 2,
      state_scope: "server",
    },
    staging: {
      max_bytes: 1000,
      retention_ms: 86400000,
      cleanup_on_startup: true,
      startup_deleted_records: 0,
      startup_deleted_files: 0,
    },
    artifact_lifecycle: {
      transcode_artifact_retention_ms: 86400000,
      transcode_artifact_cleanup_on_startup: true,
      hls_segment_cleanup_enabled: true,
      hls_segment_keep_ms: 60000,
      startup_examined_artifacts: 0,
      startup_deleted_artifacts: 0,
      startup_deleted_files: 0,
      startup_deleted_directories: 0,
      startup_deleted_bytes: 0,
      startup_skipped_security: 0,
    },
    throttle: {
      enabled: false,
      delay_ms: 0,
    },
  }
}

function adminRawCacheSettingsResponse() {
  return {
    admin_api_version: "v1",
    retention_ms: 86400000,
    cleanup_on_startup: true,
    source: "configured",
    effect: "active",
    updated_at_ms: null,
  }
}

function adminAccessUserResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    user: adminAccessUsersResponse().users[0],
  }
}

function adminLocalPasswordResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    user_id: "user-1",
    local_password_configured: true,
  }
}

function adminAddonSummary(status: "enabled" | "disabled" = "enabled") {
  return {
    id: "addon-1",
    manifest_id: "nako.tmdb",
    name: "TMDb Metadata Sidecar",
    version: "0.1.0",
    protocol_version: "v1",
    base_url: "http://127.0.0.1:9101",
    outbound_task_dispatch_secret_env: "NAKO_ADDON_TMDB_TASK_SECRET",
    granted_scopes: ["catalog_read", "item_metadata_read"],
    status,
    created_at: "2026-05-28T00:00:00Z",
    updated_at: "2026-05-28T00:00:00Z",
  }
}

function adminAddonRegistrationsResponse() {
  return {
    addons: [adminAddonSummary()],
  }
}

function adminAddonRegistrationResponse(status: "enabled" | "disabled" = "disabled") {
  return {
    addon: {
      summary: adminAddonSummary(status),
      manifest: {
        id: "nako.tmdb",
        name: "TMDb Metadata Sidecar",
        version: "0.1.0",
        protocol_version: "v1",
        base_url: "http://127.0.0.1:9101",
        description: null,
        resources: [],
        auth: "bearer",
        default_timeout_ms: null,
        default_max_attempts: null,
        scopes: ["catalog_read"],
      },
    },
  }
}

function adminAddonCatalogSourcesResponse() {
  return {
    sources: [
      {
        id: "nako-official",
        name: "Nako Official",
        description: "Built-in official addon catalog.",
        kind: "builtin_official",
        entry_count: 1,
        provides_package_signing: false,
        provides_process_supervision: false,
        provides_provider_breadth: true,
      },
    ],
  }
}

function adminAddonCatalogEntriesResponse() {
  return {
    source_id: "nako-official",
    entries: [
      {
        source_id: "nako-official",
        entry_id: "nako.tmdb",
        manifest_id: "nako.tmdb",
        addon_name: "TMDb Metadata Sidecar",
        addon_version: "0.1.0",
        protocol_version: "v1",
        description: "Metadata sidecar",
        runtime_kind: "http_sidecar",
        resources: ["metadata", "image"],
        scopes: ["catalog_read", "item_metadata_read"],
        tasks: ["refresh-metadata"],
        package_signing_verified: false,
        lifecycle_boundary: {
          nako_manages_containers: false,
          nako_manages_processes: false,
          nako_manages_packages: false,
          message: "Manual sidecar lifecycle boundary.",
        },
      },
    ],
  }
}

describe("public media data source contracts", () => {
  it("uses local fixtures when configured for fixture mode", async () => {
    const source = createPublicMediaDataSource({ mode: "fixture" })

    const payload = await source.listMedia()

    expect(payload.source).toBe("fixture")
    expect(payload.fallback).toBe(true)
    expect(payload.items[0]).toMatchObject({
      id: "1",
      title: "沙丘2",
      type: "movie",
    })
  })

  it("maps live Public Client DTOs into UI media items", async () => {
    const fetcher = vi.fn<FetchLike>(async () =>
      jsonResponse({
        items: [publicMediaItem()],
        page,
      }),
    )

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.listMedia()

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      readiness: [
        {
          id: "recently-added-sort",
          status: "missing_contract",
        },
      ],
      items: [
        {
          id: "live-movie",
          title: "Live Movie",
          originalTitle: "Live Original",
          year: 2026,
          rating: 7.9,
          duration: "2h 5m",
          type: "movie",
        },
      ],
    })
    expect(fetcher.mock.calls[0][0]).toBe("http://nako.test/items?limit=40&offset=0")
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("maps live User Playlist DTOs into playlist tabs and items", async () => {
    const fetcher = vi.fn<FetchLike>(async (input) => {
      const url = new URL(String(input))

      if (url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist()],
          page,
        })
      }

      if (url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist(),
          items: [
            {
              playlist_id: "playlist-live",
              item_id: "live-movie",
              position: 0,
              added_at: "2026-05-29T01:00:00Z",
              item: publicMediaItem(),
              images: [],
            },
          ],
          page,
        })
      }

      return jsonResponse({ message: "not found" }, 404)
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const playlists = await source.listUserPlaylists()
    const items = await source.listUserPlaylistItems("playlist-live")
    const calledTargets = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })

    expect(playlists).toMatchObject({
      source: "live",
      fallback: false,
      playlists: [
        {
          id: "playlist-live",
          name: "Live Playlist",
          itemCount: 1,
        },
      ],
    })
    expect(items).toMatchObject({
      source: "live",
      fallback: false,
      playlist: {
        id: "playlist-live",
        name: "Live Playlist",
      },
      items: [
        {
          playlistId: "playlist-live",
          itemId: "live-movie",
          position: 0,
          item: {
            id: "live-movie",
            title: "Live Movie",
          },
        },
      ],
    })
    expect(calledTargets).toEqual([
      "/users/me/playlists?limit=20&offset=0",
      "/users/me/playlists/playlist-live/items?limit=50&offset=0",
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("maps live User Playlist mutations through Public Client routes", async () => {
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<FetchLike>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method,
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "POST /users/me/playlists":
          return jsonResponse({
            playlist: publicUserPlaylist({
              id: "playlist-new",
              name: (body as { name: string }).name,
              item_count: 0,
              version: 1,
            }),
          })
        case "PATCH /users/me/playlists/playlist-live":
          return jsonResponse({
            playlist: publicUserPlaylist({
              name: (body as { name: string }).name,
              version: 3,
            }),
          })
        case "DELETE /users/me/playlists/playlist-live":
          return jsonResponse({ playlist_id: "playlist-live", deleted: true })
        case "PUT /users/me/playlists/playlist-live/items/live-movie":
          return jsonResponse({
            playlist: publicUserPlaylist({
              item_count: 2,
              version: 4,
            }),
          })
        case "DELETE /users/me/playlists/playlist-live/items/live-movie":
          return jsonResponse({
            playlist: publicUserPlaylist({
              item_count: 0,
              version: 5,
            }),
          })
        case "PUT /users/me/playlists/playlist-live/items/reorder":
          return jsonResponse({
            playlist: publicUserPlaylist({
              version: 6,
            }),
          })
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })
    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const created = await source.createUserPlaylist({ name: "New Queue" })
    const renamed = await source.updateUserPlaylist("playlist-live", {
      name: "Renamed Queue",
      expected_version: 2,
    })
    const deleted = await source.deleteUserPlaylist("playlist-live")
    const added = await source.addUserPlaylistItem("playlist-live", "live-movie", {
      position: 1,
      expected_version: 3,
    })
    const removed = await source.removeUserPlaylistItem("playlist-live", "live-movie")
    const reordered = await source.reorderUserPlaylistItems("playlist-live", {
      item_ids: ["live-movie-b", "live-movie-a"],
      expected_version: 5,
    })

    expect(created).toMatchObject({
      source: "live",
      fallback: false,
      persisted: true,
      playlist: {
        id: "playlist-new",
        name: "New Queue",
        itemCount: 0,
      },
    })
    expect(renamed).toMatchObject({
      source: "live",
      fallback: false,
      persisted: true,
      playlist: {
        id: "playlist-live",
        name: "Renamed Queue",
        version: 3,
      },
    })
    expect(deleted).toMatchObject({
      source: "live",
      fallback: false,
      persisted: true,
      playlistId: "playlist-live",
      deleted: true,
    })
    expect(added).toMatchObject({
      source: "live",
      fallback: false,
      persisted: true,
      playlist: {
        itemCount: 2,
        version: 4,
      },
    })
    expect(removed).toMatchObject({
      source: "live",
      fallback: false,
      persisted: true,
      playlist: {
        itemCount: 0,
        version: 5,
      },
    })
    expect(reordered).toMatchObject({
      source: "live",
      fallback: false,
      persisted: true,
      playlist: {
        version: 6,
      },
    })
    expect(calls).toEqual([
      {
        method: "POST",
        path: "/users/me/playlists",
        body: { name: "New Queue" },
        authorization: "Bearer public-token",
      },
      {
        method: "PATCH",
        path: "/users/me/playlists/playlist-live",
        body: { name: "Renamed Queue", expected_version: 2 },
        authorization: "Bearer public-token",
      },
      {
        method: "DELETE",
        path: "/users/me/playlists/playlist-live",
        body: undefined,
        authorization: "Bearer public-token",
      },
      {
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: { position: 1, expected_version: 3 },
        authorization: "Bearer public-token",
      },
      {
        method: "DELETE",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: undefined,
        authorization: "Bearer public-token",
      },
      {
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/reorder",
        body: { item_ids: ["live-movie-b", "live-movie-a"], expected_version: 5 },
        authorization: "Bearer public-token",
      },
    ])
  })

  it("does not claim fixture User Playlist mutations are persisted", async () => {
    const source = createPublicMediaDataSource({ mode: "fixture" })

    const created = await source.createUserPlaylist({ name: "Draft Queue" })
    const added = await source.addUserPlaylistItem("fixture-watch-later", "2")
    const removed = await source.removeUserPlaylistItem("fixture-watch-later", "1")
    const deleted = await source.deleteUserPlaylist("fixture-watch-later")

    expect(created).toMatchObject({
      source: "fixture",
      fallback: true,
      persisted: false,
      playlist: null,
      error: "Fixture mode does not persist playlist mutations.",
    })
    expect(added).toMatchObject({
      source: "fixture",
      fallback: true,
      persisted: false,
      playlist: {
        id: "fixture-watch-later",
      },
      error: "Fixture mode does not persist playlist mutations.",
    })
    expect(removed).toMatchObject({
      source: "fixture",
      fallback: true,
      persisted: false,
      playlist: {
        id: "fixture-watch-later",
      },
      error: "Fixture mode does not persist playlist mutations.",
    })
    expect(deleted).toMatchObject({
      source: "fixture",
      fallback: true,
      persisted: false,
      playlistId: "fixture-watch-later",
      deleted: false,
      error: "Fixture mode does not persist playlist mutations.",
    })
  })

  it("maps live item detail read models with source and image refs", async () => {
    const fetcher = vi.fn<FetchLike>(async () =>
      jsonResponse({
        item: publicMediaItem(),
        sources: [
          {
            id: "source-1",
            item_id: "live-movie",
            library_id: "library-a",
            file_name: "Live Movie.mkv",
            fingerprint: null,
            size_bytes: 1024,
          },
        ],
        images: [
          {
            id: "image-1",
            kind: "poster",
            url: "/images/image-1",
            width: 1000,
            height: 1500,
            etag: null,
            language: null,
            media_type: "image/jpeg",
            owner: {},
          },
        ],
        collections: [],
        credits: [],
        genres: [],
        studios: [],
        tags: [],
      }),
    )

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.getMediaDetails("live-movie", "movie")

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      item: {
        id: "live-movie",
        title: "Live Movie",
      },
      sources: [
        {
          id: "source-1",
          itemId: "live-movie",
          libraryId: "library-a",
          fileName: "Live Movie.mkv",
          sizeBytes: 1024,
        },
      ],
      images: [
        {
          id: "image-1",
          kind: "poster",
          url: "/images/image-1",
        },
      ],
    })
    expect(fetcher.mock.calls[0][0]).toBe("http://nako.test/items/live-movie")
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("falls back to local search results when the live Public Client request fails", async () => {
    const fetcher = vi.fn<FetchLike>(async () => {
      throw new Error("public offline")
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test",
      },
      fetcher,
    )

    const payload = await source.searchMedia("Dune")

    expect(payload).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "public offline",
    })
    expect(payload.items.map((item) => item.title)).toEqual(["沙丘2"])
  })

  it("reports library item browse as ready once the Public Client route exists", async () => {
    const fetcher = vi.fn<FetchLike>(async (input) => {
      const url = new URL(String(input))

      if (url.pathname === "/libraries/library-a") {
        return jsonResponse({
          library: publicLibrary(),
        })
      }

      if (url.pathname === "/libraries/library-a/sources") {
        return jsonResponse({
          library: publicLibrary(),
          page,
          sources: [
            {
              source: {
                id: "source-1",
                item_id: "live-movie",
                library_id: "library-a",
                file_name: "Live Movie.mkv",
                fingerprint: null,
                size_bytes: 1024,
              },
              item: publicMediaItem(),
              probe: null,
            },
          ],
        })
      }

      return jsonResponse({ message: "not found" }, 404)
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.getLibraryReadiness("library-a")
    const calledTargets = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      library: {
        id: "library-a",
        name: "Live Library",
        domain: "video",
        preset: "movies",
      },
      sources: [
        {
          source: {
            id: "source-1",
            libraryId: "library-a",
          },
          item: {
            id: "live-movie",
            title: "Live Movie",
          },
        },
      ],
      itemBrowse: {
        id: "library-scoped-item-browse",
        status: "ready",
      },
    })
    expect(calledTargets).toEqual([
      "/libraries/library-a",
      "/libraries/library-a/sources?limit=20&offset=0",
    ])
    expect(calledTargets.some((target) => target.includes("library_id="))).toBe(false)
  })

  it("lists library items through the scoped Public Client browse route", async () => {
    const fetcher = vi.fn<FetchLike>(async (input) => {
      const url = new URL(String(input))

      expect(url.pathname).toBe("/libraries/library-a/items")

      return jsonResponse({
        library: publicLibrary(),
        page,
        items: [publicMediaItem()],
      })
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.listLibraryItems("library-a", {
      limit: 25,
      offset: 50,
      sort: "date_added",
      order: "desc",
      facet: "kind:movie",
      watchState: "unwatched",
    })
    const calledUrl = new URL(String(fetcher.mock.calls[0][0]))

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      readiness: [],
      page: {
        limit: 10,
        offset: 0,
        returned: 1,
      },
      items: [
        {
          id: "live-movie",
          title: "Live Movie",
        },
      ],
    })
    expect(`${calledUrl.pathname}${calledUrl.search}`).toBe(
      "/libraries/library-a/items?limit=25&offset=50&sort=date_added&order=desc&facet=kind%3Amovie&watch_state=unwatched",
    )
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("maps continue-watching playback state through the Public Client", async () => {
    const fetcher = vi.fn<FetchLike>(async (input) => {
      const url = new URL(String(input))

      expect(url.pathname).toBe("/users/me/playback-state/continue-watching")
      expect(url.searchParams.get("limit")).toBe("12")

      return jsonResponse({
        page,
        items: [
          {
            item: publicMediaItem({ id: "live-movie" }),
            images: [],
            state: {
              item_id: "live-movie",
              source_id: "source-1",
              resume_position_ms: 120000,
              duration_ms: 600000,
              progress_percent: 20,
              watched: false,
              watched_at: null,
              last_played_at: "2026-05-28T10:00:00Z",
              updated_at: "2026-05-28T10:01:00Z",
              version: 3,
            },
          },
        ],
      })
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.listContinueWatching()

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      items: [
        {
          item: {
            id: "live-movie",
            title: "Live Movie",
          },
          state: {
            itemId: "live-movie",
            sourceId: "source-1",
            resumePositionMs: 120000,
            durationMs: 600000,
            progressPercent: 20,
            watched: false,
            version: 3,
          },
        },
      ],
    })
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("writes playback progress and watched state through the Public Client", async () => {
    const calls: Array<{ path: string; body: unknown }> = []
    const fetcher = vi.fn<FetchLike>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ path: url.pathname, body })

      return jsonResponse({
        state: {
          item_id: "live-movie",
          source_id: body?.source_id ?? null,
          resume_position_ms: body?.position_ms ?? null,
          duration_ms: body?.duration_ms ?? null,
          progress_percent: body?.duration_ms ? (body.position_ms / body.duration_ms) * 100 : null,
          watched: body?.watched ?? false,
          watched_at: body?.watched ? "2026-05-28T10:00:00Z" : null,
          last_played_at: "2026-05-28T10:00:00Z",
          updated_at: "2026-05-28T10:00:00Z",
          version: 4,
        },
      })
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const progress = await source.updatePlaybackProgress("live-movie", {
      source_id: "source-1",
      position_ms: 300000,
      duration_ms: 600000,
    })
    const watched = await source.setWatchedState("live-movie", {
      watched: true,
      source_id: "source-1",
      position_ms: 600000,
      duration_ms: 600000,
    })

    expect(progress.state).toMatchObject({
      itemId: "live-movie",
      sourceId: "source-1",
      resumePositionMs: 300000,
      progressPercent: 50,
      watched: false,
    })
    expect(watched.state).toMatchObject({
      watched: true,
      watchedAt: "2026-05-28T10:00:00Z",
    })
    expect(calls).toEqual([
      {
        path: "/users/me/playback-state/items/live-movie/progress",
        body: {
          source_id: "source-1",
          position_ms: 300000,
          duration_ms: 600000,
        },
      },
      {
        path: "/users/me/playback-state/items/live-movie/watched",
        body: {
          watched: true,
          source_id: "source-1",
          position_ms: 600000,
          duration_ms: 600000,
        },
      },
    ])
  })

  it("sends playback heartbeat through the Public Client session route", async () => {
    const calls: Array<{ path: string; body: unknown }> = []
    const fetcher = vi.fn<FetchLike>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ path: url.pathname, body })

      return jsonResponse({
        session: {
          id: "playback-session-1",
          source_id: "source-1",
          item_id: "live-movie",
          mode: "direct",
          state: body?.state ?? "active",
          position_ms: body?.position_ms ?? null,
          duration_ms: body?.duration_ms ?? null,
          started_at: "2026-05-28T10:00:00Z",
          updated_at: "2026-05-28T10:00:01Z",
          ended_at: null,
        },
      })
    })
    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    await source.heartbeatPlaybackSession("playback-session-1", {
      state: "active",
      position_ms: 12000,
      duration_ms: 120000,
    })

    expect(calls).toEqual([
      {
        path: "/playback/sessions/playback-session-1/heartbeat",
        body: {
          state: "active",
          position_ms: 12000,
          duration_ms: 120000,
        },
      },
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("builds browser-ticket playback plans with sidecar subtitle track URLs", async () => {
    const ticketBodies: unknown[] = []
    const fetcher = vi.fn<FetchLike>(async (input, init) => {
      const url = new URL(String(input))

      if (url.pathname === "/items/live-movie") {
        return jsonResponse({
          item: publicMediaItem({ id: "live-movie" }),
          sources: [
            {
              id: "source-1",
              item_id: "live-movie",
              library_id: "library-a",
              file_name: "Demo.mkv",
              fingerprint: null,
              size_bytes: 1024,
            },
          ],
          collections: [],
          credits: [],
          genres: [],
          images: [],
          studios: [],
          tags: [],
        })
      }

      if (url.pathname === "/sources/source-1/playback/decision") {
        expect(url.searchParams.get("supports_subtitles")).toBe("true")
        return jsonResponse({
          source: {
            id: "source-1",
            item_id: "live-movie",
            library_id: "library-a",
            file_name: "Demo.mkv",
            fingerprint: null,
            size_bytes: 1024,
          },
          probe: null,
          target: {
            kind: "browser",
            network_scope: "local",
            transport_auth: "ticket",
            control_capabilities: {
              can_pause: true,
              can_seek: true,
              can_set_volume: true,
              can_stop: true,
            },
            media_capabilities: {
              direct_play: true,
            },
          },
          decision: {
            mode: "direct_play",
            reason: "compatible",
            direct_play: {},
            transcode_plan: null,
            denial: null,
            report: {
              selected_mode: "direct_play",
              direct_play: { supported: true, conditions: ["compatible"] },
              remux: { supported: true, conditions: ["compatible"] },
              transcode: { supported: false, conditions: ["requested_transcode_output"] },
            },
          },
        })
      }

      if (url.pathname === "/sources/source-1/probe") {
        const disposition = {
          attached_pic: false,
          captions: false,
          commentary: false,
          default: true,
          descriptions: false,
          forced: false,
          hearing_impaired: false,
          visual_impaired: false,
        }

        return jsonResponse({
          source_id: "source-1",
          probe: {
            container: "matroska",
            duration_ms: 60000,
            bit_rate: null,
            streams: [
              {
                index: 0,
                kind: "video",
                codec: "h264",
                language: null,
                duration_ms: 60000,
                bit_rate: null,
                width: 1920,
                height: 1080,
                channels: null,
                sample_rate: null,
                disposition,
                origin: null,
              },
              {
                index: 2,
                kind: "subtitle",
                codec: "srt",
                language: "en",
                duration_ms: null,
                bit_rate: null,
                width: null,
                height: null,
                channels: null,
                sample_rate: null,
                disposition,
                origin: "sidecar",
              },
            ],
          },
        })
      }

      if (url.pathname === "/sources/source-1/playback/browser-ticket") {
        const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
        ticketBodies.push(body)
        if (body?.mode === "subtitle") {
          return jsonResponse({
            source_id: "source-1",
            item_id: "live-movie",
            playback_session_id: null,
            mode: "subtitle",
            expires_at: "2026-05-28T10:00:00Z",
            urls: [
              {
                kind: "subtitle",
                url: "/sources/source-1/subtitles/2?ticket=subtitle-ticket",
                content_type: "application/x-subrip; charset=utf-8",
                supports_range_requests: false,
              },
            ],
          })
        }

        return jsonResponse({
          source_id: "source-1",
          item_id: "live-movie",
          playback_session_id: "playback-session-1",
          mode: "direct",
          expires_at: "2026-05-28T10:00:00Z",
          urls: [
            {
              kind: "stream",
              url: "/sources/source-1/stream?ticket=video-ticket",
              content_type: "video/x-matroska",
              supports_range_requests: true,
            },
          ],
        })
      }

      return jsonResponse({ message: "not found" }, 404)
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const plan = await source.loadPlaybackPlan("live-movie", "movie", "source-1")

    expect(plan).toMatchObject({
      source: "live",
      fallback: false,
      sourceId: "source-1",
      playbackSessionId: "playback-session-1",
      mode: "direct",
      mediaUrl: "http://nako.test/sources/source-1/stream?ticket=video-ticket",
      subtitles: [
        {
          streamIndex: 2,
          language: "en",
          srcLang: "en",
          url: "http://nako.test/sources/source-1/subtitles/2?ticket=subtitle-ticket",
          contentType: "application/x-subrip; charset=utf-8",
          default: true,
        },
      ],
    })
    expect(plan.mediaUrl).not.toContain("public-token")
    expect(plan.subtitles[0].url).not.toContain("public-token")
    expect(ticketBodies).toEqual([
      {
        mode: "direct",
        capabilities: {
          direct_play: true,
          supports_subtitles: true,
          hls_variant_policy: "single_variant",
          hls_segment_container: "mpeg_ts",
        },
      },
      {
        mode: "subtitle",
        subtitle_stream_index: 2,
      },
    ])
  })
})

describe("public management context link contracts", () => {
  it("loads live Management Context Links through the Public Client", async () => {
    const fetcher = vi.fn<FetchLike>(async () =>
      jsonResponse(managementContextLinksResponse()),
    )
    const source = createPublicManagementContextDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.loadManagementContextLinks({
      libraryId: "library-a",
      itemId: "item-a",
      sourceId: "source-a",
      playbackSessionId: "session-a",
    })

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      context: {
        libraryId: "library-a",
        itemId: "item-a",
        sourceId: "source-a",
        playbackSessionId: "session-a",
      },
      links: [
        {
          routeName: "library.scan",
          action: "scan_library",
          enabled: true,
          method: "POST",
          requiredAccess: "library_manage",
          target: {
            libraryId: "library-a",
          },
        },
      ],
    })
    expect(fetcher.mock.calls[0][0]).toBe(
      "http://nako.test/management/context-links?library_id=library-a&item_id=item-a&source_id=source-a&playback_session_id=session-a",
    )
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("uses fixture Management Context Links when fixture mode is selected or live loading fails", async () => {
    const fixtureSource = createPublicManagementContextDataSource({ mode: "fixture" })
    const liveSource = createPublicManagementContextDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
      },
      vi.fn<FetchLike>(async () => jsonResponse({ message: "unavailable" }, 503)),
    )

    const fixturePayload = await fixtureSource.loadManagementContextLinks({
      libraryId: "library-a",
      itemId: "item-a",
    })
    const fallbackPayload = await liveSource.loadManagementContextLinks({
      libraryId: "library-a",
    })

    expect(fixturePayload).toMatchObject({
      source: "fixture",
      fallback: true,
      context: {
        libraryId: "library-a",
        itemId: "item-a",
      },
      links: expect.arrayContaining([
        expect.objectContaining({
          routeName: "library.scan",
          enabled: true,
        }),
        expect.objectContaining({
          routeName: "playback.support",
          enabled: false,
          disabledReason: "missing_context",
        }),
      ]),
    })
    expect(fallbackPayload).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "unavailable",
    })
  })

  it("omits unsafe Management Context Link query and target identifiers", async () => {
    const fetcher = vi.fn<FetchLike>(async () =>
      jsonResponse(
        managementContextLinksResponse({
          context: {
            library_id: "../private",
            item_id: "item-a",
            source_id: "file:///mnt/private/Movie.mkv",
            playback_session_id: "session-a",
          },
          links: [
            managementContextLink({
              target: {
                library_id: "../private",
                item_id: "item-a",
                source_id: "file:///mnt/private/Movie.mkv",
                playback_session_id: "session-a",
              },
            }),
          ],
        }),
      ),
    )
    const source = createPublicManagementContextDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
      },
      fetcher,
    )

    const payload = await source.loadManagementContextLinks({
      libraryId: "../private",
      itemId: "item-a",
      sourceId: "file:///mnt/private/Movie.mkv",
      playbackSessionId: "session-a",
    })

    expect(fetcher.mock.calls[0][0]).toBe(
      "http://nako.test/management/context-links?item_id=item-a&playback_session_id=session-a",
    )
    expect(payload.context).toEqual({
      itemId: "item-a",
      playbackSessionId: "session-a",
    })
    expect(payload.links[0]?.target).toEqual({
      itemId: "item-a",
      playbackSessionId: "session-a",
    })
  })

  it("resolves known Management Context Link routes and filters unknown route names", () => {
    const knownRoutes: Array<{
      link: PublicManagementContextLink
      expected: ReturnType<typeof resolveManagementContextLink>
    }> = [
      {
        link: publicManagementContextLink({
          routeName: "library.scan",
          action: "scan_library",
          method: "POST",
        }),
        expected: {
          routeName: "library.scan",
          action: "scan_library",
          path: "/admin/libraries",
          search: {
            library_id: "library-a",
            intent: "scan_library",
          },
        },
      },
      {
        link: publicManagementContextLink({
          routeName: "library.metadata_profile",
          action: "update_library_metadata_profile",
          method: "GET",
        }),
        expected: {
          routeName: "library.metadata_profile",
          action: "update_library_metadata_profile",
          path: "/admin/libraries",
          search: {
            library_id: "library-a",
            panel: "metadata_profile",
          },
        },
      },
      {
        link: publicManagementContextLink({
          routeName: "item.metadata_refresh",
          action: "refresh_item_metadata",
          method: "POST",
        }),
        expected: {
          routeName: "item.metadata_refresh",
          action: "refresh_item_metadata",
          path: "/admin/libraries",
          search: {
            library_id: "library-a",
            item_id: "item-a",
            source_id: "source-a",
            playback_session_id: "session-a",
            intent: "refresh_item_metadata",
          },
        },
      },
      {
        link: publicManagementContextLink({
          routeName: "jobs.filtered",
          action: "view_jobs",
          method: "GET",
        }),
        expected: {
          routeName: "jobs.filtered",
          action: "view_jobs",
          path: "/admin/tasks",
          search: {
            context: "management_link",
            library_id: "library-a",
            item_id: "item-a",
            source_id: "source-a",
            playback_session_id: "session-a",
          },
        },
      },
      {
        link: publicManagementContextLink({
          routeName: "playback.support",
          action: "view_playback_diagnostics",
          method: "GET",
        }),
        expected: {
          routeName: "playback.support",
          action: "view_playback_diagnostics",
          path: "/admin/transcoding",
          search: {
            library_id: "library-a",
            item_id: "item-a",
            source_id: "source-a",
            playback_session_id: "session-a",
            panel: "support",
          },
        },
      },
      {
        link: publicManagementContextLink({
          routeName: "playback.runtime",
          action: "view_playback_runtime",
          method: "GET",
        }),
        expected: {
          routeName: "playback.runtime",
          action: "view_playback_runtime",
          path: "/admin/transcoding",
          search: {
            panel: "runtime",
            library_id: "library-a",
            item_id: "item-a",
            source_id: "source-a",
            playback_session_id: "session-a",
          },
        },
      },
      {
        link: publicManagementContextLink({
          routeName: "access.library_policies",
          action: "manage_library_access",
          method: "GET",
        }),
        expected: {
          routeName: "access.library_policies",
          action: "manage_library_access",
          path: "/admin/users",
          search: {
            library_id: "library-a",
            panel: "library_access",
          },
        },
      },
    ]
    const links: PublicManagementContextLink[] = [
      ...knownRoutes.map((route) => route.link),
      publicManagementContextLink({
        routeName: "unknown.future_route",
        action: "view_jobs",
        method: "GET",
      }),
    ]

    for (const { link, expected } of knownRoutes) {
      expect(resolveManagementContextLink(link)).toEqual(expected)
    }
    expect(resolveManagementContextLink(links[7])).toBeNull()
    expect(resolveManagementContextLinks(links).map((target) => target.routeName)).toEqual([
      "library.scan",
      "library.metadata_profile",
      "item.metadata_refresh",
      "jobs.filtered",
      "playback.support",
      "playback.runtime",
      "access.library_policies",
    ])
  })

  it("does not resolve disabled or unsafe Management Context Link route targets", () => {
    expect(
      resolveManagementContextLink(
        publicManagementContextLink({
          enabled: false,
          disabledReason: "insufficient_permission",
        }),
      ),
    ).toBeNull()
    expect(
      resolveManagementContextLink(
        publicManagementContextLink({
          target: {
            libraryId: "../private",
            itemId: "item-a",
            sourceId: "source-a",
            playbackSessionId: "session-a",
          },
        }),
      ),
    ).toBeNull()
  })
})

describe("admin dashboard data source contracts", () => {
  it("uses the dashboard fixture when configured for fixture mode", async () => {
    const source = createAdminDashboardDataSource({ mode: "fixture" })

    await expect(source.loadDashboard()).resolves.toBe(ADMIN_DASHBOARD_FIXTURE)
  })

  it("maps live Admin API responses into dashboard data", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))

      switch (url.pathname) {
        case "/admin/v1/overview":
          return jsonResponse(adminOverviewResponse())
        case "/admin/v1/jobs":
          return jsonResponse(adminJobsResponse())
        case "/admin/v1/playback/sessions":
          return jsonResponse(adminPlaybackSessionsResponse())
        case "/admin/v1/playback/runtime":
          return jsonResponse(adminPlaybackRuntimeResponse())
        case "/admin/v1/system/config":
          return jsonResponse(adminSystemConfigResponse())
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })

    const source = createAdminDashboardDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const dashboard = await source.loadDashboard()

    expect(dashboard).toMatchObject({
      source: "live",
      fallback: false,
      metrics: {
        storage: 75,
        totalLibraries: 7,
        activeStreams: 1,
        version: "Admin v1",
        latestVersion: "Public v1",
      },
      activeTasks: [
        {
          id: "job-1",
          type: "library_scan",
          status: "running",
          library: "library-a",
          progress: 50,
          startedAt: "2026-05-28T10:01:00Z",
        },
      ],
      playbackSessions: [
        {
          id: "session-1",
          user: "user-1",
          item: "item-1",
          playbackMethod: "direct_play",
          progress: 0,
          quality: "playing",
        },
      ],
    })

    const calledTargets = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })
    expect(calledTargets).toEqual([
      "/admin/v1/overview",
      "/admin/v1/jobs?limit=3&offset=0",
      "/admin/v1/playback/sessions?limit=5&offset=0",
      "/admin/v1/playback/runtime",
      "/admin/v1/system/config",
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )
  })

  it("falls back to the dashboard fixture when a live Admin API request fails", async () => {
    const fetcher = vi.fn<typeof fetch>(async () => {
      throw new Error("admin offline")
    })

    const source = createAdminDashboardDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test",
      },
      fetcher,
    )

    const dashboard = await source.loadDashboard()

    expect(dashboard).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "admin offline",
    })
    expect(dashboard.metrics).toBe(ADMIN_DASHBOARD_FIXTURE.metrics)
  })
})

describe("admin read model data source contracts", () => {
  it("uses explicit fixtures for deeper Admin pages in fixture mode", async () => {
    const source = createAdminReadModelsDataSource({ mode: "fixture" })

    await expect(source.loadLibraries()).resolves.toBe(ADMIN_LIBRARY_READ_MODEL_FIXTURE)
    await expect(source.loadUsers()).resolves.toBe(ADMIN_USERS_READ_MODEL_FIXTURE)
    await expect(source.loadTasks()).resolves.toBe(ADMIN_TASKS_READ_MODEL_FIXTURE)
    await expect(source.loadLogs()).resolves.toBe(ADMIN_LOGS_READ_MODEL_FIXTURE)
    await expect(source.loadAcquisitionIntake()).resolves.toBe(
      ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE,
    )
    await expect(source.loadGeneratedArtifacts()).resolves.toBe(
      ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE,
    )
    await expect(source.loadSettings()).resolves.toBe(ADMIN_SETTINGS_READ_MODEL_FIXTURE)
  })

  it("maps live Admin acquisition intake candidates into a redacted read model", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))
      if (url.pathname === "/admin/v1/acquisition/intake/candidates") {
        return jsonResponse(adminAcquisitionIntakeCandidatesResponse())
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const intake = await source.loadAcquisitionIntake({
      library_id: "library-a",
      state: "ready",
      source_kind: "watch_folder",
      managed_import_artifact_id: "artifact-1",
      limit: 25,
      offset: 50,
    })

    expect(intake).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      query: {
        library_id: "library-a",
        state: "ready",
        source_kind: "watch_folder",
        managed_import_artifact_id: "artifact-1",
        limit: 25,
        offset: 50,
      },
      candidates: [
        {
          id: "candidate-1",
          targetLibraryId: "library-a",
          sourceKind: "watch_folder",
          sourceSummary: "file://<redacted>/Movie.mkv",
          managedImportArtifactId: "artifact-1",
          state: "ready",
          readiness: {
            hasDisplayName: true,
            hasIntendedLocator: true,
            hasFingerprint: true,
            hasDiagnostics: true,
          },
        },
      ],
    })

    const calledTarget = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })
    expect(calledTarget).toEqual([
      [
        "/admin/v1/acquisition/intake/candidates",
        "?library_id=library-a&state=ready&source_kind=watch_folder",
        "&managed_import_artifact_id=artifact-1&limit=25&offset=50",
      ].join(""),
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )

    const serialized = JSON.stringify(intake)
    expect(serialized).not.toContain("/mnt/private/raw")
    expect(serialized).not.toContain("unsafe prompt body")
  })

  it("maps live Admin generated artifact proposals into a redacted read model", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))
      if (url.pathname === "/admin/v1/automation/generated-artifacts/proposals") {
        return jsonResponse(adminGeneratedArtifactProposalsResponse())
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const artifacts = await source.loadGeneratedArtifacts({
      limit: 25,
      offset: 50,
    })

    expect(artifacts).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      query: {
        limit: 25,
        offset: 50,
      },
      proposals: [
        {
          id: "artifact-live",
          kind: "metadata_suggestion",
          capability: "item_metadata_suggest",
          status: "pending_review",
          target: {
            kind: "media_item",
            libraryId: "library-a",
            itemId: "item-live",
            sourceId: "source-live",
          },
          provenance: {
            providerId: "provider-live",
            providerName: "Live Automation Provider",
            jobId: "job-live",
            idempotencyKeyFingerprint: "sha256:idempotency-live",
            promptFingerprint: "sha256:prompt-live",
            attemptCount: 2,
          },
          payload: {
            validJson: true,
            shape: "object",
            payloadFingerprint: "sha256:payload-live",
            payloadBytes: 4096,
            objectFieldCount: 9,
            hasTextualValues: true,
            hasExplanation: true,
            confidenceMilli: 910,
          },
          readiness: {
            status: "ready",
            actionable: true,
            reasons: ["ready_for_review"],
          },
        },
      ],
    })

    const calledTarget = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })
    expect(calledTarget).toEqual([
      "/admin/v1/automation/generated-artifacts/proposals?limit=25&offset=50",
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )

    const serialized = JSON.stringify(artifacts)
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("artifact_storage_handle")
  })

  it("maps live Admin generated artifact review plans through POST without unsafe fields", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/review-plan") {
        return jsonResponse(adminGeneratedArtifactReviewPlanResponse("artifact/unsafe id", "reject"))
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const plan = await source.loadGeneratedArtifactReviewPlan("artifact/unsafe id", "reject")

    expect(plan).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      artifactId: "artifact/unsafe id",
      decision: "reject",
      status: "ready",
      action: "reject_generated_artifact",
      target: {
        kind: "media_item",
        libraryId: "library-a",
        itemId: "item-live",
        sourceId: "source-live",
      },
      payload: {
        payloadFingerprint: "sha256:payload-live",
        payloadBytes: 4096,
        confidenceMilli: 910,
      },
      readiness: {
        actionable: true,
        reasons: ["ready_for_review"],
      },
      boundary: {
        acceptedIntoCanonicalMetadata: false,
        writesSidecar: false,
        writesLibraryFiles: false,
        appliesImmediately: false,
        requiresMetadataAuthorityApply: false,
      },
    })

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/review-plan",
        body: { decision: "reject" },
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(plan)
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("artifact_storage_handle")
  })

  it("returns deterministic fixture generated artifact review plans", async () => {
    const source = createAdminReadModelsDataSource({ mode: "fixture" })

    await expect(
      source.loadGeneratedArtifactReviewPlan("fixture-generated-artifact-1", "accept"),
    ).resolves.toEqual(ADMIN_GENERATED_ARTIFACT_REVIEW_PLAN_FIXTURE)
  })

  it("maps live Admin metadata candidate review detail without unsafe fields", async () => {
    const calls: Array<{ method: string; path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/metadata/candidate-reviews/review%2Funsafe%20id") {
        return jsonResponse(adminMetadataCandidateReviewResponse("review/unsafe id"))
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const detail = await source.loadMetadataCandidateReview("review/unsafe id")

    expect(detail).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      reviewId: "review/unsafe id",
      itemId: "item-live",
      status: "accepted",
      sourceKey: "bangumi:1437",
      root: {
        kind: "series",
        subject: {
          provider: "bangumi",
          subjectKind: "subject",
          subjectKey: "1437",
          title: "Live Candidate",
        },
        metadata: {
          title: "Live Candidate",
          descriptionPresent: true,
          tagCount: 1,
        },
      },
      related: [
        expect.objectContaining({
          kind: "episode",
          subject: expect.objectContaining({
            subjectKey: "1437/1",
          }),
        }),
      ],
      relationships: [
        expect.objectContaining({
          kind: "contains",
          childSubject: expect.objectContaining({
            subjectKey: "1437/1",
          }),
        }),
      ],
      applicationPlan: {
        action: "apply",
        reasons: ["ready"],
        existingMappingId: null,
        existingMappingStatus: null,
      },
      boundary: {
        applyMutationRequired: true,
        applyUpdatesRootProviderSubject: true,
        applyUpdatesRootProviderMapping: true,
        applyUpdatesRelatedProviderSubjects: false,
        updatesHierarchy: false,
        updatesCanonicalMetadata: false,
      },
    })

    expect(calls).toEqual([
      {
        method: "GET",
        path: "/admin/v1/metadata/candidate-reviews/review%2Funsafe%20id",
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(detail)
    expect(serialized).not.toContain("secret candidate overview")
    expect(serialized).not.toContain("secret related overview")
    expect(serialized).not.toContain("secret-candidate-tag")
    expect(serialized).not.toContain("local:///")
    expect(serialized).not.toContain("sha256-private")
    expect(serialized).not.toContain("candidate-review:operator-secret")
    expect(serialized).not.toContain("provider secret response")
  })

  it("maps live Admin metadata candidate review lists into redacted navigation rows", async () => {
    const calls: Array<{ method: string; path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        method: init?.method ?? "GET",
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/metadata/items/item%2Funsafe%20id/candidate-reviews") {
        return jsonResponse(adminMetadataCandidateReviewListResponse("item/unsafe id"))
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const list = await source.loadMetadataCandidateReviewsForItem("item/unsafe id", {
      limit: 25,
      offset: 50,
    })

    expect(list).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      itemId: "item/unsafe id",
      page: {
        limit: 25,
        offset: 50,
        returned: 2,
      },
      reviews: [
        {
          reviewId: "review-live-newer",
          itemId: "item/unsafe id",
          status: "pending",
          sourceLabel: "bangumi",
          sourceKey: "bangumi:newer",
          root: {
            metadata: {
              title: "Newer Candidate",
              descriptionPresent: true,
              tagCount: 1,
            },
          },
          relatedCount: 1,
          relationshipCount: 1,
          applicationAction: "skip",
          applicationReasons: ["review_status_not_accepted"],
        },
        {
          reviewId: "review-live-older",
          status: "accepted",
          applicationAction: "apply",
          applicationReasons: ["ready"],
        },
      ],
    })

    expect(calls).toEqual([
      {
        method: "GET",
        path: "/admin/v1/metadata/items/item%2Funsafe%20id/candidate-reviews?limit=25&offset=50",
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(list)
    expect(serialized).not.toContain("secret candidate overview")
    expect(serialized).not.toContain("secret related overview")
    expect(serialized).not.toContain("secret-candidate-tag")
    expect(serialized).not.toContain("local:///")
    expect(serialized).not.toContain("sha256-private")
    expect(serialized).not.toContain("provider secret response")
  })

  it("maps live Admin metadata candidate review global queues into redacted navigation rows", async () => {
    const calls: Array<{ method: string; path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        method: init?.method ?? "GET",
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/metadata/candidate-reviews") {
        return jsonResponse(adminMetadataCandidateReviewQueueResponse())
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const queue = await source.loadMetadataCandidateReviews({
      status: "accepted",
      provider: "bangumi",
      limit: 25,
      offset: 50,
    })

    expect(queue).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      page: {
        limit: 25,
        offset: 50,
        returned: 2,
      },
      reviews: [
        {
          reviewId: "review-live-newer",
          itemId: "item/unsafe id",
          status: "pending",
          sourceLabel: "bangumi",
          sourceKey: "bangumi:newer",
          applicationAction: "skip",
        },
        {
          reviewId: "review-live-older",
          itemId: "item-live-other",
          status: "accepted",
          applicationAction: "apply",
        },
      ],
    })

    expect(calls).toEqual([
      {
        method: "GET",
        path: "/admin/v1/metadata/candidate-reviews?status=accepted&provider=bangumi&limit=25&offset=50",
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(queue)
    expect(serialized).not.toContain("secret candidate overview")
    expect(serialized).not.toContain("secret related overview")
    expect(serialized).not.toContain("secret-candidate-tag")
    expect(serialized).not.toContain("local:///")
    expect(serialized).not.toContain("sha256-private")
    expect(serialized).not.toContain("provider secret response")
  })

  it("returns deterministic fixture metadata candidate review detail", async () => {
    const source = createAdminReadModelsDataSource({ mode: "fixture" })

    await expect(
      source.loadMetadataCandidateReview("fixture-metadata-candidate-review-1"),
    ).resolves.toMatchObject({
      source: "fixture",
      fallback: true,
      reviewId: "fixture-metadata-candidate-review-1",
      root: {
        subject: {
          subjectKey: "1437",
        },
      },
      related: [
        expect.objectContaining({
          subject: expect.objectContaining({
            subjectKey: "1437/1",
          }),
        }),
      ],
      applicationPlan: {
        action: "apply",
      },
    })
  })

  it("maps live Admin generated artifact metadata apply plans through POST without unsafe fields", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/metadata-apply-plan") {
        return jsonResponse(adminGeneratedArtifactMetadataApplyPlanResponse("artifact/unsafe id"))
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const plan = await source.loadGeneratedArtifactMetadataApplyPlan("artifact/unsafe id")

    expect(plan).toMatchObject({
      source: "live",
      fallback: false,
      versions: {
        adminApi: "v1",
        publicApi: "v1",
      },
      artifactId: "artifact/unsafe id",
      status: "ready",
      executable: true,
      reasons: ["accepted_generated_artifact"],
      applyFieldCount: 1,
      skippedFieldCount: 1,
      noopFieldCount: 1,
      applyProviderMappingCount: 1,
      skippedProviderMappingCount: 0,
      noopProviderMappingCount: 1,
      target: {
        kind: "media_item",
        libraryId: "library-a",
        itemId: "item-live",
        sourceId: "source-live",
      },
      payload: {
        payloadFingerprint: "sha256:payload-live",
        payloadBytes: 4096,
        confidenceMilli: 910,
      },
      fields: expect.arrayContaining([
        expect.objectContaining({
          field: "title",
          action: "apply",
          reasons: ["incoming_differs"],
          current: {
            present: true,
            empty: false,
            valueFingerprint: "sha256:current-title",
            valueBytes: 12,
            itemCount: null,
          },
          incoming: {
            present: true,
            empty: false,
            valueFingerprint: "sha256:incoming-title",
            valueBytes: 16,
            itemCount: null,
          },
        }),
      ]),
      providerMappings: expect.arrayContaining([
        expect.objectContaining({
          action: "apply",
          reasons: ["incoming_provider_subject"],
          confidenceMilli: 910,
          existingMappingStatus: null,
          subject: {
            provider: "tmdb",
            providerName: "TMDB",
            subjectKind: "movie",
            subjectKindName: "Movie",
            subjectKey: "tmdb-123",
            title: "Live Movie",
            releaseYear: 2026,
            locale: "zh-CN",
          },
        }),
        expect.objectContaining({
          action: "noop",
          existingMappingStatus: "accepted",
          subject: expect.objectContaining({
            provider: "tmdb",
            subjectKind: "collection",
            subjectKey: "tmdb-collection-9",
          }),
        }),
      ]),
    })

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/metadata-apply-plan",
        body: undefined,
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(plan)
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe current title")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("artifact_storage_handle")
  })

  it("returns deterministic fixture generated artifact metadata apply plans", async () => {
    const source = createAdminReadModelsDataSource({ mode: "fixture" })

    await expect(
      source.loadGeneratedArtifactMetadataApplyPlan("fixture-generated-artifact-1"),
    ).resolves.toEqual(ADMIN_GENERATED_ARTIFACT_METADATA_APPLY_PLAN_FIXTURE)
  })

  it("maps live Admin generated artifact apply recovery without unsafe fields", async () => {
    const calls: Array<{ method: string; path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        method: init?.method ?? "GET",
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifact-apply-recovery") {
        return jsonResponse(adminGeneratedArtifactMetadataApplyRecoveryResponse())
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const recovery = await source.loadGeneratedArtifactApplyRecovery({
      attention: "needs_repair",
      limit: 25,
      offset: 50,
    })

    expect(recovery).toMatchObject({
      source: "live",
      fallback: false,
      summary: {
        returnedEntryCount: 1,
        needsRepairCount: 1,
      },
      entries: [
        expect.objectContaining({
          artifactId: "artifact/unsafe id",
          outcomeId: "metadata-apply-outcome-live",
          attention: "needs_repair",
          reason: "apply_outcome_failed",
          outcomeStatus: "failed",
          errorCode: "target_stale",
          plan: expect.objectContaining({
            artifactId: "artifact/unsafe id",
            providerMappings: expect.arrayContaining([
              expect.objectContaining({
                subject: expect.objectContaining({
                  provider: "tmdb",
                  subjectKey: "tmdb-123",
                }),
              }),
            ]),
          }),
        }),
      ],
      page: {
        limit: 25,
        offset: 50,
        returned: 1,
      },
    })

    expect(calls).toEqual([
      {
        method: "GET",
        path: "/admin/v1/automation/generated-artifact-apply-recovery?attention=needs_repair&limit=25&offset=50",
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(recovery)
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe current title")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("unsafe-recovery-idempotency")
  })

  it("maps live Admin generated artifact metadata bulk apply plans through POST without unsafe fields", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifacts/metadata-apply-plan") {
        return jsonResponse(
          adminGeneratedArtifactMetadataBulkApplyPlanResponse([
            "artifact/unsafe id",
            "artifact-missing",
          ]),
        )
      }

      return jsonResponse({ message: "not found" }, 404)
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const plan = await source.loadGeneratedArtifactMetadataBulkApplyPlan([
      "artifact/unsafe id",
      "artifact-missing",
    ])

    expect(plan).toMatchObject({
      source: "live",
      fallback: false,
      selection: {
        requestedArtifactCount: 2,
        selectedArtifactCount: 2,
      },
      summary: {
        plannedArtifactCount: 1,
        missingArtifactCount: 1,
        executableArtifactCount: 1,
        applyProviderMappingCount: 1,
        skippedProviderMappingCount: 0,
        noopProviderMappingCount: 1,
      },
      items: [
        expect.objectContaining({
          artifactId: "artifact/unsafe id",
          status: "planned",
          executable: true,
          plan: expect.objectContaining({
            artifactId: "artifact/unsafe id",
            applyFieldCount: 1,
            applyProviderMappingCount: 1,
            providerMappings: expect.arrayContaining([
              expect.objectContaining({
                action: "apply",
                subject: expect.objectContaining({
                  provider: "tmdb",
                  subjectKey: "tmdb-123",
                }),
              }),
            ]),
          }),
        }),
        expect.objectContaining({
          artifactId: "artifact-missing",
          status: "missing",
          executable: false,
          plan: null,
        }),
      ],
    })

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/metadata-apply-plan",
        body: { artifact_ids: ["artifact/unsafe id", "artifact-missing"] },
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify(plan)
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("artifact_storage_handle")
    expect(serialized).not.toContain("raw_payload")
  })

  it("returns deterministic fixture generated artifact metadata bulk apply plans", async () => {
    const source = createAdminReadModelsDataSource({ mode: "fixture" })

    await expect(
      source.loadGeneratedArtifactMetadataBulkApplyPlan(["fixture-generated-artifact-accepted-1"]),
    ).resolves.toMatchObject({
      source: "fixture",
      fallback: true,
      summary: {
        executableArtifactCount: 1,
      },
      items: [
        expect.objectContaining({
          artifactId: "fixture-generated-artifact-accepted-1",
          executable: true,
        }),
      ],
    })
  })

  it("maps live Admin API responses into deeper Admin page read models", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))

      switch (url.pathname) {
        case "/admin/v1/overview":
          return jsonResponse(adminReadModelOverviewResponse())
        case "/admin/v1/system/config":
          return jsonResponse(adminReadModelSystemConfigResponse())
        case "/admin/v1/access/summary":
          return jsonResponse(adminAccessSummaryResponse())
        case "/admin/v1/access/users":
          return jsonResponse(adminAccessUsersResponse())
        case "/admin/v1/playback/sessions":
          return jsonResponse(adminPlaybackSessionsResponse())
        case "/admin/v1/jobs":
          return jsonResponse(adminJobsResponse())
        case "/admin/v1/events":
          return jsonResponse(adminEventsResponse())
        case "/admin/v1/playback/runtime":
          return jsonResponse(adminPlaybackRuntimeFullResponse())
        case "/admin/v1/storage/staging":
          return jsonResponse(adminStorageStagingResponse())
        case "/admin/v1/settings/metadata/raw-cache":
          return jsonResponse(adminRawCacheSettingsResponse())
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })

    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const [libraries, users, tasks, logs, settings] = await Promise.all([
      source.loadLibraries(),
      source.loadUsers(),
      source.loadTasks(),
      source.loadLogs(),
      source.loadSettings(),
    ])

    expect(libraries).toMatchObject({
      source: "live",
      fallback: false,
      libraries: [
        {
          id: "library-a",
          name: "Movies",
          type: "movie",
          paths: [{ path: "file://local", available: true }],
          settings: {
            useNfo: true,
            downloadArt: true,
            metadataLanguage: "zh-CN",
          },
        },
      ],
    })
    expect(users).toMatchObject({
      source: "live",
      users: [
        {
          id: "user-1",
          role: "admin",
          status: "online",
          libraryAccess: ["all"],
        },
      ],
      activeSessions: [
        {
          id: "session-1",
          userId: "user-1",
          lastActivity: "正在播放: item-1",
        },
      ],
    })
    expect(tasks).toMatchObject({
      source: "live",
      tasks: [
        {
          id: "job-1",
          type: "scan",
          status: "running",
          progress: 50,
        },
      ],
      runningTask: {
        id: "job-1",
      },
      history: [
        {
          id: "event-1",
          status: "success",
        },
      ],
    })
    expect(logs.logs[0]).toMatchObject({
      id: "event-1",
      level: "warn",
      source: "scanner",
      requestId: "event-1",
    })
    expect(settings).toMatchObject({
      source: "live",
      general: {
        listenAddr: "0.0.0.0:8096",
        authEnabled: true,
      },
      metadata: {
        rawCacheRetentionMs: 86400000,
        enabledProviderCount: 1,
      },
      transcode: {
        hardwareAcceleration: "qsv",
        remuxConcurrency: 2,
      },
      storage: {
        stagingUsedBytes: 250,
      },
    })

    const calledTargets = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })
    expect(calledTargets).toEqual(
      expect.arrayContaining([
        "/admin/v1/access/users?limit=100&offset=0",
        "/admin/v1/events?limit=200&offset=0",
        "/admin/v1/settings/metadata/raw-cache",
        "/admin/v1/storage/staging?limit=100&offset=0",
      ]),
    )
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )
  })

  it("falls back to explicit Admin page fixtures when a live read model request fails", async () => {
    const fetcher = vi.fn<typeof fetch>(async () => {
      throw new Error("admin read model offline")
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test",
      },
      fetcher,
    )

    const libraries = await source.loadLibraries()

    expect(libraries).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "admin read model offline",
    })
    expect(libraries.libraries).toBe(ADMIN_LIBRARY_READ_MODEL_FIXTURE.libraries)
  })
})

describe("admin mutation data source contracts", () => {
  it("rejects mutations in fixture mode", async () => {
    const source = createAdminMutationDataSource({ mode: "fixture" })

    expect(source.canMutate).toBe(false)
    await expect(source.scanLibrary("library-a")).rejects.toThrow("live Admin API")
    await expect(source.reviewGeneratedArtifact("artifact-live", "accept")).rejects.toThrow(
      "live Admin API",
    )
    await expect(
      source.applyGeneratedArtifactMetadata(
        "artifact-live",
        "web-generated-artifact-metadata-apply:artifact-live:test",
      ),
    ).rejects.toThrow("live Admin API")
    await expect(
      source.confirmGeneratedArtifactMetadataBulkApplyBatch(
        ["artifact-live"],
        "web-generated-artifact-metadata-bulk-apply:test",
      ),
    ).rejects.toThrow("live Admin API")
    await expect(
      source.applyMetadataCandidateReview("review-live", {
        itemId: "item-live",
        expectedUpdatedAtMs: 300,
        idempotencyKey: "web-metadata-candidate-review-apply:review-live:test",
      }),
    ).rejects.toThrow("live Admin API")
  })

  it("maps accepted Admin mutations to versioned routes with JSON bodies", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const rawBody = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        body: rawBody,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${init?.method ?? "GET"} ${url.pathname}`) {
        case "POST /admin/v1/libraries/library-a/scan":
        case "POST /admin/v1/libraries/library-a/nfo/import":
        case "POST /admin/v1/libraries/library-a/nfo/export":
          return jsonResponse(adminJobsResponse().jobs[0])
        case "POST /admin/v1/access/users":
        case "PUT /admin/v1/access/users/user-1/roles":
        case "PATCH /admin/v1/access/users/user-1/status":
          return jsonResponse(adminAccessUserResponse())
        case "PUT /admin/v1/access/users/user-1/local-password":
        case "DELETE /admin/v1/access/users/user-1/local-password":
          return jsonResponse(adminLocalPasswordResponse())
        case "PUT /admin/v1/settings/metadata/raw-cache":
          return jsonResponse(adminRawCacheSettingsResponse())
        case "PATCH /admin/v1/addons/addon-1/status":
          return jsonResponse(adminAddonRegistrationResponse("disabled"))
        case "POST /admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/review":
          return jsonResponse(adminGeneratedArtifactReviewResponse("artifact/unsafe id", "accept"))
        case "POST /admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/metadata-apply":
          return jsonResponse(adminGeneratedArtifactMetadataApplyResponse("artifact/unsafe id"))
        case "POST /admin/v1/metadata/candidate-reviews/review%2Funsafe%20id/apply":
          return jsonResponse(adminMetadataCandidateReviewApplyResponse("review/unsafe id"))
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })

    const source = createAdminMutationDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    await expect(source.scanLibrary("library-a")).resolves.toMatchObject({
      kind: "library.scan",
      id: "job-1",
    })
    await expect(source.importLibraryNfo("library-a")).resolves.toMatchObject({
      kind: "library.nfo.import",
    })
    await expect(source.exportLibraryNfo("library-a")).resolves.toMatchObject({
      kind: "library.nfo.export",
    })
    await expect(
      source.createUser({ username: "new-user", display_name: "New User", roles: ["viewer"] }),
    ).resolves.toMatchObject({
      kind: "user.create",
      id: "user-1",
    })
    await source.replaceUserRoles("user-1", ["administrator"])
    await source.updateUserStatus("user-1", "disabled")
    await source.setUserLocalPassword("user-1", "secret")
    await source.deleteUserLocalPassword("user-1")
    await source.updateMetadataRawCacheSettings({
      retention_ms: 86400000,
      cleanup_on_startup: true,
    })
    await source.updateAddonStatus("addon-1", "disabled")
    const reviewResult = await source.reviewGeneratedArtifact("artifact/unsafe id", "accept")
    expect(reviewResult).toMatchObject({
      kind: "generated-artifact.review",
      artifactId: "artifact/unsafe id",
      decision: "accept",
      artifactStatus: "accepted",
      acceptedAt: "2026-05-29T01:10:00Z",
      idempotentReplay: true,
      plan: {
        artifactId: "artifact/unsafe id",
        decision: "accept",
        boundary: {
          requiresMetadataAuthorityApply: true,
        },
      },
    })
    const applyResult = await source.applyGeneratedArtifactMetadata(
      "artifact/unsafe id",
      "web-generated-artifact-metadata-apply:artifact-unsafe-id:test",
    )
    expect(applyResult).toMatchObject({
      kind: "generated-artifact.metadata-apply",
      artifactId: "artifact/unsafe id",
      outcomeId: "metadata-apply-outcome-live",
      status: "applied",
      applied: true,
      changed: true,
      idempotentReplay: true,
      appliedSource: "user",
      plan: {
        artifactId: "artifact/unsafe id",
        executable: true,
        fields: expect.arrayContaining([
          expect.objectContaining({
            field: "title",
            action: "apply",
          }),
        ]),
      },
    })
    const candidateApplyResult = await source.applyMetadataCandidateReview("review/unsafe id", {
      itemId: "item-live",
      expectedUpdatedAtMs: 300,
      idempotencyKey: "web-metadata-candidate-review-apply:review-unsafe-id:test",
    })
    expect(candidateApplyResult).toMatchObject({
      kind: "metadata-candidate-review.apply",
      reviewId: "review/unsafe id",
      itemId: "item-live",
      applied: true,
      changed: false,
      idempotentReplay: true,
      idempotencyKeyFingerprint: "0123456789abcdef",
      plan: {
        action: "noop",
        reasons: ["existing_accepted_mapping"],
      },
      providerSubject: {
        provider: "bangumi",
        subjectKind: "subject",
        subjectKey: "1437",
      },
      providerMapping: {
        status: "accepted",
      },
      boundary: {
        applyUpdatesRelatedProviderSubjects: false,
        updatesHierarchy: false,
      },
    })

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/admin/v1/libraries/library-a/scan",
        body: undefined,
        authorization: "Bearer admin-token",
      },
      {
        method: "POST",
        path: "/admin/v1/libraries/library-a/nfo/import",
        body: undefined,
        authorization: "Bearer admin-token",
      },
      {
        method: "POST",
        path: "/admin/v1/libraries/library-a/nfo/export",
        body: undefined,
        authorization: "Bearer admin-token",
      },
      {
        method: "POST",
        path: "/admin/v1/access/users",
        body: { username: "new-user", display_name: "New User", roles: ["viewer"] },
        authorization: "Bearer admin-token",
      },
      {
        method: "PUT",
        path: "/admin/v1/access/users/user-1/roles",
        body: { roles: ["administrator"] },
        authorization: "Bearer admin-token",
      },
      {
        method: "PATCH",
        path: "/admin/v1/access/users/user-1/status",
        body: { status: "disabled" },
        authorization: "Bearer admin-token",
      },
      {
        method: "PUT",
        path: "/admin/v1/access/users/user-1/local-password",
        body: { password: "secret" },
        authorization: "Bearer admin-token",
      },
      {
        method: "DELETE",
        path: "/admin/v1/access/users/user-1/local-password",
        body: undefined,
        authorization: "Bearer admin-token",
      },
      {
        method: "PUT",
        path: "/admin/v1/settings/metadata/raw-cache",
        body: { retention_ms: 86400000, cleanup_on_startup: true },
        authorization: "Bearer admin-token",
      },
      {
        method: "PATCH",
        path: "/admin/v1/addons/addon-1/status",
        body: { status: "disabled" },
        authorization: "Bearer admin-token",
      },
      {
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/review",
        body: { decision: "accept" },
        authorization: "Bearer admin-token",
      },
      {
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact%2Funsafe%20id/metadata-apply",
        body: {
          idempotency_key: "web-generated-artifact-metadata-apply:artifact-unsafe-id:test",
        },
        authorization: "Bearer admin-token",
      },
      {
        method: "POST",
        path: "/admin/v1/metadata/candidate-reviews/review%2Funsafe%20id/apply",
        body: {
          item_id: "item-live",
          expected_updated_at_ms: 300,
          idempotency_key: "web-metadata-candidate-review-apply:review-unsafe-id:test",
        },
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify({ reviewResult, applyResult, candidateApplyResult })
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe current title")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("artifact_storage_handle")
    expect(serialized).not.toContain("web-metadata-candidate-review-apply:review-unsafe-id:test")
  })

  it("maps live Admin metadata candidate review durable batch creation and status", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const rawBody = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        body: rawBody,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${init?.method ?? "GET"} ${url.pathname}`) {
        case "POST /admin/v1/metadata/candidate-reviews/batches":
          return jsonResponse(adminMetadataCandidateReviewBatchResponse(["review/unsafe id"], "queued"))
        case "GET /admin/v1/metadata/candidate-reviews/batches/candidate-review-batch-live":
          return jsonResponse(adminMetadataCandidateReviewBatchResponse(["review/unsafe id"], "completed"))
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })
    const connection = {
      mode: "live" as const,
      baseUrl: "http://nako-admin.test",
      bearerToken: "admin-token",
    }
    const mutationSource = createAdminMutationDataSource(connection, fetcher)
    const readSource = createAdminReadModelsDataSource(connection, fetcher)

    const confirmed = await mutationSource.createMetadataCandidateReviewBatch(
      [
        {
          reviewId: "review/unsafe id",
          itemId: "item-live",
          expectedUpdatedAtMs: 300,
        },
      ],
      "web-metadata-candidate-review-batch-apply:test",
    )
    const status = await readSource.loadMetadataCandidateReviewBatch("candidate-review-batch-live")

    expect(confirmed).toMatchObject({
      kind: "metadata-candidate-review.batch-create",
      id: "candidate-review-batch-live",
      jobId: "candidate-review-batch-job-live",
      status: "queued",
      selection: {
        selectedReviewCount: 1,
      },
      summary: {
        applyCount: 1,
      },
      executionSummary: {
        pendingItemCount: 1,
      },
    })
    expect(status).toMatchObject({
      source: "live",
      fallback: false,
      id: "candidate-review-batch-live",
      status: "completed",
      executionSummary: {
        appliedItemCount: 1,
        failedItemCount: 0,
      },
      items: [
        expect.objectContaining({
          reviewId: "review/unsafe id",
          itemId: "item-live",
          status: "applied",
          providerMappingId: "mapping-batch-live",
          error: null,
        }),
      ],
    })

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/admin/v1/metadata/candidate-reviews/batches",
        body: {
          reviews: [
            {
              review_id: "review/unsafe id",
              item_id: "item-live",
              expected_updated_at_ms: 300,
            },
          ],
          idempotency_key: "web-metadata-candidate-review-batch-apply:test",
        },
        authorization: "Bearer admin-token",
      },
      {
        method: "GET",
        path: "/admin/v1/metadata/candidate-reviews/batches/candidate-review-batch-live",
        body: undefined,
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify({ confirmed, status })
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("unsafe-batch-idempotency-key")
    expect(serialized).not.toContain("unsafe-item-idempotency-key")
    expect(serialized).not.toContain("web-metadata-candidate-review-batch-apply:test")
  })

  it("maps live Admin generated artifact metadata bulk apply confirmation and batch status", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const rawBody = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method: init?.method ?? "GET",
        path: url.pathname,
        body: rawBody,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${init?.method ?? "GET"} ${url.pathname}`) {
        case "POST /admin/v1/automation/generated-artifacts/metadata-apply-batches":
          return jsonResponse(
            adminGeneratedArtifactMetadataBulkApplyBatchResponse([
              "artifact/unsafe id",
              "artifact-missing",
            ]),
          )
        case "GET /admin/v1/automation/generated-artifacts/metadata-apply-batches/bulk-batch-live":
          return jsonResponse(
            adminGeneratedArtifactMetadataBulkApplyBatchResponse([
              "artifact/unsafe id",
              "artifact-missing",
            ]),
          )
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })
    const connection = {
      mode: "live" as const,
      baseUrl: "http://nako-admin.test",
      bearerToken: "admin-token",
    }
    const mutationSource = createAdminMutationDataSource(connection, fetcher)
    const readSource = createAdminReadModelsDataSource(connection, fetcher)

    const confirmed = await mutationSource.confirmGeneratedArtifactMetadataBulkApplyBatch(
      ["artifact/unsafe id", "artifact-missing"],
      "web-generated-artifact-metadata-bulk-apply:test",
    )
    const status = await readSource.loadGeneratedArtifactMetadataBulkApplyBatch("bulk-batch-live")

    expect(confirmed).toMatchObject({
      kind: "generated-artifact.metadata-bulk-apply",
      id: "bulk-batch-live",
      jobId: "bulk-job-live",
      status: "completed",
      executionSummary: {
        appliedItemCount: 1,
        skippedItemCount: 1,
      },
      items: [
        expect.objectContaining({
          artifactId: "artifact/unsafe id",
          status: "applied",
          outcomeId: "metadata-apply-outcome-live",
        }),
        expect.objectContaining({
          artifactId: "artifact-missing",
          status: "skipped",
          errorCode: "missing_artifact",
        }),
      ],
    })
    expect(status).toMatchObject({
      source: "live",
      fallback: false,
      id: "bulk-batch-live",
      status: "completed",
      summary: {
        executableArtifactCount: 1,
      },
    })

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/metadata-apply-batches",
        body: {
          artifact_ids: ["artifact/unsafe id", "artifact-missing"],
          idempotency_key: "web-generated-artifact-metadata-bulk-apply:test",
        },
        authorization: "Bearer admin-token",
      },
      {
        method: "GET",
        path: "/admin/v1/automation/generated-artifacts/metadata-apply-batches/bulk-batch-live",
        body: undefined,
        authorization: "Bearer admin-token",
      },
    ])

    const serialized = JSON.stringify({ confirmed, status })
    expect(serialized).not.toContain("F:\\private")
    expect(serialized).not.toContain("/mnt/private/source")
    expect(serialized).not.toContain("unsafe prompt body")
    expect(serialized).not.toContain("unsafe generated payload title")
    expect(serialized).not.toContain("provider secret response")
    expect(serialized).not.toContain("provider-secret")
    expect(serialized).not.toContain("unsafe-bulk-idempotency-key")
    expect(serialized).not.toContain("unsafe-item-idempotency")
    expect(serialized).not.toContain("raw_artifact_json")
  })
})

describe("admin addon manager data source contracts", () => {
  it("uses the addon manager fixture in fixture mode", async () => {
    const source = createAdminAddonManagerDataSource({ mode: "fixture" })

    await expect(source.loadAddonManager()).resolves.toBe(ADMIN_ADDON_MANAGER_FIXTURE)
  })

  it("maps live Admin Addon API responses into Addon Manager read models", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))

      switch (url.pathname) {
        case "/admin/v1/addons":
          return jsonResponse(adminAddonRegistrationsResponse())
        case "/admin/v1/addons/catalog/sources":
          return jsonResponse(adminAddonCatalogSourcesResponse())
        case "/admin/v1/addons/catalog/entries":
          return jsonResponse(adminAddonCatalogEntriesResponse())
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })

    const source = createAdminAddonManagerDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const manager = await source.loadAddonManager()

    expect(manager).toMatchObject({
      source: "live",
      fallback: false,
      installed: [
        {
          id: "addon-1",
          manifestId: "nako.tmdb",
          status: "enabled",
          grantedScopes: ["catalog_read", "item_metadata_read"],
        },
      ],
      catalog: [
        {
          entryId: "nako.tmdb",
          manifestId: "nako.tmdb",
          installedStatus: "enabled",
          lifecycleBoundary: {
            message: "Manual sidecar lifecycle boundary.",
          },
        },
      ],
      sources: [
        {
          id: "nako-official",
          entryCount: 1,
          providesProviderBreadth: true,
        },
      ],
    })
    expect(fetcher.mock.calls.map(([input]) => new URL(String(input)).pathname)).toEqual([
      "/admin/v1/addons",
      "/admin/v1/addons/catalog/sources",
      "/admin/v1/addons/catalog/entries",
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )
  })
})
