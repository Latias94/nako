import { AdminApiClient } from "./client"
import { loadAdminApiConnection, type AdminApiConnection } from "./connection"
import type {
  AddonStatus,
  AdminCreateUserRequest,
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

export interface AdminMutationResult {
  kind: AdminMutationKind
  id: string
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
  }
}

function disabledMutationDataSource(reason: string): AdminMutationDataSource {
  const reject = async (): Promise<AdminMutationResult> => {
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
  }
}

function mutationResult(kind: AdminMutationKind, id: string, message: string): AdminMutationResult {
  return {
    kind,
    id,
    message,
  }
}
