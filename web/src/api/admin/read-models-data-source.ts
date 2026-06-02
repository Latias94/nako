import { AdminApiClient } from "./client"
import { loadAdminApiConnection, type AdminApiConnection } from "./connection"
import type {
  AdminAccessSummaryResponse,
  AdminAccessUserRecord,
  AdminAcquisitionIntakeCandidateDiagnostic,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
  AdminGeneratedArtifactApplyRecoveryQuery,
  AdminGeneratedArtifactMetadataApplyFieldPlan,
  AdminGeneratedArtifactMetadataApplyOutcome,
  AdminGeneratedArtifactMetadataApplyOutcomeListResponse,
  AdminGeneratedArtifactMetadataApplyOutcomeResponse,
  AdminGeneratedArtifactMetadataApplyRecoveryEntry,
  AdminGeneratedArtifactMetadataApplyRecoveryResponse,
  AdminGeneratedArtifactMetadataApplyPlan,
  AdminGeneratedArtifactMetadataApplyPlanResponse,
  AdminGeneratedArtifactMetadataBulkApplyBatch,
  AdminGeneratedArtifactMetadataBulkApplyBatchItem,
  AdminGeneratedArtifactMetadataBulkApplyBatchResponse,
  AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummary,
  AdminGeneratedArtifactMetadataBulkApplyPlanItem,
  AdminGeneratedArtifactMetadataBulkApplyPlanResponse,
  AdminGeneratedArtifactMetadataBulkApplyPlanSelection,
  AdminGeneratedArtifactMetadataBulkApplyPlanSummary,
  AdminGeneratedArtifactMetadataValueSummary,
  AdminGeneratedArtifactProviderMappingPlan,
  AdminGeneratedArtifactProviderSubjectPlan,
  AdminGeneratedArtifactProposal,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
  AdminGeneratedArtifactReviewPlanResponse,
  AdminGeneratedArtifactReviewRequest,
  AdminGeneratedArtifactPayloadSummary,
  AdminGeneratedArtifactReadiness,
  AdminGeneratedArtifactTarget,
  AdminJobListItem,
  AdminMetadataCandidateReviewApplicationBoundary,
  AdminMetadataCandidateReviewApplicationPlan,
  AdminMetadataCandidateReviewApplyResponse,
  AdminMetadataCandidateReviewBatch,
  AdminMetadataCandidateReviewBatchExecutionSummary,
  AdminMetadataCandidateReviewBatchItem,
  AdminMetadataCandidateReviewBatchPlanSelection,
  AdminMetadataCandidateReviewBatchPlanSummary,
  AdminMetadataCandidateReviewBatchPlanResponse,
  AdminMetadataCandidateReviewBatchResponse,
  AdminMetadataCandidateReviewDetail,
  AdminMetadataCandidateReviewListEntry,
  AdminMetadataCandidateReviewListResponse,
  AdminMetadataCandidateReviewMetadataSummary,
  AdminMetadataCandidateReviewNode,
  AdminMetadataCandidateReviewProviderMapping,
  AdminMetadataCandidateReviewProviderSubject,
  AdminMetadataCandidateReviewQueueQuery,
  AdminMetadataCandidateReviewQueueResponse,
  AdminMetadataCandidateReviewRelationship,
  AdminMetadataCandidateReviewResponse,
  AdminMetadataCandidateSubject,
  AdminMetadataRawCacheSettingsResponse,
  AdminOutboxEventListItem,
  AdminOverviewResponse,
  AdminPageQuery,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListItem,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
  AdminUserRole,
} from "./generated/contract"

export type AdminReadModelSource = "live" | "fixture"

export interface AdminReadModelEnvelope {
  source: AdminReadModelSource
  fallback: boolean
  error?: string
}

export type AdminLibraryKind =
  | "movie"
  | "tv"
  | "anime"
  | "music"
  | "photo"
  | "documentary"
  | "personal"
  | "unknown"

export type AdminLibraryScanStatus = "idle" | "scanning" | "error"

export interface AdminLibraryReadModel {
  id: string
  name: string
  type: AdminLibraryKind
  paths: Array<{
    path: string
    available: boolean
  }>
  itemCount: number
  totalSize: string
  lastScanned: string
  scanStatus: AdminLibraryScanStatus
  scanProgress?: number
  errorMessage?: string
  settings: {
    autoScan: boolean
    scanInterval: number
    useNfo: boolean
    downloadArt: boolean
    metadataLanguage: string
  }
}

export interface AdminLibrariesReadModel extends AdminReadModelEnvelope {
  libraries: AdminLibraryReadModel[]
}

export type AdminUserUiRole = "admin" | "user" | "guest"
export type AdminUserUiStatus = "online" | "offline" | "disabled"

export interface AdminUserReadModel {
  id: string
  name: string
  username: string
  email: string | null
  role: AdminUserUiRole
  avatar: string | null
  status: AdminUserUiStatus
  lastActive: string
  createdAt: string
  libraryAccess: string[]
  settings: {
    canDownload: boolean
    canDelete: boolean
    canManageUsers: boolean
    maxBitrate: number | null
    transcodePolicy: string
    maxStreams: number | null
    remoteAccess: boolean
  }
  stats: {
    totalPlays: number
    totalWatchTime: string
    lastLogin: string
  }
}

export interface AdminActiveSessionReadModel {
  id: string
  userId: string
  userName: string
  device: string
  deviceType: string
  ip: string
  location: string
  lastActivity: string
  startTime: string
  current: boolean
}

export interface AdminLibraryOptionReadModel {
  id: string
  name: string
  type: AdminLibraryKind
}

export interface AdminUsersReadModel extends AdminReadModelEnvelope {
  users: AdminUserReadModel[]
  activeSessions: AdminActiveSessionReadModel[]
  libraries: AdminLibraryOptionReadModel[]
  accessMode: string
  accessCapabilities: {
    userAccounts: string
    roles: string
    libraryAccessPolicy: string
  }
}

export type AdminTaskType =
  | "scan"
  | "metadata"
  | "backup"
  | "cleanup"
  | "update"
  | "optimize"
  | "subtitle"
  | "thumbnail"

export type AdminTaskStatus = "idle" | "running" | "success" | "failed" | "scheduled"
export type AdminScheduleFrequency = "interval" | "daily" | "weekly" | "monthly" | "cron"

export interface AdminTaskConfig {
  targetLibraries?: string[]
  scanNewOnly?: boolean
  backupPath?: string
  keepBackups?: number
  includeImages?: boolean
  cacheMaxAge?: number
  minFreeSpace?: number
  cleanTranscodes?: boolean
  cleanImages?: boolean
  cleanLogs?: boolean
  metadataLanguage?: string
  refreshAll?: boolean
  downloadImages?: boolean
  subtitleLanguages?: string[]
  preferSDH?: boolean
  overwriteExisting?: boolean
  chapters?: boolean
}

export interface AdminScheduledTaskReadModel {
  id: string
  name: string
  description: string
  type: AdminTaskType
  enabled: boolean
  schedule: {
    frequency: AdminScheduleFrequency
    time?: string
    days?: number[]
    interval?: number
    cron?: string
  }
  config: AdminTaskConfig
  lastRun?: {
    timestamp: string
    status: "success" | "failed"
    duration: string
    itemsProcessed?: number
    error?: string
  }
  nextRun?: string
  status: AdminTaskStatus
  progress?: number
}

export interface AdminTaskHistoryReadModel {
  id: string
  taskId: string
  taskName: string
  taskType: AdminTaskType
  timestamp: string
  status: "success" | "failed"
  duration: string
  itemsProcessed?: number
  error?: string
}

export interface AdminTasksReadModel extends AdminReadModelEnvelope {
  tasks: AdminScheduledTaskReadModel[]
  runningTask: AdminScheduledTaskReadModel | null
  history: AdminTaskHistoryReadModel[]
  libraries: AdminLibraryOptionReadModel[]
}

export type AdminLogLevel = "error" | "warn" | "info" | "debug"
export type AdminLogSource = "server" | "auth" | "database" | "api" | "playback" | "scanner"

export interface AdminLogEntryReadModel {
  id: string
  timestamp: string
  level: AdminLogLevel
  source: AdminLogSource
  message: string
  details?: string
  userId?: string
  requestId?: string
}

export interface AdminLogsReadModel extends AdminReadModelEnvelope {
  logs: AdminLogEntryReadModel[]
}

export type AdminAcquisitionIntakeCandidateState =
  | "discovered"
  | "inspecting"
  | "ready"
  | "blocked"
  | "accepted"
  | "rejected"
  | "failed"
  | "superseded"
  | (string & {})

export type AdminAcquisitionIntakeSourceKind =
  | "watch_folder"
  | "operator_submitted"
  | "external_download_output"
  | "addon_proposed"
  | "resource_search_selection"
  | (string & {})

export interface AdminAcquisitionIntakeCandidateReadModel {
  id: string
  targetLibraryId: string
  sourceKind: AdminAcquisitionIntakeSourceKind
  customSourceKind: boolean
  sourceScheme: string | null
  sourceSummary: string
  sourceKeyFingerprint: string
  sizeBytes: number | null
  managedImportArtifactId: string | null
  state: AdminAcquisitionIntakeCandidateState
  readiness: {
    hasDisplayName: boolean
    hasIntendedLocator: boolean
    hasFingerprint: boolean
    hasDiagnostics: boolean
  }
  firstSeenAt: string
  lastSeenAt: string
  createdAt: string
  updatedAt: string
}

export interface AdminAcquisitionIntakeReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  query: AdminAcquisitionIntakeCandidatesQuery
  candidates: AdminAcquisitionIntakeCandidateReadModel[]
  page: AdminAcquisitionIntakeCandidateListResponse["page"]
}

export interface AdminGeneratedArtifactTargetReadModel {
  kind: string
  libraryId: string | null
  itemId: string | null
  sourceId: string | null
}

export interface AdminGeneratedArtifactProvenanceReadModel {
  providerId: string
  providerName: string | null
  jobId: string
  capability: string
  idempotencyKeyFingerprint: string | null
  promptFingerprint: string | null
  attemptCount: number | null
  artifactCreatedAt: string
}

export interface AdminGeneratedArtifactPayloadReadModel {
  validJson: boolean
  shape: string
  payloadFingerprint: string
  payloadBytes: number
  objectFieldCount: number | null
  arrayItemCount: number | null
  hasTextualValues: boolean
  hasExplanation: boolean
  confidenceMilli: number | null
}

export interface AdminGeneratedArtifactReadinessReadModel {
  status: string
  actionable: boolean
  reasons: string[]
}

export interface AdminGeneratedArtifactProposalReadModel {
  id: string
  kind: string
  capability: string
  status: string
  target: AdminGeneratedArtifactTargetReadModel
  provenance: AdminGeneratedArtifactProvenanceReadModel
  payload: AdminGeneratedArtifactPayloadReadModel
  readiness: AdminGeneratedArtifactReadinessReadModel
  createdAt: string
  updatedAt: string
  acceptedAt: string | null
}

export interface AdminGeneratedArtifactsReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  query: AdminGeneratedArtifactProposalsQuery
  proposals: AdminGeneratedArtifactProposalReadModel[]
  page: AdminGeneratedArtifactProposalListResponse["page"]
}

export type AdminGeneratedArtifactReviewDecision = AdminGeneratedArtifactReviewRequest["decision"]

export interface AdminGeneratedArtifactAcceptanceBoundaryReadModel {
  acceptedIntoCanonicalMetadata: boolean
  writesSidecar: boolean
  writesLibraryFiles: boolean
  appliesImmediately: boolean
  requiresMetadataAuthorityApply: boolean
}

export interface AdminGeneratedArtifactReviewPlanReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  artifactId: string
  decision: AdminGeneratedArtifactReviewDecision
  status: string
  action: string
  reasons: string[]
  capability: string
  kind: string
  target: AdminGeneratedArtifactTargetReadModel
  payload: AdminGeneratedArtifactPayloadReadModel
  readiness: AdminGeneratedArtifactReadinessReadModel
  boundary: AdminGeneratedArtifactAcceptanceBoundaryReadModel
}

export interface AdminGeneratedArtifactMetadataValueSummaryReadModel {
  present: boolean
  empty: boolean
  valueFingerprint: string | null
  valueBytes: number | null
  itemCount: number | null
}

export interface AdminGeneratedArtifactMetadataApplyFieldPlanReadModel {
  field: string
  action: string
  reasons: string[]
  current: AdminGeneratedArtifactMetadataValueSummaryReadModel
  incoming: AdminGeneratedArtifactMetadataValueSummaryReadModel
}

export interface AdminGeneratedArtifactProviderSubjectPlanReadModel {
  provider: string | null
  providerName: string | null
  subjectKind: string | null
  subjectKindName: string | null
  subjectKey: string | null
  title: string | null
  releaseYear: number | null
  locale: string | null
}

export interface AdminGeneratedArtifactProviderMappingPlanReadModel {
  subject: AdminGeneratedArtifactProviderSubjectPlanReadModel
  action: string
  reasons: string[]
  confidenceMilli: number | null
  existingMappingStatus: string | null
}

export interface AdminGeneratedArtifactMetadataApplyPlanReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  artifactId: string
  status: string
  executable: boolean
  reasons: string[]
  target: AdminGeneratedArtifactTargetReadModel
  payload: AdminGeneratedArtifactPayloadReadModel
  fields: AdminGeneratedArtifactMetadataApplyFieldPlanReadModel[]
  providerMappings: AdminGeneratedArtifactProviderMappingPlanReadModel[]
  applyFieldCount: number
  skippedFieldCount: number
  noopFieldCount: number
  applyProviderMappingCount: number
  skippedProviderMappingCount: number
  noopProviderMappingCount: number
}

