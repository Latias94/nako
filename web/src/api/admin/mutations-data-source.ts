import { AdminApiClient } from "./client"
import { loadAdminApiConnection, type AdminApiConnection } from "./connection"
import {
  mapGeneratedArtifactMetadataBulkApplyBatchResponse,
  mapGeneratedArtifactMetadataApplyPlan,
  mapGeneratedArtifactReviewPlanResponse,
  mapMetadataCandidateReviewBatchResponse,
  mapMetadataCandidateReviewApplyResponse,
  type AdminMetadataCandidateReviewBatchReadModel,
  type AdminMetadataCandidateReviewApplyReadModel,
  type AdminGeneratedArtifactReviewDecision,
  type AdminGeneratedArtifactMetadataBulkApplyBatchReadModel,
  type AdminGeneratedArtifactMetadataApplyPlanReadModel,
  type AdminGeneratedArtifactReviewPlanReadModel,
} from "./read-models-data-source"
import type {
  AddonStatus,
  AdminCreateUserRequest,
  AdminGeneratedArtifactMetadataBulkApplyBatchResponse,
  AdminGeneratedArtifactMetadataApplyResponse,
  AdminGeneratedArtifactReviewPlanResponse,
  AdminUpdateMetadataRawCacheSettingsRequest,
  AdminUserRole,
  AdminUserStatus,
} from "./generated/contract"

export type AdminMutationKind =
  | "library.scan"
  | "library.nfo.import"
  | "library.nfo.export"
  | "user.create"
  | "user.roles.replace"
  | "user.status.update"
  | "user.password.set"
  | "user.password.delete"
  | "settings.metadata.raw-cache.update"
  | "addon.status.update"
  | "generated-artifact.review"
  | "generated-artifact.metadata-apply"
  | "generated-artifact.metadata-bulk-apply"
  | "metadata-candidate-review.batch-create"

export interface AdminMutationResult {
  kind: AdminMutationKind
  id: string
  message: string
}

export interface AdminGeneratedArtifactReviewMutationResult {
  kind: "generated-artifact.review"
  artifactId: string
  decision: AdminGeneratedArtifactReviewDecision
  artifactStatus: string
  acceptedAt: string | null
  idempotentReplay: boolean
  message: string
  plan: AdminGeneratedArtifactReviewPlanReadModel
}

export interface AdminGeneratedArtifactMetadataApplyMutationResult {
  kind: "generated-artifact.metadata-apply"
  artifactId: string
  outcomeId: string | null
  status: string
  applied: boolean
  changed: boolean
  idempotentReplay: boolean
  appliedSource: string | null
  message: string
  plan: AdminGeneratedArtifactMetadataApplyPlanReadModel
}

export interface AdminGeneratedArtifactMetadataBulkApplyMutationResult
  extends AdminGeneratedArtifactMetadataBulkApplyBatchReadModel {
  kind: "generated-artifact.metadata-bulk-apply"
  message: string
}

export interface AdminMetadataCandidateReviewApplyMutationRequest {
  itemId: string
  expectedUpdatedAtMs: number | null
  idempotencyKey: string
}

export interface AdminMetadataCandidateReviewApplyMutationResult
  extends AdminMetadataCandidateReviewApplyReadModel {
  kind: "metadata-candidate-review.apply"
  message: string
}

export interface AdminMetadataCandidateReviewBatchMutationRequestItem {
  reviewId: string
  itemId: string
  expectedUpdatedAtMs: number | null
}

export interface AdminMetadataCandidateReviewBatchCreateMutationResult
  extends AdminMetadataCandidateReviewBatchReadModel {
  kind: "metadata-candidate-review.batch-create"
  message: string
}

