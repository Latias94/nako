import { AdminApiClient } from "./client"
import { loadAdminApiConnection, type AdminApiConnection } from "./connection"
import type {
  AddonResource,
  AddonScope,
  AddonStatus,
  AdminAddonRegistrationSummary,
  AdminAddonSourceCatalogEntry,
  AdminAddonSourceCatalogSource,
} from "./generated/contract"
import type { AdminReadModelEnvelope } from "./read-models-data-source"

export interface AdminAddonManagerReadModel extends AdminReadModelEnvelope {
  installed: AdminAddonInstalledReadModel[]
  catalog: AdminAddonCatalogEntryReadModel[]
  sources: AdminAddonCatalogSourceReadModel[]
}

export interface AdminAddonInstalledReadModel {
  id: string
  manifestId: string
  name: string
  version: string
  protocolVersion: string
  baseUrl: string
  status: AddonStatus
  grantedScopes: string[]
  createdAt: string
  updatedAt: string
}

export interface AdminAddonCatalogEntryReadModel {
  sourceId: string
  entryId: string
  manifestId: string
  name: string
  version: string
  protocolVersion: string
  description?: string
  runtimeKind: string
  resources: AddonResource[]
  scopes: AddonScope[]
  tasks: string[]
  packageSigningVerified: boolean
  lifecycleBoundary: {
    nako_manages_containers: boolean
    nako_manages_processes: boolean
    nako_manages_packages: boolean
    message: string
  }
  installedStatus?: AddonStatus
}

export interface AdminAddonCatalogSourceReadModel {
  id: string
  name: string
  description?: string
  entryCount: number
  providesPackageSigning: boolean
  providesProcessSupervision: boolean
  providesProviderBreadth: boolean
}

export const ADMIN_ADDON_MANAGER_FIXTURE: AdminAddonManagerReadModel = {
  source: "fixture",
  fallback: true,
  installed: [
    {
      id: "fixture-addon-tmdb",
      manifestId: "nako.tmdb",
      name: "TMDb Metadata Sidecar",
      version: "0.1.0",
      protocolVersion: "v1",
      baseUrl: "http://127.0.0.1:9101",
      status: "enabled",
      grantedScopes: ["catalog_read", "item_metadata_read", "item_metadata_suggest"],
      createdAt: "2026-05-28T00:00:00Z",
      updatedAt: "2026-05-28T00:00:00Z",
    },
  ],
  catalog: [
    {
      sourceId: "nako-official",
      entryId: "nako.tmdb",
      manifestId: "nako.tmdb",
      name: "TMDb Metadata Sidecar",
      version: "0.1.0",
      protocolVersion: "v1",
      description: "Metadata and artwork suggestions through the Nako Addon Protocol.",
      runtimeKind: "http_sidecar",
      resources: ["metadata", "image"],
      scopes: ["catalog_read", "item_metadata_read", "item_metadata_suggest", "image_read"],
      tasks: ["refresh-metadata"],
      packageSigningVerified: false,
      lifecycleBoundary: {
        nako_manages_containers: false,
        nako_manages_processes: false,
        nako_manages_packages: false,
        message: "Manual sidecar lifecycle boundary.",
      },
      installedStatus: "enabled",
    },
    {
      sourceId: "nako-official",
      entryId: "nako.bangumi",
      manifestId: "nako.bangumi",
      name: "Bangumi Metadata Sidecar",
      version: "0.1.0",
      protocolVersion: "v1",
      description: "Anime metadata lookup and subject matching.",
      runtimeKind: "http_sidecar",
      resources: ["metadata"],
      scopes: ["catalog_read", "item_metadata_read", "item_metadata_suggest"],
      tasks: [],
      packageSigningVerified: false,
      lifecycleBoundary: {
        nako_manages_containers: false,
        nako_manages_processes: false,
        nako_manages_packages: false,
        message: "Manual sidecar lifecycle boundary.",
      },
    },
  ],
  sources: [
    {
      id: "nako-official",
      name: "Nako Official",
      description: "Built-in official addon catalog.",
      entryCount: 2,
      providesPackageSigning: false,
      providesProcessSupervision: false,
      providesProviderBreadth: true,
    },
  ],
}

export function createAdminAddonManagerDataSource(
  connection: AdminApiConnection = loadAdminApiConnection(),
  fetcher?: typeof fetch,
) {
  if (connection.mode === "fixture") {
    return {
      async loadAddonManager() {
        return ADMIN_ADDON_MANAGER_FIXTURE
      },
    }
  }

  const client = new AdminApiClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetcher,
  })

  return {
    async loadAddonManager(): Promise<AdminAddonManagerReadModel> {
      try {
        const [installed, sources, catalog] = await Promise.all([
          client.getAddons(),
          client.getAddonCatalogSources(),
          client.getAddonCatalogEntries(),
        ])
        return mapAddonManager(installed.addons, sources.sources, catalog.entries)
      } catch (error) {
        return {
          ...ADMIN_ADDON_MANAGER_FIXTURE,
          source: "fixture",
          fallback: true,
          error: error instanceof Error ? error.message : "Admin Addon API request failed",
        }
      }
    },
  }
}

function mapAddonManager(
  installed: AdminAddonRegistrationSummary[],
  sources: AdminAddonSourceCatalogSource[],
  catalog: AdminAddonSourceCatalogEntry[],
): AdminAddonManagerReadModel {
  const installedByManifest = new Map(installed.map((addon) => [addon.manifest_id, addon.status]))

  return {
    source: "live",
    fallback: false,
    installed: installed.map(mapInstalledAddon),
    catalog: catalog.map((entry) => mapCatalogEntry(entry, installedByManifest.get(entry.manifest_id))),
    sources: sources.map((source) => ({
      id: source.id,
      name: source.name,
      description: source.description,
      entryCount: source.entry_count,
      providesPackageSigning: source.provides_package_signing,
      providesProcessSupervision: source.provides_process_supervision,
      providesProviderBreadth: source.provides_provider_breadth,
    })),
  }
}

function mapInstalledAddon(addon: AdminAddonRegistrationSummary): AdminAddonInstalledReadModel {
  return {
    id: addon.id,
    manifestId: addon.manifest_id,
    name: addon.name,
    version: addon.version,
    protocolVersion: addon.protocol_version,
    baseUrl: addon.base_url,
    status: addon.status,
    grantedScopes: addon.granted_scopes,
    createdAt: addon.created_at,
    updatedAt: addon.updated_at,
  }
}

function mapCatalogEntry(
  entry: AdminAddonSourceCatalogEntry,
  installedStatus?: AddonStatus,
): AdminAddonCatalogEntryReadModel {
  return {
    sourceId: entry.source_id,
    entryId: entry.entry_id,
    manifestId: entry.manifest_id,
    name: entry.addon_name,
    version: entry.addon_version,
    protocolVersion: entry.protocol_version,
    description: entry.description,
    runtimeKind: entry.runtime_kind,
    resources: entry.resources,
    scopes: entry.scopes,
    tasks: entry.tasks,
    packageSigningVerified: entry.package_signing_verified,
    lifecycleBoundary: entry.lifecycle_boundary,
    installedStatus,
  }
}