export interface AdminGeneratedArtifactMetadataApplyOutcomeReadModel
  extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  id: string
  artifactId: string
  idempotencyKeyFingerprint: string
  status: string
  applied: boolean
  changed: boolean
  appliedSource: string | null
  itemId: string | null
  plan: AdminGeneratedArtifactMetadataApplyPlanReadModel
  errorCode: string | null
  errorMessage: string | null
  createdAt: string
  updatedAt: string
}

export interface AdminGeneratedArtifactMetadataApplyOutcomesReadModel
  extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  outcomes: AdminGeneratedArtifactMetadataApplyOutcomeReadModel[]
  page: AdminGeneratedArtifactMetadataApplyOutcomeListResponse["page"]
}

export interface AdminGeneratedArtifactMetadataApplyRecoveryEntryReadModel {
  source: string
  attention: string
  reason: string
  artifactId: string
  outcomeId: string | null
  batchId: string | null
  batchItemStatus: string | null
  outcomeStatus: string | null
  itemId: string | null
  plan: AdminGeneratedArtifactMetadataApplyPlanReadModel | null
  errorCode: string | null
  errorMessage: string | null
  createdAt: string
  updatedAt: string
}

export interface AdminGeneratedArtifactMetadataApplyRecoveryReadModel
  extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  summary: {
    returnedEntryCount: number
    needsRepairCount: number
    needsReviewCount: number
    replayOnlyCount: number
    resolvedCount: number
  }
  entries: AdminGeneratedArtifactMetadataApplyRecoveryEntryReadModel[]
  page: AdminGeneratedArtifactMetadataApplyRecoveryResponse["page"]
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanSelectionReadModel {
  requestedArtifactCount: number
  selectedArtifactCount: number
  duplicateArtifactCount: number
  maxArtifactCount: number
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanSummaryReadModel {
  plannedArtifactCount: number
  missingArtifactCount: number
  readyArtifactCount: number
  blockedArtifactCount: number
  staleArtifactCount: number
  executableArtifactCount: number
  applyFieldCount: number
  skippedFieldCount: number
  noopFieldCount: number
  applyProviderMappingCount: number
  skippedProviderMappingCount: number
  noopProviderMappingCount: number
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanItemReadModel {
  artifactId: string
  status: string
  executable: boolean
  reasons: string[]
  plan: AdminGeneratedArtifactMetadataApplyPlanReadModel | null
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  selection: AdminGeneratedArtifactMetadataBulkApplyPlanSelectionReadModel
  summary: AdminGeneratedArtifactMetadataBulkApplyPlanSummaryReadModel
  items: AdminGeneratedArtifactMetadataBulkApplyPlanItemReadModel[]
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummaryReadModel {
  totalItemCount: number
  pendingItemCount: number
  skippedItemCount: number
  appliedItemCount: number
  noopItemCount: number
  staleItemCount: number
  failedItemCount: number
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatchItemReadModel {
  artifactId: string
  position: number
  status: string
  outcomeId: string | null
  errorCode: string | null
  errorMessage: string | null
  planItem: AdminGeneratedArtifactMetadataBulkApplyPlanItemReadModel
  createdAt: string
  updatedAt: string
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatchReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  id: string
  jobId: string
  status: string
  selection: AdminGeneratedArtifactMetadataBulkApplyPlanSelectionReadModel
  summary: AdminGeneratedArtifactMetadataBulkApplyPlanSummaryReadModel
  executionSummary: AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummaryReadModel
  items: AdminGeneratedArtifactMetadataBulkApplyBatchItemReadModel[]
  createdAt: string
  updatedAt: string
}

export interface AdminMetadataCandidateSubjectReadModel {
  provider: string
  subjectKind: string
  subjectKey: string
  title: string | null
  releaseYear: number | null
  locale: string | null
}

export interface AdminMetadataCandidateReviewMetadataSummaryReadModel {
  title: string | null
  releaseDate: string | null
  descriptionPresent: boolean
  genreCount: number
  tagCount: number
  imageCount: number
}

export interface AdminMetadataCandidateReviewNodeReadModel {
  sourceLabel: string
  kind: string
  subject: AdminMetadataCandidateSubjectReadModel | null
  metadata: AdminMetadataCandidateReviewMetadataSummaryReadModel
}

export interface AdminMetadataCandidateReviewRelationshipReadModel {
  parentSubject: AdminMetadataCandidateSubjectReadModel
  childSubject: AdminMetadataCandidateSubjectReadModel
  kind: string
}

export interface AdminMetadataCandidateReviewApplicationPlanReadModel {
  reviewId: string
  itemId: string
  action: string
  reasons: string[]
  existingMappingId: string | null
  existingMappingStatus: string | null
}

export interface AdminMetadataCandidateReviewBoundaryReadModel {
  applyMutationRequired: boolean
  applyUpdatesRootProviderSubject: boolean
  applyUpdatesRootProviderMapping: boolean
  applyUpdatesRelatedProviderSubjects: boolean
  updatesCanonicalMetadata: boolean
  updatesHierarchy: boolean
}

export interface AdminMetadataCandidateReviewReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  reviewId: string
  itemId: string
  sourceLabel: string
  sourceKey: string
  status: string
  root: AdminMetadataCandidateReviewNodeReadModel
  related: AdminMetadataCandidateReviewNodeReadModel[]
  relationships: AdminMetadataCandidateReviewRelationshipReadModel[]
  relatedCount: number
  relationshipCount: number
  expiresAtMs: number | null
  createdAtMs: number
  updatedAtMs: number
  applicationPlan: AdminMetadataCandidateReviewApplicationPlanReadModel
  boundary: AdminMetadataCandidateReviewBoundaryReadModel
}

export interface AdminMetadataCandidateReviewListItemReadModel {
  reviewId: string
  itemId: string
  sourceLabel: string
  sourceKey: string
  status: string
  root: AdminMetadataCandidateReviewNodeReadModel
  relatedCount: number
  relationshipCount: number
  expiresAtMs: number | null
  createdAtMs: number
  updatedAtMs: number
  applicationAction: string
  applicationReasons: string[]
}

export interface AdminMetadataCandidateReviewListReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  itemId: string
  reviews: AdminMetadataCandidateReviewListItemReadModel[]
  page: {
    limit: number
    offset: number
    returned: number
  }
}

export interface AdminMetadataCandidateReviewQueueReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  reviews: AdminMetadataCandidateReviewListItemReadModel[]
  page: {
    limit: number
    offset: number
    returned: number
  }
}

export interface AdminMetadataCandidateReviewBatchPlanSummaryReadModel {
  requestedCount: number
  returnedCount: number
  maxReviewCount: number
  applyCount: number
  noopCount: number
  skipCount: number
}

export interface AdminMetadataCandidateReviewBatchPlanReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  summary: AdminMetadataCandidateReviewBatchPlanSummaryReadModel
  reviews: AdminMetadataCandidateReviewListItemReadModel[]
}

export interface AdminMetadataCandidateReviewProviderSubjectReadModel
  extends AdminMetadataCandidateSubjectReadModel {
  subjectId: string
}

export interface AdminMetadataCandidateReviewProviderMappingReadModel {
  mappingId: string
  itemId: string
  subjectId: string
  status: string
  confidenceMilli: number | null
  sourceLabel: string
}

export interface AdminMetadataCandidateReviewApplyReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  reviewId: string
  itemId: string
  applied: boolean
  changed: boolean
  idempotentReplay: boolean
  idempotencyKeyFingerprint: string
  plan: AdminMetadataCandidateReviewApplicationPlanReadModel
  providerSubject: AdminMetadataCandidateReviewProviderSubjectReadModel | null
  providerMapping: AdminMetadataCandidateReviewProviderMappingReadModel | null
  boundary: AdminMetadataCandidateReviewBoundaryReadModel
}

export interface AdminMetadataCandidateReviewBatchApplyErrorReadModel {
  code: string
  message: string
}

export interface AdminMetadataCandidateReviewBatchSelectionReadModel {
  requestedReviewCount: number
  selectedReviewCount: number
  duplicateReviewCount: number
  maxReviewCount: number
}

export interface AdminMetadataCandidateReviewBatchExecutionSummaryReadModel {
  totalItemCount: number
  pendingItemCount: number
  skippedItemCount: number
  blockedItemCount: number
  appliedItemCount: number
  noopItemCount: number
  staleItemCount: number
  conflictItemCount: number
  failedItemCount: number
}

export interface AdminMetadataCandidateReviewBatchItemReadModel {
  reviewId: string
  itemId: string
  position: number
  status: string
  providerSubjectId: string | null
  providerMappingId: string | null
  error: AdminMetadataCandidateReviewBatchApplyErrorReadModel | null
}

export interface AdminMetadataCandidateReviewBatchReadModel extends AdminReadModelEnvelope {
  versions: {
    adminApi: string
    publicApi: string
  }
  id: string
  jobId: string
  status: string
  selection: AdminMetadataCandidateReviewBatchSelectionReadModel
  summary: AdminMetadataCandidateReviewBatchPlanSummaryReadModel
  executionSummary: AdminMetadataCandidateReviewBatchExecutionSummaryReadModel
  items: AdminMetadataCandidateReviewBatchItemReadModel[]
}

export interface AdminSettingsReadModel extends AdminReadModelEnvelope {
  general: {
    serverName: string
    serverId: string
    listenAddr: string
    authEnabled: boolean
    adminApiVersion: string
    publicApiVersion: string
  }
  network: {
    exposureMode: string
    readinessStatus: string
    readinessReason: string
    externalEndpointConfigured: boolean
    allowedOriginCount: number
  }
  database: {
    configuredBackendKind: string
    activeBackendKind: string
    migratedOnStartup: boolean
    librariesSupported: boolean
  }
  metadata: {
    rawCacheRetentionMs: number
    rawCacheCleanupOnStartup: boolean
    providerCount: number
    enabledProviderCount: number
    language: string
  }
  transcode: {
    hardwareAcceleration: string
    hardwareFallbackUsed: boolean
    cpuConcurrency: number
    gpuConcurrency: number
    remuxConcurrency: number
    remuxTimeoutMs: number
  }
  storage: {
    stagingMaxBytes: number
    stagingUsedBytes: number
    stagingRetentionMs: number
    stagingCleanupOnStartup: boolean
    stagingRecordCount: number
  }
  runtime: {
    scanConcurrency: number
    probeConcurrency: number
    metadataConcurrency: number
    webhookConcurrency: number
  }
}

export const ADMIN_LIBRARY_READ_MODEL_FIXTURE: AdminLibrariesReadModel = {
  source: "fixture",
  fallback: true,
  libraries: [
    fixtureLibrary("1", "电影", "movie", "/media/movies", 847, "4.2 TB", "2024-03-15 14:30"),
    {
      ...fixtureLibrary("2", "剧集", "tv", "/media/tv", 156, "8.7 TB", "2024-03-15 14:35"),
      scanStatus: "scanning",
      scanProgress: 67,
    },
    fixtureLibrary("3", "动画", "anime", "/media/anime", 234, "2.1 TB", "2024-03-15 12:00"),
    {
      ...fixtureLibrary("4", "纪录片", "documentary", "/media/documentary", 89, "890 GB", "2024-03-14 22:00"),
      paths: [{ path: "/media/documentary", available: false }],
      scanStatus: "error",
      errorMessage: "路径不可访问: /media/documentary",
      settings: {
        autoScan: false,
        scanInterval: 24,
        useNfo: false,
        downloadArt: true,
        metadataLanguage: "zh-CN",
      },
    },
  ],
}

export const ADMIN_USERS_READ_MODEL_FIXTURE: AdminUsersReadModel = {
  source: "fixture",
  fallback: true,
  users: [
    fixtureUser("1", "管理员", "admin", "admin", "online", ["all"], "在线"),
    fixtureUser("2", "小明", "xiaoming", "user", "online", ["1", "2", "3"], "3 分钟前"),
    fixtureUser("3", "小红", "xiaohong", "user", "offline", ["1", "3"], "昨天 22:15"),
    fixtureUser("4", "访客", "guest", "guest", "offline", ["1"], "从未登录"),
    fixtureUser("5", "家人", "family", "user", "disabled", ["1", "2"], "账户已禁用"),
  ],
  activeSessions: [
    fixtureSession("s1", "1", "管理员", "Chrome - Windows", "desktop", "正在播放: 沙丘2", true),
    fixtureSession("s2", "2", "小明", "Nako iOS App", "mobile", "浏览媒体库", false),
    fixtureSession("s3", "2", "小明", "Samsung TV", "tv", "正在播放: 奥本海默", false),
  ],
  libraries: [
    { id: "1", name: "电影", type: "movie" },
    { id: "2", name: "剧集", type: "tv" },
    { id: "3", name: "动画", type: "anime" },
    { id: "4", name: "纪录片", type: "documentary" },
    { id: "5", name: "个人收藏", type: "personal" },
  ],
  accessMode: "fixture",
  accessCapabilities: {
    userAccounts: "planned",
    roles: "planned",
    libraryAccessPolicy: "planned",
  },
}

export const ADMIN_TASKS_READ_MODEL_FIXTURE: AdminTasksReadModel = {
  source: "fixture",
  fallback: true,
  tasks: [
    fixtureTask("1", "每日媒体库扫描", "扫描所有媒体库以发现新内容", "scan", "scheduled", true),
    fixtureTask("2", "元数据刷新", "更新最近添加项目的元数据", "metadata", "scheduled", true),
    fixtureTask("3", "数据库备份", "创建数据库和配置的备份", "backup", "scheduled", true),
    {
      ...fixtureTask("4", "转码缓存清理", "删除旧的转码缓存文件", "cleanup", "scheduled", true),
      lastRun: {
        timestamp: "2024-03-09T05:00:00Z",
        status: "failed",
        duration: "0分45秒",
        error: "任务执行失败: 权限不足或网络超时",
      },
    },
    fixtureTask("5", "自动更新检查", "检查服务器更新", "update", "idle", false),
    fixtureTask("6", "数据库优化", "优化数据库性能", "optimize", "scheduled", true),
    fixtureTask("7", "字幕自动下载", "为缺失字幕的视频下载字幕", "subtitle", "scheduled", true),
  ],
  runningTask: {
    ...fixtureTask("running-1", "手动媒体库扫描", "全媒体库扫描（手动触发）", "scan", "running", true),
    progress: 67,
  },
  history: buildFixtureHistory(),
  libraries: ADMIN_USERS_READ_MODEL_FIXTURE.libraries.slice(0, 4),
}

