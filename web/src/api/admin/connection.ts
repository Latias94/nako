import {
  loadConnectionState,
  saveApiClientConnection,
  toApiClientConnection,
  type ApiClientConnection,
} from "@/src/api/connection-profile"

export type AdminApiConnection = ApiClientConnection

export function loadAdminApiConnection(): AdminApiConnection {
  return toApiClientConnection(loadConnectionState())
}

export function saveAdminApiConnection(connection: AdminApiConnection) {
  saveApiClientConnection(connection)
}
