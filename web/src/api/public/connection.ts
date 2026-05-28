import {
  loadConnectionState,
  saveApiClientConnection,
  toApiClientConnection,
  type ApiClientConnection,
} from "@/src/api/connection-profile"

export type PublicClientConnection = ApiClientConnection

export function loadPublicClientConnection(): PublicClientConnection {
  return toApiClientConnection(loadConnectionState())
}

export function savePublicClientConnection(connection: PublicClientConnection) {
  saveApiClientConnection(connection)
}