export const ADMIN_LOGS_READ_MODEL_FIXTURE: AdminLogsReadModel = {
  source: "fixture",
  fallback: true,
  logs: buildFixtureLogs(),
}

export const ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE: AdminAcquisitionIntakeReadModel =
  acquisitionIntakeFixture(normalizeAcquisitionIntakeQuery({}))

export const ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE: AdminGeneratedArtifactsReadModel =
  generatedArtifactsFixture(normalizeGeneratedArtifactsQuery({}))

export const ADMIN_GENERATED_ARTIFACT_APPLY_OUTCOMES_READ_MODEL_FIXTURE: AdminGeneratedArtifactMetadataApplyOutcomesReadModel =
  generatedArtifactApplyOutcomesFixture()

export const ADMIN_GENERATED_ARTIFACT_APPLY_RECOVERY_READ_MODEL_FIXTURE: AdminGeneratedArtifactMetadataApplyRecoveryReadModel =
  generatedArtifactApplyRecoveryFixture()

export const ADMIN_GENERATED_ARTIFACT_REVIEW_PLAN_FIXTURE: AdminGeneratedArtifactReviewPlanReadModel =
  generatedArtifactReviewPlanFixture("fixture-generated-artifact-1", "accept")

export const ADMIN_GENERATED_ARTIFACT_METADATA_APPLY_PLAN_FIXTURE: AdminGeneratedArtifactMetadataApplyPlanReadModel =
  generatedArtifactMetadataApplyPlanFixture("fixture-generated-artifact-1")

export const ADMIN_GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_FIXTURE: AdminGeneratedArtifactMetadataBulkApplyPlanReadModel =
  generatedArtifactMetadataBulkApplyPlanFixture(["fixture-generated-artifact-accepted-1"])

export const ADMIN_SETTINGS_READ_MODEL_FIXTURE: AdminSettingsReadModel = {
  source: "fixture",
  fallback: true,
  general: {
    serverName: "Nako Server",
    serverId: "nako-fixture",
    listenAddr: "0.0.0.0:8096",
    authEnabled: true,
    adminApiVersion: "fixture",
    publicApiVersion: "fixture",
  },
  network: {
    exposureMode: "private_network",
    readinessStatus: "ready",
    readinessReason: "ready",
    externalEndpointConfigured: true,
    allowedOriginCount: 1,
  },
  database: {
    configuredBackendKind: "sqlite",
    activeBackendKind: "sqlite",
    migratedOnStartup: true,
    librariesSupported: true,
  },
  metadata: {
    rawCacheRetentionMs: 7 * 24 * 60 * 60 * 1000,
    rawCacheCleanupOnStartup: true,
    providerCount: 3,
    enabledProviderCount: 2,
    language: "zh-CN",
  },
  transcode: {
    hardwareAcceleration: "qsv",
    hardwareFallbackUsed: false,
    cpuConcurrency: 2,
    gpuConcurrency: 1,
    remuxConcurrency: 2,
    remuxTimeoutMs: 300_000,
  },
  storage: {
    stagingMaxBytes: 100 * 1024 * 1024 * 1024,
    stagingUsedBytes: 45 * 1024 * 1024 * 1024,
    stagingRetentionMs: 24 * 60 * 60 * 1000,
    stagingCleanupOnStartup: true,
    stagingRecordCount: 3,
  },
  runtime: {
    scanConcurrency: 2,
    probeConcurrency: 4,
    metadataConcurrency: 4,
    webhookConcurrency: 2,
  },
}

export function createAdminReadModelsDataSource(
  connection: AdminApiConnection = loadAdminApiConnection(),
  fetcher?: typeof fetch,
) {
  if (connection.mode === "fixture") {
    return fixtureDataSource()
  }

  const client = new AdminApiClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetcher,
  })

  return {
    async loadLibraries(): Promise<AdminLibrariesReadModel> {
      return withFallback(ADMIN_LIBRARY_READ_MODEL_FIXTURE, async () => {
        const [overview, config] = await Promise.all([client.getOverview(), client.getSystemConfig()])
        return mapLibraries(overview, config)
      })
    },

    async loadUsers(): Promise<AdminUsersReadModel> {
      return withFallback(ADMIN_USERS_READ_MODEL_FIXTURE, async () => {
        const [summary, users, sessions] = await Promise.all([
          client.getAccessSummary(),
          client.getAccessUsers({ limit: 100, offset: 0 }),
          client.getPlaybackSessions({ limit: 100, offset: 0 }),
        ])
        return mapUsers(summary, users.users, sessions.sessions)
      })
    },

    async loadTasks(): Promise<AdminTasksReadModel> {
      return withFallback(ADMIN_TASKS_READ_MODEL_FIXTURE, async () => {
        const [jobs, events, summary] = await Promise.all([
          client.getJobs({ limit: 100, offset: 0 }),
          client.getEvents({ limit: 100, offset: 0 }),
          client.getAccessSummary(),
        ])
        return mapTasks(jobs.jobs, events.events, summary)
      })
    },

    async loadLogs(): Promise<AdminLogsReadModel> {
      return withFallback(ADMIN_LOGS_READ_MODEL_FIXTURE, async () => {
        const events = await client.getEvents({ limit: 200, offset: 0 })
        return {
          source: "live",
          fallback: false,
          logs: events.events.map(mapEventToLog),
        }
      })
    },

    async loadAcquisitionIntake(
      query: AdminAcquisitionIntakeCandidatesQuery = {},
    ): Promise<AdminAcquisitionIntakeReadModel> {
      const normalizedQuery = normalizeAcquisitionIntakeQuery(query)
      return withFallback(acquisitionIntakeFixture(normalizedQuery), async () => {
        const response = await client.getAcquisitionIntakeCandidates(normalizedQuery)
        return mapAcquisitionIntake(response, normalizedQuery)
      })
    },

    async loadGeneratedArtifacts(
      query: AdminGeneratedArtifactProposalsQuery = {},
    ): Promise<AdminGeneratedArtifactsReadModel> {
      const normalizedQuery = normalizeGeneratedArtifactsQuery(query)
      return withFallback(generatedArtifactsFixture(normalizedQuery), async () => {
        const response = await client.getGeneratedArtifactProposals(normalizedQuery)
        return mapGeneratedArtifacts(response, normalizedQuery)
      })
    },

    async loadGeneratedArtifactApplyOutcomes(): Promise<AdminGeneratedArtifactMetadataApplyOutcomesReadModel> {
      return withFallback(ADMIN_GENERATED_ARTIFACT_APPLY_OUTCOMES_READ_MODEL_FIXTURE, async () => {
        const response = await client.getGeneratedArtifactApplyOutcomes({ limit: 50, offset: 0 })
        return mapGeneratedArtifactApplyOutcomes(response)
      })
    },

    async loadGeneratedArtifactApplyOutcome(
      outcomeId: string,
    ): Promise<AdminGeneratedArtifactMetadataApplyOutcomeReadModel> {
      return withFallback(generatedArtifactApplyOutcomeFixture(outcomeId), async () => {
        const response = await client.getGeneratedArtifactApplyOutcome(outcomeId)
        return mapGeneratedArtifactApplyOutcomeResponse(response)
      })
    },

    async loadGeneratedArtifactApplyRecovery(
      query: AdminGeneratedArtifactApplyRecoveryQuery = {},
    ): Promise<AdminGeneratedArtifactMetadataApplyRecoveryReadModel> {
      const normalizedQuery = normalizeGeneratedArtifactApplyRecoveryQuery(query)
      return withFallback(generatedArtifactApplyRecoveryFixture(normalizedQuery), async () => {
        const response = await client.getGeneratedArtifactApplyRecovery(normalizedQuery)
        return mapGeneratedArtifactApplyRecovery(response)
      })
    },

    async loadGeneratedArtifactReviewPlan(
      artifactId: string,
      decision: AdminGeneratedArtifactReviewDecision,
    ): Promise<AdminGeneratedArtifactReviewPlanReadModel> {
      return withFallback(generatedArtifactReviewPlanFixture(artifactId, decision), async () => {
        const response = await client.planGeneratedArtifactReview(artifactId, { decision })
        return mapGeneratedArtifactReviewPlanResponse(response)
      })
    },

    async loadGeneratedArtifactMetadataApplyPlan(
      artifactId: string,
    ): Promise<AdminGeneratedArtifactMetadataApplyPlanReadModel> {
      return withFallback(generatedArtifactMetadataApplyPlanFixture(artifactId), async () => {
        const response = await client.planGeneratedArtifactMetadataApply(artifactId)
        return mapGeneratedArtifactMetadataApplyPlanResponse(response)
      })
    },

    async loadMetadataCandidateReview(
      reviewId: string,
    ): Promise<AdminMetadataCandidateReviewReadModel> {
      return withFallback(metadataCandidateReviewFixture(reviewId), async () => {
        const response = await client.getMetadataCandidateReview(reviewId)
        return mapMetadataCandidateReviewResponse(response)
      })
    },

    async loadMetadataCandidateReviews(
      query: AdminMetadataCandidateReviewQueueQuery = {},
    ): Promise<AdminMetadataCandidateReviewQueueReadModel> {
      const normalizedQuery = normalizeMetadataCandidateReviewQueueQuery(query)
      return withFallback(metadataCandidateReviewQueueFixture(normalizedQuery), async () => {
        const response = await client.listMetadataCandidateReviews(normalizedQuery)
        return mapMetadataCandidateReviewQueueResponse(response)
      })
    },

    async loadMetadataCandidateReviewsForItem(
      itemId: string,
      query: AdminPageQuery = {},
    ): Promise<AdminMetadataCandidateReviewListReadModel> {
      const normalizedQuery = normalizeMetadataCandidateReviewListQuery(query)
      return withFallback(metadataCandidateReviewListFixture(itemId, normalizedQuery), async () => {
        const response = await client.listMetadataCandidateReviewsForItem(itemId, normalizedQuery)
        return mapMetadataCandidateReviewListResponse(response)
      })
    },

    async loadMetadataCandidateReviewBatchPlan(
      reviewIds: string[],
    ): Promise<AdminMetadataCandidateReviewBatchPlanReadModel> {
      return withFallback(metadataCandidateReviewBatchPlanFixture(reviewIds), async () => {
        const response = await client.planMetadataCandidateReviewBatchApplication({
          review_ids: reviewIds,
        })
        return mapMetadataCandidateReviewBatchPlanResponse(response)
      })
    },

    async loadMetadataCandidateReviewBatch(
      batchId: string,
    ): Promise<AdminMetadataCandidateReviewBatchReadModel> {
      return withFallback(metadataCandidateReviewBatchFixture(batchId), async () => {
        const response = await client.getMetadataCandidateReviewBatch(batchId)
        return mapMetadataCandidateReviewBatchResponse(response)
      })
    },

    async loadGeneratedArtifactMetadataBulkApplyPlan(
      artifactIds: string[],
    ): Promise<AdminGeneratedArtifactMetadataBulkApplyPlanReadModel> {
      return withFallback(generatedArtifactMetadataBulkApplyPlanFixture(artifactIds), async () => {
        const response = await client.planGeneratedArtifactMetadataBulkApply({
          artifact_ids: artifactIds,
        })
        return mapGeneratedArtifactMetadataBulkApplyPlanResponse(response)
      })
    },

    async loadGeneratedArtifactMetadataBulkApplyBatch(
      batchId: string,
    ): Promise<AdminGeneratedArtifactMetadataBulkApplyBatchReadModel> {
      return withFallback(generatedArtifactMetadataBulkApplyBatchFixture(batchId), async () => {
        const response = await client.getGeneratedArtifactMetadataBulkApplyBatch(batchId)
        return mapGeneratedArtifactMetadataBulkApplyBatchResponse(response)
      })
    },

    async loadSettings(): Promise<AdminSettingsReadModel> {
      return withFallback(ADMIN_SETTINGS_READ_MODEL_FIXTURE, async () => {
        const [config, runtime, staging, rawCache] = await Promise.all([
          client.getSystemConfig(),
          client.getPlaybackRuntime(),
          client.getStorageStaging({ limit: 100, offset: 0 }),
          client.getMetadataRawCacheSettings(),
        ])
        return mapSettings(config, runtime, staging, rawCache)
      })
    },
  }
}