export interface AdminMutationDataSource {
  canMutate: boolean
  unavailableReason?: string
  scanLibrary(libraryId: string): Promise<AdminMutationResult>
  importLibraryNfo(libraryId: string): Promise<AdminMutationResult>
  exportLibraryNfo(libraryId: string): Promise<AdminMutationResult>
  createUser(request: AdminCreateUserRequest): Promise<AdminMutationResult>
  replaceUserRoles(userId: string, roles: AdminUserRole[]): Promise<AdminMutationResult>
  updateUserStatus(userId: string, status: AdminUserStatus): Promise<AdminMutationResult>
  setUserLocalPassword(userId: string, password: string): Promise<AdminMutationResult>
  deleteUserLocalPassword(userId: string): Promise<AdminMutationResult>
  updateMetadataRawCacheSettings(
    request: AdminUpdateMetadataRawCacheSettingsRequest,
  ): Promise<AdminMutationResult>
  updateAddonStatus(addonId: string, status: AddonStatus): Promise<AdminMutationResult>
  reviewGeneratedArtifact(
    artifactId: string,
    decision: AdminGeneratedArtifactReviewDecision,
  ): Promise<AdminGeneratedArtifactReviewMutationResult>
  applyGeneratedArtifactMetadata(
    artifactId: string,
    idempotencyKey: string,
  ): Promise<AdminGeneratedArtifactMetadataApplyMutationResult>
  confirmGeneratedArtifactMetadataBulkApplyBatch(
    artifactIds: string[],
    idempotencyKey: string,
  ): Promise<AdminGeneratedArtifactMetadataBulkApplyMutationResult>
  applyMetadataCandidateReview(
    reviewId: string,
    request: AdminMetadataCandidateReviewApplyMutationRequest,
  ): Promise<AdminMetadataCandidateReviewApplyMutationResult>
  createMetadataCandidateReviewBatch(
    reviews: AdminMetadataCandidateReviewBatchMutationRequestItem[],
    idempotencyKey: string,
  ): Promise<AdminMetadataCandidateReviewBatchCreateMutationResult>
}

export function createAdminMutationDataSource(
  connection: AdminApiConnection = loadAdminApiConnection(),
  fetcher?: typeof fetch,
): AdminMutationDataSource {
  if (connection.mode === "fixture") {
    return disabledMutationDataSource("连接 live Admin API 后才能执行管理操作")
  }

  const client = new AdminApiClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetcher,
  })

  return {
    canMutate: true,

    async scanLibrary(libraryId) {
      const job = await client.requestLibraryScan(libraryId)
      return mutationResult("library.scan", job.id, "媒体库扫描已提交")
    },

    async importLibraryNfo(libraryId) {
      const job = await client.requestLibraryNfoImport(libraryId)
      return mutationResult("library.nfo.import", job.id, "NFO 导入已提交")
    },

    async exportLibraryNfo(libraryId) {
      const job = await client.requestLibraryNfoExport(libraryId)
      return mutationResult("library.nfo.export", job.id, "NFO 导出已提交")
    },

    async createUser(request) {
      const response = await client.createAccessUser(request)
      return mutationResult("user.create", response.user.user_id, "用户已创建")
    },

    async replaceUserRoles(userId, roles) {
      const response = await client.replaceAccessUserRoles(userId, { roles })
      return mutationResult("user.roles.replace", response.user.user_id, "用户角色已更新")
    },

    async updateUserStatus(userId, status) {
      const response = await client.updateAccessUserStatus(userId, { status })
      return mutationResult(
        "user.status.update",
        response.user.user_id,
        status === "disabled" ? "用户已禁用" : "用户已启用",
      )
    },

    async setUserLocalPassword(userId, password) {
      const response = await client.setAccessUserLocalPassword(userId, { password })
      return mutationResult("user.password.set", response.user_id, "本地密码已更新")
    },

    async deleteUserLocalPassword(userId) {
      const response = await client.deleteAccessUserLocalPassword(userId)
      return mutationResult("user.password.delete", response.user_id, "本地密码已移除")
    },

    async updateMetadataRawCacheSettings(request) {
      const response = await client.updateMetadataRawCacheSettings(request)
      return mutationResult(
        "settings.metadata.raw-cache.update",
        "metadata-raw-cache",
        `元数据缓存设置已更新: ${response.effect}`,
      )
    },

    async updateAddonStatus(addonId, status) {
      const response = await client.updateAddonStatus(addonId, { status })
      return mutationResult(
        "addon.status.update",
        response.addon.summary.id,
        status === "enabled" ? "Addon 已启用" : "Addon 已禁用",
      )
    },

    async reviewGeneratedArtifact(artifactId, decision) {
      const response = await client.reviewGeneratedArtifact(artifactId, { decision })
      return {
        kind: "generated-artifact.review",
        artifactId: response.artifact_id,
        decision: response.decision,
        artifactStatus: response.artifact_status,
        acceptedAt: response.accepted_at,
        idempotentReplay: response.idempotent_replay,
        message: generatedArtifactReviewMessage(response.decision, response.idempotent_replay),
        plan: mapGeneratedArtifactReviewPlanResponse(reviewResponseToPlanResponse(response)),
      }
    },

    async applyGeneratedArtifactMetadata(artifactId, idempotencyKey) {
      const response = await client.applyGeneratedArtifactMetadata(artifactId, {
        idempotency_key: idempotencyKey,
      })
      return mapGeneratedArtifactMetadataApplyMutationResponse(response)
    },

    async confirmGeneratedArtifactMetadataBulkApplyBatch(artifactIds, idempotencyKey) {
      const response = await client.createGeneratedArtifactMetadataBulkApplyBatch({
        artifact_ids: artifactIds,
        idempotency_key: idempotencyKey,
      })
      return mapGeneratedArtifactMetadataBulkApplyMutationResponse(response)
    },

    async applyMetadataCandidateReview(reviewId, request) {
      const response = await client.applyMetadataCandidateReview(reviewId, {
        item_id: request.itemId,
        expected_updated_at_ms: request.expectedUpdatedAtMs,
        idempotency_key: request.idempotencyKey,
      })
      return mapMetadataCandidateReviewApplyMutationResponse(response)
    },

    async createMetadataCandidateReviewBatch(reviews, idempotencyKey) {
      const response = await client.createMetadataCandidateReviewBatch({
        idempotency_key: idempotencyKey,
        reviews: reviews.map((review) => ({
          review_id: review.reviewId,
          item_id: review.itemId,
          expected_updated_at_ms: review.expectedUpdatedAtMs,
        })),
      })
      return mapMetadataCandidateReviewBatchCreateMutationResponse(response)
    },
  }
}