function fixtureDataSource() {
  return {
    async loadLibraries() {
      return ADMIN_LIBRARY_READ_MODEL_FIXTURE
    },
    async loadUsers() {
      return ADMIN_USERS_READ_MODEL_FIXTURE
    },
    async loadTasks() {
      return ADMIN_TASKS_READ_MODEL_FIXTURE
    },
    async loadLogs() {
      return ADMIN_LOGS_READ_MODEL_FIXTURE
    },
    async loadAcquisitionIntake(query: AdminAcquisitionIntakeCandidatesQuery = {}) {
      if (
        Object.values(query).every(
          (value) => value === undefined || value === null || value === "",
        )
      ) {
        return ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE
      }

      return acquisitionIntakeFixture(normalizeAcquisitionIntakeQuery(query))
    },
    async loadGeneratedArtifacts(query: AdminGeneratedArtifactProposalsQuery = {}) {
      if (
        Object.values(query).every(
          (value) => value === undefined || value === null || value === "",
        )
      ) {
        return ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE
      }

      return generatedArtifactsFixture(normalizeGeneratedArtifactsQuery(query))
    },
    async loadGeneratedArtifactApplyOutcomes() {
      return ADMIN_GENERATED_ARTIFACT_APPLY_OUTCOMES_READ_MODEL_FIXTURE
    },
    async loadGeneratedArtifactApplyOutcome(outcomeId: string) {
      return generatedArtifactApplyOutcomeFixture(outcomeId)
    },
    async loadGeneratedArtifactApplyRecovery(query: AdminGeneratedArtifactApplyRecoveryQuery = {}) {
      return generatedArtifactApplyRecoveryFixture(normalizeGeneratedArtifactApplyRecoveryQuery(query))
    },
    async loadGeneratedArtifactReviewPlan(
      artifactId: string,
      decision: AdminGeneratedArtifactReviewDecision,
    ) {
      return generatedArtifactReviewPlanFixture(artifactId, decision)
    },
    async loadGeneratedArtifactMetadataApplyPlan(artifactId: string) {
      return generatedArtifactMetadataApplyPlanFixture(artifactId)
    },
    async loadMetadataCandidateReview(reviewId: string) {
      return metadataCandidateReviewFixture(reviewId)
    },
    async loadMetadataCandidateReviews(query: AdminMetadataCandidateReviewQueueQuery = {}) {
      return metadataCandidateReviewQueueFixture(normalizeMetadataCandidateReviewQueueQuery(query))
    },
    async loadMetadataCandidateReviewsForItem(itemId: string, query: AdminPageQuery = {}) {
      return metadataCandidateReviewListFixture(
        itemId,
        normalizeMetadataCandidateReviewListQuery(query),
      )
    },
    async loadMetadataCandidateReviewBatchPlan(reviewIds: string[]) {
      return metadataCandidateReviewBatchPlanFixture(reviewIds)
    },
    async loadMetadataCandidateReviewBatch(batchId: string) {
      return metadataCandidateReviewBatchFixture(batchId)
    },
    async loadGeneratedArtifactMetadataBulkApplyPlan(artifactIds: string[]) {
      return generatedArtifactMetadataBulkApplyPlanFixture(artifactIds)
    },
    async loadGeneratedArtifactMetadataBulkApplyBatch(batchId: string) {
      return generatedArtifactMetadataBulkApplyBatchFixture(batchId)
    },
    async loadSettings() {
      return ADMIN_SETTINGS_READ_MODEL_FIXTURE
    },
  }
}

async function withFallback<T extends AdminReadModelEnvelope>(
  fixture: T,
  loadLive: () => Promise<T>,
): Promise<T> {
  try {
    return await loadLive()
  } catch (error) {
    return {
      ...fixture,
      source: "fixture",
      fallback: true,
      error: error instanceof Error ? error.message : "Admin API request failed",
    }
  }
}

function mapLibraries(
  overview: AdminOverviewResponse,
  config: AdminServerConfigDiagnosticsResponse,
): AdminLibrariesReadModel {
  const backendStatusByLibrary = new Map(
    overview.storage.backends.map((backend) => [backend.library_id, backend.status]),
  )

  return {
    source: "live",
    fallback: false,
    libraries: config.libraries.map((library) => {
      const backendStatus = backendStatusByLibrary.get(library.id) ?? "ready"
      const available = backendStatus !== "unavailable"

      return {
        id: library.id,
        name: library.name,
        type: normalizeLibraryKind(library.preset),
        paths: [
          {
            path: `${library.root_scheme}://${library.backend_kind}`,
            available,
          },
        ],
        itemCount: 0,
        totalSize: "live",
        lastScanned: "live",
        scanStatus: available ? "idle" : "error",
        errorMessage: available ? undefined : `存储后端不可用: ${backendStatus}`,
        settings: {
          autoScan: true,
          scanInterval: 6,
          useNfo: config.database.capabilities.metadata,
          downloadArt: config.artwork.ingest_worker_enabled,
          metadataLanguage: firstMetadataLanguage(config),
        },
      }
    }),
  }
}

function mapUsers(
  summary: AdminAccessSummaryResponse,
  users: AdminAccessUserRecord[],
  sessions: AdminPlaybackSessionListItem[],
): AdminUsersReadModel {
  const activeSessions = sessions.filter((session) => session.active)
  const activeUserIds = new Set(activeSessions.map((session) => session.principal_id))

  return {
    source: "live",
    fallback: false,
    users: users.map((user) => mapUser(user, activeUserIds)),
    activeSessions: activeSessions.map((session) => ({
      id: session.id,
      userId: session.principal_id,
      userName: session.principal_id,
      device: "Nako Client",
      deviceType: "desktop",
      ip: "redacted",
      location: "Admin API",
      lastActivity: session.active ? `正在播放: ${session.item_id}` : session.state,
      startTime: isoFromMs(session.started_at_ms),
      current: false,
    })),
    libraries: summary.library_access.libraries.map((library) => ({
      id: library.library_id,
      name: library.library_name,
      type: normalizeLibraryKind(library.preset),
    })),
    accessMode: summary.mode,
    accessCapabilities: {
      userAccounts: summary.readiness.user_accounts,
      roles: summary.readiness.roles,
      libraryAccessPolicy: summary.readiness.library_access_policy,
    },
  }
}

function mapTasks(
  jobs: AdminJobListItem[],
  events: AdminOutboxEventListItem[],
  summary: AdminAccessSummaryResponse,
): AdminTasksReadModel {
  const tasks = jobs.map(mapJobToTask)
  const runningTask = tasks.find((task) => task.status === "running") ?? null

  return {
    source: "live",
    fallback: false,
    tasks,
    runningTask,
    history: events.map(mapEventToHistory),
    libraries: summary.library_access.libraries.map((library) => ({
      id: library.library_id,
      name: library.library_name,
      type: normalizeLibraryKind(library.preset),
    })),
  }
}

function mapSettings(
  config: AdminServerConfigDiagnosticsResponse,
  runtime: AdminPlaybackRuntimeDiagnosticsResponse,
  staging: AdminStorageStagingDiagnosticsResponse,
  rawCache: AdminMetadataRawCacheSettingsResponse,
): AdminSettingsReadModel {
  return {
    source: "live",
    fallback: false,
    general: {
      serverName: "Nako Server",
      serverId: `nako-${config.database.active_backend_kind}`,
      listenAddr: config.runtime.listen_addr,
      authEnabled: config.auth.enabled,
      adminApiVersion: config.admin_api_version,
      publicApiVersion: config.public_api_version,
    },
    network: {
      exposureMode: config.network.exposure_mode,
      readinessStatus: config.network.readiness.status,
      readinessReason: config.network.readiness.reason,
      externalEndpointConfigured: config.network.external_endpoint.configured,
      allowedOriginCount: config.network.origins.allowed_origin_count,
    },
    database: {
      configuredBackendKind: config.database.configured_backend_kind,
      activeBackendKind: config.database.active_backend_kind,
      migratedOnStartup: config.database.migrated_on_startup,
      librariesSupported: config.database.capabilities.libraries,
    },
    metadata: {
      rawCacheRetentionMs: rawCache.retention_ms,
      rawCacheCleanupOnStartup: rawCache.cleanup_on_startup,
      providerCount: config.metadata.providers.length,
      enabledProviderCount: config.metadata.providers.filter((provider) => provider.enabled).length,
      language: firstMetadataLanguage(config),
    },
    transcode: {
      hardwareAcceleration: runtime.hardware.selection.acceleration,
      hardwareFallbackUsed: runtime.hardware.selection.fallback_used,
      cpuConcurrency: config.transcode.cpu_concurrency,
      gpuConcurrency: config.transcode.gpu_concurrency,
      remuxConcurrency: runtime.remux.max_concurrent_sessions,
      remuxTimeoutMs: runtime.remux.timeout_ms,
    },
    storage: {
      stagingMaxBytes: staging.summary.configured_max_bytes,
      stagingUsedBytes: staging.summary.used_manifest_bytes,
      stagingRetentionMs: staging.summary.retention_ms,
      stagingCleanupOnStartup: staging.summary.cleanup_on_startup,
      stagingRecordCount: staging.records.length,
    },
    runtime: {
      scanConcurrency: config.runtime.scan_concurrency,
      probeConcurrency: config.runtime.probe_concurrency,
      metadataConcurrency: config.runtime.metadata_concurrency,
      webhookConcurrency: config.runtime.webhook_concurrency,
    },
  }
}

function mapAcquisitionIntake(
  response: AdminAcquisitionIntakeCandidateListResponse,
  query: AdminAcquisitionIntakeCandidatesQuery,
): AdminAcquisitionIntakeReadModel {
  return {
    source: "live",
    fallback: false,
    versions: {
      adminApi: response.admin_api_version,
      publicApi: response.public_api_version,
    },
    query,
    candidates: response.candidates.map(mapAcquisitionIntakeCandidate),
    page: response.page,
  }
}

function mapAcquisitionIntakeCandidate(
  candidate: AdminAcquisitionIntakeCandidateDiagnostic,
): AdminAcquisitionIntakeCandidateReadModel {
  return {
    id: candidate.id,
    targetLibraryId: candidate.target_library_id,
    sourceKind: candidate.source_kind,
    customSourceKind: candidate.custom_source_kind,
    sourceScheme: candidate.source_scheme,
    sourceSummary: candidate.source_ref_redacted,
    sourceKeyFingerprint: candidate.source_key_fingerprint,
    sizeBytes: candidate.size_bytes,
    managedImportArtifactId: candidate.managed_import_artifact_id,
    state: candidate.state,
    readiness: {
      hasDisplayName: candidate.has_display_name,
      hasIntendedLocator: candidate.has_intended_locator,
      hasFingerprint: candidate.has_fingerprint,
      hasDiagnostics: candidate.has_diagnostics,
    },
    firstSeenAt: isoFromMs(candidate.first_seen_at_ms),
    lastSeenAt: isoFromMs(candidate.last_seen_at_ms),
    createdAt: isoFromMs(candidate.created_at_ms),
    updatedAt: isoFromMs(candidate.updated_at_ms),
  }
}

function mapGeneratedArtifacts(
  response: AdminGeneratedArtifactProposalListResponse,
  query: AdminGeneratedArtifactProposalsQuery,
): AdminGeneratedArtifactsReadModel {
  return {
    source: "live",
    fallback: false,
    versions: {
      adminApi: response.admin_api_version,
      publicApi: response.public_api_version,
    },
    query,
    proposals: response.proposals.map(mapGeneratedArtifactProposal),
    page: response.page,
  }
}

export function mapGeneratedArtifactApplyOutcomes(
  response: AdminGeneratedArtifactMetadataApplyOutcomeListResponse,
): AdminGeneratedArtifactMetadataApplyOutcomesReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return {
    source: "live",
    fallback: false,
    versions,
    outcomes: response.outcomes.map((outcome) => mapGeneratedArtifactApplyOutcome(outcome, versions)),
    page: response.page,
  }
}

export function mapGeneratedArtifactApplyOutcomeResponse(
  response: AdminGeneratedArtifactMetadataApplyOutcomeResponse,
): AdminGeneratedArtifactMetadataApplyOutcomeReadModel {
  return mapGeneratedArtifactApplyOutcome(response.outcome, {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  })
}

export function mapGeneratedArtifactApplyRecovery(
  response: AdminGeneratedArtifactMetadataApplyRecoveryResponse,
): AdminGeneratedArtifactMetadataApplyRecoveryReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return {
    source: "live",
    fallback: false,
    versions,
    summary: {
      returnedEntryCount: response.summary.returned_entry_count,
      needsRepairCount: response.summary.needs_repair_count,
      needsReviewCount: response.summary.needs_review_count,
      replayOnlyCount: response.summary.replay_only_count,
      resolvedCount: response.summary.resolved_count,
    },
    entries: response.entries.map((entry) => mapGeneratedArtifactApplyRecoveryEntry(entry, versions)),
    page: response.page,
  }
}

function mapGeneratedArtifactProposal(
  proposal: AdminGeneratedArtifactProposal,
): AdminGeneratedArtifactProposalReadModel {
  return {
    id: proposal.id,
    kind: proposal.kind,
    capability: proposal.capability,
    status: proposal.status,
    target: mapGeneratedArtifactTarget(proposal.target),
    provenance: {
      providerId: proposal.provenance.provider_id,
      providerName: proposal.provenance.provider_name,
      jobId: proposal.provenance.job_id,
      capability: proposal.provenance.capability,
      idempotencyKeyFingerprint: proposal.provenance.idempotency_key_fingerprint,
      promptFingerprint: proposal.provenance.prompt_fingerprint,
      attemptCount: proposal.provenance.attempt_count,
      artifactCreatedAt: proposal.provenance.artifact_created_at,
    },
    payload: mapGeneratedArtifactPayload(proposal.payload),
    readiness: mapGeneratedArtifactReadiness(proposal.readiness),
    createdAt: proposal.created_at,
    updatedAt: proposal.updated_at,
    acceptedAt: proposal.accepted_at,
  }
}

function mapGeneratedArtifactApplyOutcome(
  outcome: AdminGeneratedArtifactMetadataApplyOutcome,
  versions: { adminApi: string; publicApi: string },
): AdminGeneratedArtifactMetadataApplyOutcomeReadModel {
  return {
    source: "live",
    fallback: false,
    versions,
    id: outcome.id,
    artifactId: outcome.artifact_id,
    idempotencyKeyFingerprint: outcome.idempotency_key_fingerprint,
    status: outcome.status,
    applied: outcome.applied,
    changed: outcome.changed,
    appliedSource: outcome.applied_source,
    itemId: outcome.item_id,
    plan: mapGeneratedArtifactMetadataApplyPlan(outcome.plan, versions),
    errorCode: outcome.error_code,
    errorMessage: outcome.error_message,
    createdAt: outcome.created_at,
    updatedAt: outcome.updated_at,
  }
}

function mapGeneratedArtifactApplyRecoveryEntry(
  entry: AdminGeneratedArtifactMetadataApplyRecoveryEntry,
  versions: { adminApi: string; publicApi: string },
): AdminGeneratedArtifactMetadataApplyRecoveryEntryReadModel {
  return {
    source: entry.source,
    attention: entry.attention,
    reason: entry.reason,
    artifactId: entry.artifact_id,
    outcomeId: entry.outcome_id,
    batchId: entry.batch_id,
    batchItemStatus: entry.batch_item_status,
    outcomeStatus: entry.outcome_status,
    itemId: entry.item_id,
    plan: entry.plan ? mapGeneratedArtifactMetadataApplyPlan(entry.plan, versions) : null,
    errorCode: entry.error_code,
    errorMessage: entry.error_message,
    createdAt: entry.created_at,
    updatedAt: entry.updated_at,
  }
}

export function mapGeneratedArtifactReviewPlanResponse(
  response: AdminGeneratedArtifactReviewPlanResponse,
): AdminGeneratedArtifactReviewPlanReadModel {
  const plan = response.plan

  return {
    source: "live",
    fallback: false,
    versions: {
      adminApi: response.admin_api_version,
      publicApi: response.public_api_version,
    },
    artifactId: plan.artifact_id,
    decision: plan.decision,
    status: plan.status,
    action: plan.action,
    reasons: plan.reasons,
    capability: plan.capability,
    kind: plan.kind,
    target: mapGeneratedArtifactTarget(plan.target),
    payload: mapGeneratedArtifactPayload(plan.payload),
    readiness: mapGeneratedArtifactReadiness(plan.readiness),
    boundary: {
      acceptedIntoCanonicalMetadata: plan.boundary.accepted_into_canonical_metadata,
      writesSidecar: plan.boundary.writes_sidecar,
      writesLibraryFiles: plan.boundary.writes_library_files,
      appliesImmediately: plan.boundary.applies_immediately,
      requiresMetadataAuthorityApply: plan.boundary.requires_metadata_authority_apply,
    },
  }
}

export function mapGeneratedArtifactMetadataApplyPlanResponse(
  response: AdminGeneratedArtifactMetadataApplyPlanResponse,
): AdminGeneratedArtifactMetadataApplyPlanReadModel {
  return mapGeneratedArtifactMetadataApplyPlan(response.plan, {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  })
}

export function mapGeneratedArtifactMetadataApplyPlan(
  plan: AdminGeneratedArtifactMetadataApplyPlan,
  versions: { adminApi: string; publicApi: string },
): AdminGeneratedArtifactMetadataApplyPlanReadModel {
  const providerMappings = (plan.provider_mappings ?? []).map(
    mapGeneratedArtifactProviderMappingPlan,
  )

  return {
    source: "live",
    fallback: false,
    versions,
    artifactId: plan.artifact_id,
    status: plan.status,
    executable: plan.executable,
    reasons: plan.reasons,
    target: mapGeneratedArtifactTarget(plan.target),
    payload: mapGeneratedArtifactPayload(plan.payload),
    fields: plan.fields.map(mapGeneratedArtifactMetadataApplyFieldPlan),
    providerMappings,
    applyFieldCount: plan.apply_field_count,
    skippedFieldCount: plan.skipped_field_count,
    noopFieldCount: plan.noop_field_count,
    applyProviderMappingCount:
      plan.apply_provider_mapping_count ?? countProviderMappingsByAction(providerMappings, "apply"),
    skippedProviderMappingCount:
      plan.skipped_provider_mapping_count ?? countProviderMappingsByAction(providerMappings, "skip"),
    noopProviderMappingCount:
      plan.noop_provider_mapping_count ?? countProviderMappingsByAction(providerMappings, "noop"),
  }
}

export function mapMetadataCandidateReviewResponse(
  response: AdminMetadataCandidateReviewResponse,
): AdminMetadataCandidateReviewReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }
  const review = response.review

  return {
    source: "live",
    fallback: false,
    versions,
    reviewId: review.review_id,
    itemId: review.item_id,
    sourceLabel: sourceLabel(review.source),
    sourceKey: review.source_key,
    status: review.status,
    root: mapMetadataCandidateReviewNode(review.root),
    related: review.related.map(mapMetadataCandidateReviewNode),
    relationships: review.relationships.map(mapMetadataCandidateReviewRelationship),
    relatedCount: review.related_count,
    relationshipCount: review.relationship_count,
    expiresAtMs: review.expires_at_ms,
    createdAtMs: review.created_at_ms,
    updatedAtMs: review.updated_at_ms,
    applicationPlan: mapMetadataCandidateReviewApplicationPlan(response.application_plan),
    boundary: mapMetadataCandidateReviewBoundary(response.boundary),
  }
}

export function mapMetadataCandidateReviewListResponse(
  response: AdminMetadataCandidateReviewListResponse,
): AdminMetadataCandidateReviewListReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return {
    source: "live",
    fallback: false,
    versions,
    itemId: response.item_id,
    reviews: response.reviews.map(mapMetadataCandidateReviewListEntry),
    page: response.page,
  }
}

export function mapMetadataCandidateReviewQueueResponse(
  response: AdminMetadataCandidateReviewQueueResponse,
): AdminMetadataCandidateReviewQueueReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return {
    source: "live",
    fallback: false,
    versions,
    reviews: response.reviews.map(mapMetadataCandidateReviewListEntry),
    page: response.page,
  }
}

function mapMetadataCandidateReviewListEntry(
  review: AdminMetadataCandidateReviewListEntry,
): AdminMetadataCandidateReviewListItemReadModel {
  return {
    reviewId: review.review_id,
    itemId: review.item_id,
    sourceLabel: sourceLabel(review.source),
    sourceKey: review.source_key,
    status: review.status,
    root: mapMetadataCandidateReviewNode(review.root),
    relatedCount: review.related_count,
    relationshipCount: review.relationship_count,
    expiresAtMs: review.expires_at_ms,
    createdAtMs: review.created_at_ms,
    updatedAtMs: review.updated_at_ms,
    applicationAction: review.application_plan.action,
    applicationReasons: review.application_plan.reasons,
  }
}

export function mapMetadataCandidateReviewApplyResponse(
  response: AdminMetadataCandidateReviewApplyResponse,
): AdminMetadataCandidateReviewApplyReadModel {
  return {
    source: "live",
    fallback: false,
    versions: {
      adminApi: response.admin_api_version,
      publicApi: response.public_api_version,
    },
    reviewId: response.review_id,
    itemId: response.item_id,
    applied: response.applied,
    changed: response.changed,
    idempotentReplay: response.idempotent_replay,
    idempotencyKeyFingerprint: response.idempotency_key_fingerprint,
    plan: mapMetadataCandidateReviewApplicationPlan(response.plan),
    providerSubject: response.provider_subject
      ? mapMetadataCandidateReviewProviderSubject(response.provider_subject)
      : null,
    providerMapping: response.provider_mapping
      ? mapMetadataCandidateReviewProviderMapping(response.provider_mapping)
      : null,
    boundary: mapMetadataCandidateReviewBoundary(response.boundary),
  }
}

export function mapMetadataCandidateReviewBatchPlanResponse(
  response: AdminMetadataCandidateReviewBatchPlanResponse,
): AdminMetadataCandidateReviewBatchPlanReadModel {
  return {
    source: "live",
    fallback: false,
    versions: {
      adminApi: response.admin_api_version,
      publicApi: response.public_api_version,
    },
    summary: mapMetadataCandidateReviewBatchPlanSummary(response.summary),
    reviews: response.reviews.map(mapMetadataCandidateReviewListEntry),
  }
}

export function mapMetadataCandidateReviewBatchResponse(
  response: AdminMetadataCandidateReviewBatchResponse,
): AdminMetadataCandidateReviewBatchReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return mapMetadataCandidateReviewBatch(response.batch, versions)
}

export function mapMetadataCandidateReviewBatch(
  batch: AdminMetadataCandidateReviewBatch,
  versions: { adminApi: string; publicApi: string },
): AdminMetadataCandidateReviewBatchReadModel {
  return {
    source: "live",
    fallback: false,
    versions,
    id: batch.id,
    jobId: batch.job_id,
    status: batch.status,
    selection: mapMetadataCandidateReviewBatchSelection(batch.selection),
    summary: mapMetadataCandidateReviewBatchPlanSummary(batch.summary),
    executionSummary: mapMetadataCandidateReviewBatchExecutionSummary(batch.execution_summary),
    items: batch.items.map(mapMetadataCandidateReviewBatchItem),
  }
}

function mapMetadataCandidateReviewBatchSelection(
  selection: AdminMetadataCandidateReviewBatchPlanSelection,
): AdminMetadataCandidateReviewBatchSelectionReadModel {
  return {
    requestedReviewCount: selection.requested_review_count,
    selectedReviewCount: selection.selected_review_count,
    duplicateReviewCount: selection.duplicate_review_count,
    maxReviewCount: selection.max_review_count,
  }
}

function mapMetadataCandidateReviewBatchPlanSummary(
  summary: AdminMetadataCandidateReviewBatchPlanSummary,
): AdminMetadataCandidateReviewBatchPlanSummaryReadModel {
  return {
    requestedCount: summary.requested_count,
    returnedCount: summary.returned_count,
    maxReviewCount: summary.max_review_count,
    applyCount: summary.apply_count,
    noopCount: summary.noop_count,
    skipCount: summary.skip_count,
  }
}

function mapMetadataCandidateReviewBatchExecutionSummary(
  summary: AdminMetadataCandidateReviewBatchExecutionSummary,
): AdminMetadataCandidateReviewBatchExecutionSummaryReadModel {
  return {
    totalItemCount: summary.total_item_count,
    pendingItemCount: summary.pending_item_count,
    skippedItemCount: summary.skipped_item_count,
    blockedItemCount: summary.blocked_item_count,
    appliedItemCount: summary.applied_item_count,
    noopItemCount: summary.noop_item_count,
    staleItemCount: summary.stale_item_count,
    conflictItemCount: summary.conflict_item_count,
    failedItemCount: summary.failed_item_count,
  }
}

function mapMetadataCandidateReviewBatchItem(
  item: AdminMetadataCandidateReviewBatchItem,
): AdminMetadataCandidateReviewBatchItemReadModel {
  return {
    reviewId: item.review_id,
    itemId: item.item_id,
    position: item.position,
    status: item.status,
    providerSubjectId: item.provider_subject_id,
    providerMappingId: item.provider_mapping_id,
    error: item.error
      ? {
          code: item.error.code,
          message: item.error.message,
        }
      : null,
  }
}

export function mapGeneratedArtifactMetadataBulkApplyPlanResponse(
  response: AdminGeneratedArtifactMetadataBulkApplyPlanResponse,
): AdminGeneratedArtifactMetadataBulkApplyPlanReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return {
    source: "live",
    fallback: false,
    versions,
    selection: mapGeneratedArtifactMetadataBulkApplyPlanSelection(response.plan.selection),
    summary: mapGeneratedArtifactMetadataBulkApplyPlanSummary(response.plan.summary),
    items: response.plan.items.map((item) =>
      mapGeneratedArtifactMetadataBulkApplyPlanItem(item, versions),
    ),
  }
}

export function mapGeneratedArtifactMetadataBulkApplyBatchResponse(
  response: AdminGeneratedArtifactMetadataBulkApplyBatchResponse,
): AdminGeneratedArtifactMetadataBulkApplyBatchReadModel {
  const versions = {
    adminApi: response.admin_api_version,
    publicApi: response.public_api_version,
  }

  return mapGeneratedArtifactMetadataBulkApplyBatch(response.batch, versions)
}

export function mapGeneratedArtifactMetadataBulkApplyBatch(
  batch: AdminGeneratedArtifactMetadataBulkApplyBatch,
  versions: { adminApi: string; publicApi: string },
): AdminGeneratedArtifactMetadataBulkApplyBatchReadModel {
  return {
    source: "live",
    fallback: false,
    versions,
    id: batch.id,
    jobId: batch.job_id,
    status: batch.status,
    selection: mapGeneratedArtifactMetadataBulkApplyPlanSelection(batch.selection),
    summary: mapGeneratedArtifactMetadataBulkApplyPlanSummary(batch.summary),
    executionSummary: mapGeneratedArtifactMetadataBulkApplyBatchExecutionSummary(
      batch.execution_summary,
    ),
    items: batch.items.map((item) =>
      mapGeneratedArtifactMetadataBulkApplyBatchItem(item, versions),
    ),
    createdAt: batch.created_at,
    updatedAt: batch.updated_at,
  }
}

function mapGeneratedArtifactMetadataBulkApplyPlanSelection(
  selection: AdminGeneratedArtifactMetadataBulkApplyPlanSelection,
): AdminGeneratedArtifactMetadataBulkApplyPlanSelectionReadModel {
  return {
    requestedArtifactCount: selection.requested_artifact_count,
    selectedArtifactCount: selection.selected_artifact_count,
    duplicateArtifactCount: selection.duplicate_artifact_count,
    maxArtifactCount: selection.max_artifact_count,
  }
}

function mapGeneratedArtifactMetadataBulkApplyPlanSummary(
  summary: AdminGeneratedArtifactMetadataBulkApplyPlanSummary,
): AdminGeneratedArtifactMetadataBulkApplyPlanSummaryReadModel {
  return {
    plannedArtifactCount: summary.planned_artifact_count,
    missingArtifactCount: summary.missing_artifact_count,
    readyArtifactCount: summary.ready_artifact_count,
    blockedArtifactCount: summary.blocked_artifact_count,
    staleArtifactCount: summary.stale_artifact_count,
    executableArtifactCount: summary.executable_artifact_count,
    applyFieldCount: summary.apply_field_count,
    skippedFieldCount: summary.skipped_field_count,
    noopFieldCount: summary.noop_field_count,
    applyProviderMappingCount: summary.apply_provider_mapping_count ?? 0,
    skippedProviderMappingCount: summary.skipped_provider_mapping_count ?? 0,
    noopProviderMappingCount: summary.noop_provider_mapping_count ?? 0,
  }
}