function disabledMutationDataSource(reason: string): AdminMutationDataSource {
  const reject = async (): Promise<AdminMutationResult> => {
    throw new Error(reason)
  }
  const rejectGeneratedArtifactReview =
    async (): Promise<AdminGeneratedArtifactReviewMutationResult> => {
      throw new Error(reason)
    }
  const rejectGeneratedArtifactMetadataApply =
    async (): Promise<AdminGeneratedArtifactMetadataApplyMutationResult> => {
      throw new Error(reason)
    }
  const rejectGeneratedArtifactMetadataBulkApply =
    async (): Promise<AdminGeneratedArtifactMetadataBulkApplyMutationResult> => {
      throw new Error(reason)
    }
  const rejectMetadataCandidateReviewApply =
    async (): Promise<AdminMetadataCandidateReviewApplyMutationResult> => {
      throw new Error(reason)
    }
  const rejectMetadataCandidateReviewBatchCreate =
    async (): Promise<AdminMetadataCandidateReviewBatchCreateMutationResult> => {
      throw new Error(reason)
    }

  return {
    canMutate: false,
    unavailableReason: reason,
    scanLibrary: reject,
    importLibraryNfo: reject,
    exportLibraryNfo: reject,
    createUser: reject,
    replaceUserRoles: reject,
    updateUserStatus: reject,
    setUserLocalPassword: reject,
    deleteUserLocalPassword: reject,
    updateMetadataRawCacheSettings: reject,
    updateAddonStatus: reject,
    reviewGeneratedArtifact: rejectGeneratedArtifactReview,
    applyGeneratedArtifactMetadata: rejectGeneratedArtifactMetadataApply,
    confirmGeneratedArtifactMetadataBulkApplyBatch: rejectGeneratedArtifactMetadataBulkApply,
    applyMetadataCandidateReview: rejectMetadataCandidateReviewApply,
    createMetadataCandidateReviewBatch: rejectMetadataCandidateReviewBatchCreate,
  }
}

function mutationResult(kind: AdminMutationKind, id: string, message: string): AdminMutationResult {
  return {
    kind,
    id,
    message,
  }
}