function mapGeneratedArtifactMetadataBulkApplyPlanItem(
  item: AdminGeneratedArtifactMetadataBulkApplyPlanItem,
  versions: { adminApi: string; publicApi: string },
): AdminGeneratedArtifactMetadataBulkApplyPlanItemReadModel {
  return {
    artifactId: item.artifact_id,
    status: item.status,
    executable: item.executable,
    reasons: item.reasons,
    plan: item.plan ? mapGeneratedArtifactMetadataApplyPlan(item.plan, versions) : null,
  }
}

function mapGeneratedArtifactMetadataBulkApplyBatchExecutionSummary(
  summary: AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummary,
): AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummaryReadModel {
  return {
    totalItemCount: summary.total_item_count,
    pendingItemCount: summary.pending_item_count,
    skippedItemCount: summary.skipped_item_count,
    appliedItemCount: summary.applied_item_count,
    noopItemCount: summary.noop_item_count,
    staleItemCount: summary.stale_item_count,
    failedItemCount: summary.failed_item_count,
  }
}

function mapGeneratedArtifactMetadataBulkApplyBatchItem(
  item: AdminGeneratedArtifactMetadataBulkApplyBatchItem,
  versions: { adminApi: string; publicApi: string },
): AdminGeneratedArtifactMetadataBulkApplyBatchItemReadModel {
  return {
    artifactId: item.artifact_id,
    position: item.position,
    status: item.status,
    outcomeId: item.outcome_id,
    errorCode: item.error_code,
    errorMessage: item.error_message,
    planItem: mapGeneratedArtifactMetadataBulkApplyPlanItem(item.plan_item, versions),
    createdAt: item.created_at,
    updatedAt: item.updated_at,
  }
}

function mapGeneratedArtifactMetadataApplyFieldPlan(
  field: AdminGeneratedArtifactMetadataApplyFieldPlan,
): AdminGeneratedArtifactMetadataApplyFieldPlanReadModel {
  return {
    field: field.field,
    action: field.action,
    reasons: field.reasons,
    current: mapGeneratedArtifactMetadataValueSummary(field.current),
    incoming: mapGeneratedArtifactMetadataValueSummary(field.incoming),
  }
}

function mapGeneratedArtifactProviderMappingPlan(
  mapping: AdminGeneratedArtifactProviderMappingPlan,
): AdminGeneratedArtifactProviderMappingPlanReadModel {
  return {
    subject: mapGeneratedArtifactProviderSubjectPlan(mapping.subject),
    action: mapping.action,
    reasons: mapping.reasons,
    confidenceMilli: mapping.confidence_milli,
    existingMappingStatus: mapping.existing_mapping_status,
  }
}

function mapGeneratedArtifactProviderSubjectPlan(
  subject: AdminGeneratedArtifactProviderSubjectPlan,
): AdminGeneratedArtifactProviderSubjectPlanReadModel {
  return {
    provider: subject.provider,
    providerName: subject.provider_name,
    subjectKind: subject.subject_kind,
    subjectKindName: subject.subject_kind_name,
    subjectKey: subject.subject_key,
    title: subject.title,
    releaseYear: subject.release_year,
    locale: subject.locale,
  }
}

function countProviderMappingsByAction(
  mappings: AdminGeneratedArtifactProviderMappingPlanReadModel[],
  action: string,
) {
  return mappings.filter((mapping) => mapping.action === action).length
}

function mapGeneratedArtifactMetadataValueSummary(
  value: AdminGeneratedArtifactMetadataValueSummary,
): AdminGeneratedArtifactMetadataValueSummaryReadModel {
  return {
    present: value.present,
    empty: value.empty,
    valueFingerprint: value.value_fingerprint,
    valueBytes: value.value_bytes,
    itemCount: value.item_count,
  }
}

function mapGeneratedArtifactTarget(
  target: AdminGeneratedArtifactTarget,
): AdminGeneratedArtifactTargetReadModel {
  return {
    kind: target.kind,
    libraryId: target.library_id,
    itemId: target.item_id,
    sourceId: target.source_id,
  }
}

function mapGeneratedArtifactPayload(
  payload: AdminGeneratedArtifactPayloadSummary,
): AdminGeneratedArtifactPayloadReadModel {
  return {
    validJson: payload.valid_json,
    shape: payload.shape,
    payloadFingerprint: payload.payload_fingerprint,
    payloadBytes: payload.payload_bytes,
    objectFieldCount: payload.object_field_count,
    arrayItemCount: payload.array_item_count,
    hasTextualValues: payload.has_textual_values,
    hasExplanation: payload.has_explanation,
    confidenceMilli: payload.confidence_milli,
  }
}

function mapGeneratedArtifactReadiness(
  readiness: AdminGeneratedArtifactReadiness,
): AdminGeneratedArtifactReadinessReadModel {
  return {
    status: readiness.status,
    actionable: readiness.actionable,
    reasons: readiness.reasons,
  }
}

function mapMetadataCandidateReviewNode(
  node: AdminMetadataCandidateReviewNode,
): AdminMetadataCandidateReviewNodeReadModel {
  return {
    sourceLabel: sourceLabel(node.source),
    kind: node.kind,
    subject: node.subject ? mapMetadataCandidateSubject(node.subject) : null,
    metadata: mapMetadataCandidateReviewMetadataSummary(node.metadata),
  }
}

function mapMetadataCandidateReviewRelationship(
  relationship: AdminMetadataCandidateReviewRelationship,
): AdminMetadataCandidateReviewRelationshipReadModel {
  return {
    parentSubject: mapMetadataCandidateSubject(relationship.parent_subject),
    childSubject: mapMetadataCandidateSubject(relationship.child_subject),
    kind: relationship.kind,
  }
}

function mapMetadataCandidateReviewApplicationPlan(
  plan: AdminMetadataCandidateReviewApplicationPlan,
): AdminMetadataCandidateReviewApplicationPlanReadModel {
  return {
    reviewId: plan.review_id,
    itemId: plan.item_id,
    action: plan.action,
    reasons: plan.reasons,
    existingMappingId: plan.existing_mapping_id,
    existingMappingStatus: plan.existing_mapping_status,
  }
}

function mapMetadataCandidateReviewBoundary(
  boundary: AdminMetadataCandidateReviewApplicationBoundary,
): AdminMetadataCandidateReviewBoundaryReadModel {
  return {
    applyMutationRequired: boundary.apply_mutation_required,
    applyUpdatesRootProviderSubject: boundary.apply_updates_root_provider_subject,
    applyUpdatesRootProviderMapping: boundary.apply_updates_root_provider_mapping,
    applyUpdatesRelatedProviderSubjects: boundary.apply_updates_related_provider_subjects,
    updatesCanonicalMetadata: boundary.updates_canonical_metadata,
    updatesHierarchy: boundary.updates_hierarchy,
  }
}

function mapMetadataCandidateReviewProviderSubject(
  subject: AdminMetadataCandidateReviewProviderSubject,
): AdminMetadataCandidateReviewProviderSubjectReadModel {
  return {
    subjectId: subject.subject_id,
    ...mapMetadataCandidateSubject(subject),
  }
}

function mapMetadataCandidateReviewProviderMapping(
  mapping: AdminMetadataCandidateReviewProviderMapping,
): AdminMetadataCandidateReviewProviderMappingReadModel {
  return {
    mappingId: mapping.mapping_id,
    itemId: mapping.item_id,
    subjectId: mapping.subject_id,
    status: mapping.status,
    confidenceMilli: mapping.confidence_milli,
    sourceLabel: sourceLabel(mapping.source),
  }
}

function mapMetadataCandidateSubject(
  subject: AdminMetadataCandidateSubject,
): AdminMetadataCandidateSubjectReadModel {
  return {
    provider: sourceLabel(subject.provider),
    subjectKind: sourceLabel(subject.subject_kind),
    subjectKey: subject.subject_key,
    title: subject.title,
    releaseYear: subject.release_year,
    locale: subject.locale,
  }
}

function mapMetadataCandidateReviewMetadataSummary(
  summary: AdminMetadataCandidateReviewMetadataSummary,
): AdminMetadataCandidateReviewMetadataSummaryReadModel {
  return {
    title: summary.title,
    releaseDate: summary.release_date,
    descriptionPresent: summary.description_present,
    genreCount: summary.genre_count,
    tagCount: summary.tag_count,
    imageCount: summary.image_count,
  }
}

function sourceLabel(source: unknown): string {
  if (typeof source === "string") {
    return source
  }

  if (!source || typeof source !== "object") {
    return "unknown"
  }

  const record = source as Record<string, unknown>
  if (typeof record.provider === "string") {
    return record.provider
  }
  if (typeof record.addon === "string") {
    return `addon:${record.addon}`
  }
  if (typeof record.automation === "string") {
    return `automation:${record.automation}`
  }
  if (typeof record.other === "string") {
    return record.other
  }

  return "unknown"
}

function normalizeAcquisitionIntakeQuery(
  query: AdminAcquisitionIntakeCandidatesQuery,
): AdminAcquisitionIntakeCandidatesQuery {
  return {
    library_id: cleanQueryValue(query.library_id),
    state: cleanQueryValue(query.state),
    source_kind: cleanQueryValue(query.source_kind),
    managed_import_artifact_id: cleanQueryValue(query.managed_import_artifact_id),
    limit: query.limit ?? 50,
    offset: query.offset ?? 0,
  }
}

function normalizeGeneratedArtifactsQuery(
  query: AdminGeneratedArtifactProposalsQuery,
): AdminGeneratedArtifactProposalsQuery {
  return {
    limit: query.limit ?? 50,
    offset: query.offset ?? 0,
  }
}

function normalizeGeneratedArtifactApplyRecoveryQuery(
  query: AdminGeneratedArtifactApplyRecoveryQuery,
): AdminGeneratedArtifactApplyRecoveryQuery {
  return {
    attention: normalizeGeneratedArtifactApplyRecoveryAttention(query.attention),
    limit: query.limit ?? 50,
    offset: query.offset ?? 0,
  }
}

function normalizeMetadataCandidateReviewListQuery(query: AdminPageQuery): AdminPageQuery {
  return {
    limit: query.limit ?? 50,
    offset: query.offset ?? 0,
  }
}

function normalizeMetadataCandidateReviewQueueQuery(
  query: AdminMetadataCandidateReviewQueueQuery,
): AdminMetadataCandidateReviewQueueQuery {
  return {
    status: query.status,
    provider: query.provider,
    limit: query.limit ?? 50,
    offset: query.offset ?? 0,
  }
}

function normalizeGeneratedArtifactApplyRecoveryAttention(
  value: AdminGeneratedArtifactApplyRecoveryQuery["attention"],
): AdminGeneratedArtifactApplyRecoveryQuery["attention"] {
  switch (value) {
    case "needs_repair":
    case "needs_review":
    case "replay_only":
    case "resolved":
      return value
    default:
      return undefined
  }
}

function cleanQueryValue(value: string | undefined) {
  const trimmed = value?.trim()
  return trimmed ? trimmed : undefined
}

function acquisitionIntakeFixture(
  query: AdminAcquisitionIntakeCandidatesQuery,
): AdminAcquisitionIntakeReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    query,
    candidates: [
      {
        id: "fixture-intake-1",
        targetLibraryId: query.library_id ?? "library-movies",
        sourceKind: query.source_kind ?? "watch_folder",
        customSourceKind: false,
        sourceScheme: "file",
        sourceSummary: "file://<redacted>/Movie.mkv",
        sourceKeyFingerprint: "sha256:fixture-intake-candidate",
        sizeBytes: 8_589_934_592,
        managedImportArtifactId: query.managed_import_artifact_id ?? "artifact-fixture-1",
        state: query.state ?? "ready",
        readiness: {
          hasDisplayName: true,
          hasIntendedLocator: true,
          hasFingerprint: true,
          hasDiagnostics: true,
        },
        firstSeenAt: "2024-03-15T03:00:00.000Z",
        lastSeenAt: "2024-03-15T03:05:00.000Z",
        createdAt: "2024-03-15T03:00:00.000Z",
        updatedAt: "2024-03-15T03:05:00.000Z",
      },
    ],
    page: {
      limit: query.limit ?? 50,
      offset: query.offset ?? 0,
      returned: 1,
    },
  }
}

function generatedArtifactsFixture(
  query: AdminGeneratedArtifactProposalsQuery,
): AdminGeneratedArtifactsReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    query,
    proposals: [
      {
        id: "fixture-generated-artifact-1",
        kind: "metadata_suggestion",
        capability: "item_metadata_suggest",
        status: "pending_review",
        target: {
          kind: "media_item",
          libraryId: "library-movies",
          itemId: "item-fixture-1",
          sourceId: null,
        },
        provenance: {
          providerId: "automation-provider-fixture",
          providerName: "Fixture Automation Provider",
          jobId: "job-generated-artifact-fixture",
          capability: "item_metadata_suggest",
          idempotencyKeyFingerprint: "sha256:idempotency-fixture",
          promptFingerprint: "sha256:prompt-fixture",
          attemptCount: 1,
          artifactCreatedAt: "2024-03-15T02:59:00.000Z",
        },
        payload: {
          validJson: true,
          shape: "object",
          payloadFingerprint: "sha256:payload-fixture",
          payloadBytes: 2048,
          objectFieldCount: 8,
          arrayItemCount: null,
          hasTextualValues: true,
          hasExplanation: true,
          confidenceMilli: 820,
        },
        readiness: {
          status: "ready",
          actionable: true,
          reasons: ["ready_for_review"],
        },
        createdAt: "2024-03-15T03:00:00.000Z",
        updatedAt: "2024-03-15T03:05:00.000Z",
        acceptedAt: null,
      },
      {
        id: "fixture-generated-artifact-accepted-1",
        kind: "metadata_suggestion",
        capability: "item_metadata_suggest",
        status: "accepted",
        target: {
          kind: "media_item",
          libraryId: "library-movies",
          itemId: "item-fixture-1",
          sourceId: null,
        },
        provenance: {
          providerId: "automation-provider-fixture",
          providerName: "Fixture Automation Provider",
          jobId: "job-generated-artifact-fixture",
          capability: "item_metadata_suggest",
          idempotencyKeyFingerprint: "sha256:idempotency-fixture-accepted",
          promptFingerprint: "sha256:prompt-fixture-accepted",
          attemptCount: 1,
          artifactCreatedAt: "2024-03-15T03:09:00.000Z",
        },
        payload: {
          validJson: true,
          shape: "object",
          payloadFingerprint: "sha256:payload-fixture-accepted",
          payloadBytes: 2048,
          objectFieldCount: 8,
          arrayItemCount: null,
          hasTextualValues: true,
          hasExplanation: true,
          confidenceMilli: 840,
        },
        readiness: {
          status: "accepted",
          actionable: true,
          reasons: ["accepted_generated_artifact"],
        },
        createdAt: "2024-03-15T03:08:00.000Z",
        updatedAt: "2024-03-15T03:10:00.000Z",
        acceptedAt: "2024-03-15T03:10:00.000Z",
      },
    ],
    page: {
      limit: query.limit ?? 50,
      offset: query.offset ?? 0,
      returned: 2,
    },
  }
}

function generatedArtifactApplyOutcomesFixture(): AdminGeneratedArtifactMetadataApplyOutcomesReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    outcomes: [generatedArtifactApplyOutcomeFixture("fixture-generated-outcome-1")],
    page: {
      limit: 50,
      offset: 0,
      returned: 1,
    },
  }
}

function generatedArtifactApplyOutcomeFixture(
  outcomeId: string,
): AdminGeneratedArtifactMetadataApplyOutcomeReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    id: outcomeId,
    artifactId: "fixture-generated-artifact-1",
    idempotencyKeyFingerprint: "a1b2c3d4e5f60708",
    status: "failed",
    applied: false,
    changed: false,
    appliedSource: null,
    itemId: "fixture-item-1",
    plan: generatedArtifactMetadataApplyPlanFixture("fixture-generated-artifact-1"),
    errorCode: "target_stale",
    errorMessage: "target became stale before apply execution",
    createdAt: "2026-06-02T12:00:00Z",
    updatedAt: "2026-06-02T12:05:00Z",
  }
}

function generatedArtifactApplyRecoveryFixture(
  query: AdminGeneratedArtifactApplyRecoveryQuery = {},
): AdminGeneratedArtifactMetadataApplyRecoveryReadModel {
  const attention = query.attention ?? "needs_repair"

  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    summary: {
      returnedEntryCount: 1,
      needsRepairCount: 1,
      needsReviewCount: 0,
      replayOnlyCount: 0,
      resolvedCount: 0,
    },
    entries: [
      {
        source: "apply_outcome",
        attention,
        reason: "apply_outcome_failed",
        artifactId: "fixture-generated-artifact-1",
        outcomeId: "fixture-generated-outcome-1",
        batchId: null,
        batchItemStatus: null,
        outcomeStatus: "failed",
        itemId: "fixture-item-1",
        plan: generatedArtifactMetadataApplyPlanFixture("fixture-generated-artifact-1"),
        errorCode: "target_stale",
        errorMessage: "target became stale before apply execution",
        createdAt: "2026-06-02T12:00:00Z",
        updatedAt: "2026-06-02T12:05:00Z",
      },
    ],
    page: {
      limit: query.limit ?? 50,
      offset: query.offset ?? 0,
      returned: 1,
    },
  }
}

function generatedArtifactReviewPlanFixture(
  artifactId: string,
  decision: AdminGeneratedArtifactReviewDecision,
): AdminGeneratedArtifactReviewPlanReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    artifactId,
    decision,
    status: "ready",
    action: decision === "accept" ? "accept_generated_artifact" : "reject_generated_artifact",
    reasons: ["ready_for_review"],
    capability: "item_metadata_suggest",
    kind: "metadata_suggestion",
    target: {
      kind: "media_item",
      libraryId: "library-movies",
      itemId: "item-fixture-1",
      sourceId: null,
    },
    payload: {
      validJson: true,
      shape: "object",
      payloadFingerprint: "sha256:payload-fixture",
      payloadBytes: 2048,
      objectFieldCount: 8,
      arrayItemCount: null,
      hasTextualValues: true,
      hasExplanation: true,
      confidenceMilli: 820,
    },
    readiness: {
      status: "ready",
      actionable: true,
      reasons: ["ready_for_review"],
    },
    boundary: {
      acceptedIntoCanonicalMetadata: false,
      writesSidecar: false,
      writesLibraryFiles: false,
      appliesImmediately: false,
      requiresMetadataAuthorityApply: decision === "accept",
    },
  }
}

function metadataCandidateReviewFixture(reviewId: string): AdminMetadataCandidateReviewReadModel {
  const rootSubject = metadataCandidateSubjectFixture("subject", "1437", "Fixture Candidate")
  const childSubject = metadataCandidateSubjectFixture("episode", "1437/1", "Episode One")

  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    reviewId,
    itemId: "fixture-item-1",
    sourceLabel: "bangumi",
    sourceKey: "bangumi:1437",
    status: "accepted",
    root: {
      sourceLabel: "bangumi",
      kind: "series",
      subject: rootSubject,
      metadata: metadataCandidateSummaryFixture("Fixture Candidate"),
    },
    related: [
      {
        sourceLabel: "bangumi",
        kind: "episode",
        subject: childSubject,
        metadata: metadataCandidateSummaryFixture("Episode One"),
      },
    ],
    relationships: [
      {
        parentSubject: rootSubject,
        childSubject,
        kind: "contains",
      },
    ],
    relatedCount: 1,
    relationshipCount: 1,
    expiresAtMs: null,
    createdAtMs: 100,
    updatedAtMs: 300,
    applicationPlan: {
      reviewId,
      itemId: "fixture-item-1",
      action: "apply",
      reasons: ["ready"],
      existingMappingId: null,
      existingMappingStatus: null,
    },
    boundary: metadataCandidateReviewBoundaryFixture(),
  }
}

function metadataCandidateReviewListFixture(
  itemId: string,
  query: AdminPageQuery,
): AdminMetadataCandidateReviewListReadModel {
  const detail = metadataCandidateReviewFixture("fixture-metadata-candidate-review-accepted-1")

  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    itemId,
    reviews: [
      {
        reviewId: detail.reviewId,
        itemId,
        sourceLabel: detail.sourceLabel,
        sourceKey: detail.sourceKey,
        status: detail.status,
        root: detail.root,
        relatedCount: detail.relatedCount,
        relationshipCount: detail.relationshipCount,
        expiresAtMs: detail.expiresAtMs,
        createdAtMs: detail.createdAtMs,
        updatedAtMs: detail.updatedAtMs,
        applicationAction: detail.applicationPlan.action,
        applicationReasons: detail.applicationPlan.reasons,
      },
    ],
    page: {
      limit: query.limit ?? 50,
      offset: query.offset ?? 0,
      returned: 1,
    },
  }
}

function metadataCandidateReviewQueueFixture(
  query: AdminMetadataCandidateReviewQueueQuery,
): AdminMetadataCandidateReviewQueueReadModel {
  const itemId = "fixture-item-queue-1"
  const detail = metadataCandidateReviewFixture("fixture-metadata-candidate-review-accepted-1")

  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    reviews: [
      {
        reviewId: detail.reviewId,
        itemId,
        sourceLabel: detail.sourceLabel,
        sourceKey: detail.sourceKey,
        status: query.status ?? detail.status,
        root: detail.root,
        relatedCount: detail.relatedCount,
        relationshipCount: detail.relationshipCount,
        expiresAtMs: detail.expiresAtMs,
        createdAtMs: detail.createdAtMs,
        updatedAtMs: detail.updatedAtMs,
        applicationAction: detail.applicationPlan.action,
        applicationReasons: detail.applicationPlan.reasons,
      },
    ],
    page: {
      limit: query.limit ?? 50,
      offset: query.offset ?? 0,
      returned: 1,
    },
  }
}

function metadataCandidateReviewBatchPlanFixture(
  reviewIds: string[],
): AdminMetadataCandidateReviewBatchPlanReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: { adminApi: "fixture", publicApi: "fixture" },
    summary: {
      requestedCount: reviewIds.length,
      returnedCount: 0,
      maxReviewCount: 50,
      applyCount: 0,
      noopCount: 0,
      skipCount: 0,
    },
    reviews: [],
  }
}

function metadataCandidateReviewBatchFixture(
  batchId: string,
): AdminMetadataCandidateReviewBatchReadModel {
  const detail = metadataCandidateReviewFixture("fixture-metadata-candidate-review-accepted-1")

  return {
    source: "fixture",
    fallback: true,
    versions: { adminApi: "fixture", publicApi: "fixture" },
    id: batchId,
    jobId: "fixture-metadata-candidate-review-batch-job",
    status: "completed",
    selection: {
      requestedReviewCount: 1,
      selectedReviewCount: 1,
      duplicateReviewCount: 0,
      maxReviewCount: 50,
    },
    summary: {
      requestedCount: 1,
      returnedCount: 1,
      maxReviewCount: 50,
      applyCount: 1,
      noopCount: 0,
      skipCount: 0,
    },
    executionSummary: {
      totalItemCount: 1,
      pendingItemCount: 0,
      skippedItemCount: 0,
      blockedItemCount: 0,
      appliedItemCount: 1,
      noopItemCount: 0,
      staleItemCount: 0,
      conflictItemCount: 0,
      failedItemCount: 0,
    },
    items: [
      {
        reviewId: detail.reviewId,
        itemId: detail.itemId,
        position: 0,
        status: "applied",
        providerSubjectId: "fixture-provider-subject",
        providerMappingId: "fixture-provider-mapping",
        error: null,
      },
    ],
  }
}

function metadataCandidateSubjectFixture(
  subjectKind: string,
  subjectKey: string,
  title: string,
): AdminMetadataCandidateSubjectReadModel {
  return {
    provider: "bangumi",
    subjectKind,
    subjectKey,
    title,
    releaseYear: 2026,
    locale: "zh-CN",
  }
}

function metadataCandidateSummaryFixture(
  title: string,
): AdminMetadataCandidateReviewMetadataSummaryReadModel {
  return {
    title,
    releaseDate: "2026-06-01",
    descriptionPresent: true,
    genreCount: 1,
    tagCount: 1,
    imageCount: 1,
  }
}

function metadataCandidateReviewBoundaryFixture(): AdminMetadataCandidateReviewBoundaryReadModel {
  return {
    applyMutationRequired: true,
    applyUpdatesRootProviderSubject: true,
    applyUpdatesRootProviderMapping: true,
    applyUpdatesRelatedProviderSubjects: false,
    updatesCanonicalMetadata: false,
    updatesHierarchy: false,
  }
}

function generatedArtifactMetadataApplyPlanFixture(
  artifactId: string,
): AdminGeneratedArtifactMetadataApplyPlanReadModel {
  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    artifactId,
    status: "ready",
    executable: true,
    reasons: ["accepted_generated_artifact"],
    target: {
      kind: "media_item",
      libraryId: "library-movies",
      itemId: "item-fixture-1",
      sourceId: null,
    },
    payload: {
      validJson: true,
      shape: "object",
      payloadFingerprint: "sha256:payload-fixture",
      payloadBytes: 2048,
      objectFieldCount: 8,
      arrayItemCount: null,
      hasTextualValues: true,
      hasExplanation: true,
      confidenceMilli: 820,
    },
    fields: [
      metadataApplyFieldFixture(
        "title",
        "apply",
        "incoming_differs",
        "sha256:fixture-current-title",
        "sha256:fixture-incoming-title",
      ),
      metadataApplyFieldFixture(
        "overview",
        "skip",
        "field_locked",
        "sha256:fixture-current-overview",
        "sha256:fixture-incoming-overview",
      ),
    ],
    providerMappings: [
      providerMappingPlanFixture(
        "tmdb",
        "Fixture TMDB",
        "movie",
        "Movie",
        "tmdb-550",
        "Fixture Movie",
        2024,
        "apply",
        "incoming_provider_subject",
        820,
        null,
      ),
    ],
    applyFieldCount: 1,
    skippedFieldCount: 1,
    noopFieldCount: 0,
    applyProviderMappingCount: 1,
    skippedProviderMappingCount: 0,
    noopProviderMappingCount: 0,
  }
}