function reviewResponseToPlanResponse(
  response: {
    admin_api_version: string
    public_api_version: string
    plan: AdminGeneratedArtifactReviewPlanResponse["plan"]
  },
): AdminGeneratedArtifactReviewPlanResponse {
  return {
    admin_api_version: response.admin_api_version,
    public_api_version: response.public_api_version,
    plan: response.plan,
  }
}

function generatedArtifactReviewMessage(
  decision: AdminGeneratedArtifactReviewDecision,
  idempotentReplay: boolean,
) {
  if (idempotentReplay) {
    return "审核结果已存在，已返回幂等结果"
  }

  return decision === "accept" ? "生成产物已接受" : "生成产物已拒绝"
}

function mapGeneratedArtifactMetadataApplyMutationResponse(
  response: AdminGeneratedArtifactMetadataApplyResponse,
): AdminGeneratedArtifactMetadataApplyMutationResult {
  return {
    kind: "generated-artifact.metadata-apply",
    artifactId: response.artifact_id,
    outcomeId: response.outcome_id,
    status: response.status,
    applied: response.applied,
    changed: response.changed,
    idempotentReplay: response.idempotent_replay,
    appliedSource: response.applied_source,
    message: generatedArtifactMetadataApplyMessage(response),
    plan: mapGeneratedArtifactMetadataApplyPlan(response.plan, {
      adminApi: response.admin_api_version,
      publicApi: response.public_api_version,
    }),
  }
}

function generatedArtifactMetadataApplyMessage(response: AdminGeneratedArtifactMetadataApplyResponse) {
  if (response.idempotent_replay) {
    return "元数据应用结果已存在，已返回幂等结果"
  }

  if (!response.applied) {
    return response.status === "noop" ? "没有可应用的元数据变更" : "元数据应用未执行"
  }

  return response.changed ? "Canonical Metadata 已更新" : "元数据应用完成，没有字段变更"
}

function mapGeneratedArtifactMetadataBulkApplyMutationResponse(
  response: AdminGeneratedArtifactMetadataBulkApplyBatchResponse,
): AdminGeneratedArtifactMetadataBulkApplyMutationResult {
  const batch = mapGeneratedArtifactMetadataBulkApplyBatchResponse(response)

  return {
    ...batch,
    kind: "generated-artifact.metadata-bulk-apply",
    message: generatedArtifactMetadataBulkApplyMessage(batch.status),
  }
}

function generatedArtifactMetadataBulkApplyMessage(status: string) {
  switch (status) {
    case "completed":
      return "批量元数据应用已完成"
    case "failed":
      return "批量元数据应用失败"
    case "cancelled":
      return "批量元数据应用已取消"
    default:
      return "批量元数据应用批次已提交"
  }
}

function mapMetadataCandidateReviewApplyMutationResponse(
  response: Parameters<typeof mapMetadataCandidateReviewApplyResponse>[0],
): AdminMetadataCandidateReviewApplyMutationResult {
  const result = mapMetadataCandidateReviewApplyResponse(response)

  return {
    ...result,
    kind: "metadata-candidate-review.apply",
    message: metadataCandidateReviewApplyMessage(result),
  }
}

function mapMetadataCandidateReviewBatchCreateMutationResponse(
  response: Parameters<typeof mapMetadataCandidateReviewBatchResponse>[0],
): AdminMetadataCandidateReviewBatchCreateMutationResult {
  const result = mapMetadataCandidateReviewBatchResponse(response)

  return {
    ...result,
    kind: "metadata-candidate-review.batch-create",
    message: metadataCandidateReviewBatchMessage(),
  }
}

function metadataCandidateReviewApplyMessage(result: AdminMetadataCandidateReviewApplyReadModel) {
  if (result.idempotentReplay) {
    return "Candidate Review 已应用，未产生新的 Provider Mapping 变更"
  }

  if (!result.applied) {
    return "Candidate Review 应用未执行"
  }

  return result.changed
    ? "Candidate Review 已应用到 root Provider Mapping"
    : "Candidate Review 已应用，没有 Provider Mapping 变更"
}

function metadataCandidateReviewBatchMessage() {
  return "批量 Candidate Review 批次已提交"
}