function generatedArtifactMetadataBulkApplyPlanFixture(
  artifactIds: string[],
): AdminGeneratedArtifactMetadataBulkApplyPlanReadModel {
  const uniqueArtifactIds = Array.from(new Set(artifactIds))
  const items = uniqueArtifactIds.map((artifactId) =>
    generatedArtifactMetadataBulkApplyPlanItemFixture(artifactId),
  )
  const executableItems = items.filter((item) => item.executable && item.plan)
  const missingItems = items.filter((item) => item.status === "missing")

  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    selection: {
      requestedArtifactCount: artifactIds.length,
      selectedArtifactCount: uniqueArtifactIds.length,
      duplicateArtifactCount: artifactIds.length - uniqueArtifactIds.length,
      maxArtifactCount: 100,
    },
    summary: {
      plannedArtifactCount: items.length - missingItems.length,
      missingArtifactCount: missingItems.length,
      readyArtifactCount: executableItems.length,
      blockedArtifactCount: items.filter((item) => item.status === "blocked").length,
      staleArtifactCount: items.filter((item) => item.status === "stale").length,
      executableArtifactCount: executableItems.length,
      applyFieldCount: executableItems.reduce(
        (total, item) => total + (item.plan?.applyFieldCount ?? 0),
        0,
      ),
      skippedFieldCount: executableItems.reduce(
        (total, item) => total + (item.plan?.skippedFieldCount ?? 0),
        0,
      ),
      noopFieldCount: executableItems.reduce(
        (total, item) => total + (item.plan?.noopFieldCount ?? 0),
        0,
      ),
      applyProviderMappingCount: items.reduce(
        (total, item) => total + (item.plan?.applyProviderMappingCount ?? 0),
        0,
      ),
      skippedProviderMappingCount: items.reduce(
        (total, item) => total + (item.plan?.skippedProviderMappingCount ?? 0),
        0,
      ),
      noopProviderMappingCount: items.reduce(
        (total, item) => total + (item.plan?.noopProviderMappingCount ?? 0),
        0,
      ),
    },
    items,
  }
}

function generatedArtifactMetadataBulkApplyBatchFixture(
  batchId: string,
): AdminGeneratedArtifactMetadataBulkApplyBatchReadModel {
  const plan = generatedArtifactMetadataBulkApplyPlanFixture([
    "fixture-generated-artifact-accepted-1",
    "fixture-generated-artifact-missing",
  ])
  const now = "2024-03-15T03:12:00.000Z"

  return {
    source: "fixture",
    fallback: true,
    versions: {
      adminApi: "fixture",
      publicApi: "fixture",
    },
    id: batchId,
    jobId: "fixture-generated-artifact-metadata-bulk-apply-job",
    status: "completed",
    selection: plan.selection,
    summary: plan.summary,
    executionSummary: {
      totalItemCount: plan.items.length,
      pendingItemCount: 0,
      skippedItemCount: 1,
      appliedItemCount: 1,
      noopItemCount: 0,
      staleItemCount: 0,
      failedItemCount: 0,
    },
    items: plan.items.map((item, index) => ({
      artifactId: item.artifactId,
      position: index,
      status: item.executable ? "applied" : "skipped",
      outcomeId: item.executable ? `fixture-metadata-apply-outcome-${index + 1}` : null,
      errorCode: item.executable ? null : item.status,
      errorMessage: item.executable ? null : "Generated artifact is not available for apply",
      planItem: item,
      createdAt: now,
      updatedAt: now,
    })),
    createdAt: now,
    updatedAt: now,
  }
}

function generatedArtifactMetadataBulkApplyPlanItemFixture(
  artifactId: string,
): AdminGeneratedArtifactMetadataBulkApplyPlanItemReadModel {
  if (artifactId.includes("missing")) {
    return {
      artifactId,
      status: "missing",
      executable: false,
      reasons: ["generated_artifact_not_found"],
      plan: null,
    }
  }

  const plan = generatedArtifactMetadataApplyPlanFixture(artifactId)

  return {
    artifactId,
    status: "planned",
    executable: true,
    reasons: ["accepted_generated_artifact"],
    plan,
  }
}

function metadataApplyFieldFixture(
  field: string,
  action: string,
  reason: string,
  currentFingerprint: string,
  incomingFingerprint: string,
): AdminGeneratedArtifactMetadataApplyFieldPlanReadModel {
  return {
    field,
    action,
    reasons: [reason],
    current: metadataApplyValueFixture(currentFingerprint),
    incoming: metadataApplyValueFixture(incomingFingerprint),
  }
}

function providerMappingPlanFixture(
  provider: string,
  providerName: string,
  subjectKind: string,
  subjectKindName: string,
  subjectKey: string,
  title: string,
  releaseYear: number | null,
  action: string,
  reason: string,
  confidenceMilli: number | null,
  existingMappingStatus: string | null,
): AdminGeneratedArtifactProviderMappingPlanReadModel {
  return {
    subject: {
      provider,
      providerName,
      subjectKind,
      subjectKindName,
      subjectKey,
      title,
      releaseYear,
      locale: "zh-CN",
    },
    action,
    reasons: [reason],
    confidenceMilli,
    existingMappingStatus,
  }
}

function metadataApplyValueFixture(valueFingerprint: string): AdminGeneratedArtifactMetadataValueSummaryReadModel {
  return {
    present: true,
    empty: false,
    valueFingerprint,
    valueBytes: 24,
    itemCount: null,
  }
}

function mapUser(user: AdminAccessUserRecord, activeUserIds: Set<string>): AdminUserReadModel {
  const role = mapUserRole(user.roles)
  const status: AdminUserUiStatus =
    user.status === "disabled" ? "disabled" : activeUserIds.has(user.principal_id) ? "online" : "offline"

  return {
    id: user.user_id,
    name: user.display_name,
    username: user.username,
    email: null,
    role,
    avatar: null,
    status,
    lastActive: status === "online" ? "在线" : status === "disabled" ? "账户已禁用" : "离线",
    createdAt: formatMsDate(user.created_at_ms),
    libraryAccess: role === "admin" ? ["all"] : [],
    settings: {
      canDownload: role !== "guest",
      canDelete: role === "admin",
      canManageUsers: role === "admin",
      maxBitrate: null,
      transcodePolicy: "auto",
      maxStreams: null,
      remoteAccess: true,
    },
    stats: {
      totalPlays: 0,
      totalWatchTime: "0 小时",
      lastLogin: "live",
    },
  }
}

function mapJobToTask(job: AdminJobListItem): AdminScheduledTaskReadModel {
  const taskType = mapJobKind(job.kind)
  const status = mapJobStatus(job.status)

  return {
    id: job.id,
    name: job.kind,
    description: `${job.resource_class} · ${job.library_id ?? job.source_id ?? "server"}`,
    type: taskType,
    enabled: status !== "failed",
    schedule: {
      frequency: "cron",
      cron: "manual",
    },
    config: {
      targetLibraries: job.library_id ? [job.library_id] : undefined,
    },
    lastRun: job.completed_at
      ? {
          timestamp: job.completed_at,
          status: job.status === "failed" ? "failed" : "success",
          duration: "live",
          error: job.has_error ? "任务报告包含错误" : undefined,
        }
      : undefined,
    nextRun: job.status === "queued" ? job.queued_at : undefined,
    status,
    progress: status === "running" ? 50 : undefined,
  }
}

function mapEventToHistory(event: AdminOutboxEventListItem): AdminTaskHistoryReadModel {
  return {
    id: event.id,
    taskId: event.id,
    taskName: event.kind,
    taskType: "cleanup",
    timestamp: event.occurred_at,
    status: event.has_error ? "failed" : "success",
    duration: "live",
    itemsProcessed: event.attempts,
    error: event.has_error ? "事件投递包含错误" : undefined,
  }
}

function mapEventToLog(event: AdminOutboxEventListItem): AdminLogEntryReadModel {
  return {
    id: event.id,
    timestamp: event.occurred_at,
    level: event.has_error ? "error" : event.attempts > 0 ? "warn" : "info",
    source: event.kind.includes("playback")
      ? "playback"
      : event.kind.includes("auth")
        ? "auth"
        : event.kind.includes("scan")
          ? "scanner"
          : "server",
    message: `${event.kind} · ${event.status}`,
    details: event.has_payload ? "payload redacted by Admin API" : undefined,
    userId: undefined,
    requestId: event.id,
  }
}

function mapUserRole(roles: AdminUserRole[]): AdminUserUiRole {
  if (roles.includes("administrator")) {
    return "admin"
  }

  if (roles.includes("viewer")) {
    return "user"
  }

  return roles.length > 0 ? "user" : "guest"
}

function mapJobKind(kind: string): AdminTaskType {
  const normalized = kind.toLowerCase()
  if (normalized.includes("metadata")) return "metadata"
  if (normalized.includes("backup")) return "backup"
  if (normalized.includes("cleanup")) return "cleanup"
  if (normalized.includes("subtitle")) return "subtitle"
  if (normalized.includes("thumbnail") || normalized.includes("image")) return "thumbnail"
  if (normalized.includes("optimize")) return "optimize"
  if (normalized.includes("update")) return "update"
  return "scan"
}

function mapJobStatus(status: string): AdminTaskStatus {
  switch (status) {
    case "running":
      return "running"
    case "failed":
      return "failed"
    case "completed":
    case "succeeded":
      return "success"
    case "queued":
      return "scheduled"
    default:
      return "idle"
  }
}

function normalizeLibraryKind(value: string): AdminLibraryKind {
  const normalized = value.toLowerCase()
  if (normalized.includes("movie") || normalized.includes("film")) return "movie"
  if (normalized.includes("series") || normalized.includes("tv")) return "tv"
  if (normalized.includes("anime")) return "anime"
  if (normalized.includes("music")) return "music"
  if (normalized.includes("photo") || normalized.includes("image")) return "photo"
  if (normalized.includes("documentary")) return "documentary"
  if (normalized.includes("personal")) return "personal"
  return "unknown"
}

function firstMetadataLanguage(config: AdminServerConfigDiagnosticsResponse) {
  return config.metadata.providers.find((provider) => provider.language)?.language ?? "zh-CN"
}

function fixtureLibrary(
  id: string,
  name: string,
  type: AdminLibraryKind,
  path: string,
  itemCount: number,
  totalSize: string,
  lastScanned: string,
): AdminLibraryReadModel {
  return {
    id,
    name,
    type,
    paths: [{ path, available: true }],
    itemCount,
    totalSize,
    lastScanned,
    scanStatus: "idle",
    settings: {
      autoScan: true,
      scanInterval: 6,
      useNfo: true,
      downloadArt: true,
      metadataLanguage: type === "anime" ? "ja" : "zh-CN",
    },
  }
}

function fixtureUser(
  id: string,
  name: string,
  username: string,
  role: AdminUserUiRole,
  status: AdminUserUiStatus,
  libraryAccess: string[],
  lastActive: string,
): AdminUserReadModel {
  return {
    id,
    name,
    username,
    email: username === "guest" ? null : `${username}@example.com`,
    role,
    avatar: null,
    status,
    lastActive,
    createdAt: "2024-01-01",
    libraryAccess,
    settings: {
      canDownload: role !== "guest",
      canDelete: role === "admin",
      canManageUsers: role === "admin",
      maxBitrate: role === "admin" ? null : 20_000,
      transcodePolicy: "auto",
      maxStreams: role === "guest" ? 1 : null,
      remoteAccess: role !== "guest",
    },
    stats: {
      totalPlays: role === "admin" ? 1234 : 0,
      totalWatchTime: role === "admin" ? "156 小时" : "0 小时",
      lastLogin: status === "offline" ? "从未" : "2024-03-15 14:30",
    },
  }
}

function fixtureSession(
  id: string,
  userId: string,
  userName: string,
  device: string,
  deviceType: string,
  lastActivity: string,
  current: boolean,
): AdminActiveSessionReadModel {
  return {
    id,
    userId,
    userName,
    device,
    deviceType,
    ip: "192.168.1.100",
    location: "本地网络",
    lastActivity,
    startTime: "2024-03-15 14:30",
    current,
  }
}

function fixtureTask(
  id: string,
  name: string,
  description: string,
  type: AdminTaskType,
  status: AdminTaskStatus,
  enabled: boolean,
): AdminScheduledTaskReadModel {
  return {
    id,
    name,
    description,
    type,
    enabled,
    schedule: { frequency: "daily", time: "03:00" },
    config: {
      targetLibraries: ["1", "2", "3"],
      scanNewOnly: true,
      metadataLanguage: "zh-CN",
      downloadImages: true,
    },
    lastRun: {
      timestamp: "2024-03-15T03:00:00Z",
      status: "success",
      duration: "12分34秒",
      itemsProcessed: 847,
    },
    nextRun: "2024-03-16T03:00:00Z",
    status,
  }
}

function buildFixtureHistory(): AdminTaskHistoryReadModel[] {
  return Array.from({ length: 50 }, (_, index) => ({
    id: `history-${index}`,
    taskId: `task-${(index % 7) + 1}`,
    taskName: index % 2 === 0 ? "每日媒体库扫描" : "元数据刷新",
    taskType: index % 2 === 0 ? "scan" : "metadata",
    timestamp: new Date(Date.UTC(2024, 2, 15, 3 + (index % 12), 0, 0)).toISOString(),
    status: index % 10 === 0 ? "failed" : "success",
    duration: `${index % 30}分${index % 60}秒`,
    itemsProcessed: 100 + index,
    error: index % 10 === 0 ? "任务执行失败: 权限不足或网络超时" : undefined,
  }))
}

function buildFixtureLogs(): AdminLogEntryReadModel[] {
  const rows: AdminLogEntryReadModel[] = []
  const messages: Array<[AdminLogSource, AdminLogLevel, string]> = [
    ["server", "info", "Server started on port 8096"],
    ["auth", "warn", "Failed login attempt for user 'test'"],
    ["database", "info", "Database connection established"],
    ["api", "info", "GET /api/items - 200 OK (125ms)"],
    ["playback", "info", "Stream started: Movie 'Dune' (1080p)"],
    ["scanner", "error", "Error scanning: Permission denied"],
  ]

  for (let index = 0; index < 120; index += 1) {
    const [source, level, message] = messages[index % messages.length]
    rows.push({
      id: `log-${index}`,
      timestamp: new Date(Date.UTC(2024, 2, 15, 12, index % 60, 0)).toISOString(),
      level,
      source,
      message,
      details: level === "error" ? `Stack trace or additional details for log ${index}` : undefined,
      requestId: `req-${index}`,
    })
  }

  return rows.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
}

function formatMsDate(ms: number) {
  return new Date(ms).toISOString().slice(0, 10)
}

function isoFromMs(ms: number) {
  return new Date(ms).toISOString()
}
