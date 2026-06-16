import type { AdminApiClientOptions } from "./client";
import type { AdminDataSource } from "./dataSource";

type AdminDataSourceMethod = {
  [K in keyof AdminDataSource]: NonNullable<AdminDataSource[K]> extends (
    ...args: never[]
  ) => Promise<unknown>
    ? K
    : never;
}[keyof AdminDataSource];

const DEFAULT_ADMIN_DATA_SOURCE_METHODS = [
  "load",
  "loadAccessSummary",
  "loadAccessInvitations",
  "createAccessInvitation",
  "revokeAccessInvitation",
  "loadOverview",
  "loadOperatorReadiness",
  "loadIncidentBundle",
  "loadEvents",
  "loadAddonEventDeliveryAttempts",
  "loadAddonEventSchedulerWork",
  "deliverAddonEvents",
  "replayAddonEvents",
  "loadAddons",
  "loadAddonTaskRuns",
  "retryAddonTaskRun",
  "loadJobs",
  "cancelJob",
  "loadLibraries",
  "loadLibraryDetail",
  "loadSettings",
  "loadMetadataRawCacheSettings",
  "updateMetadataRawCacheSettings",
  "loadAcquisitionIntake",
  "loadGeneratedArtifacts",
  "loadGeneratedArtifactReviewPlan",
  "reviewGeneratedArtifact",
  "loadItemArtworkGallery",
  "loadManagedArtworkMaintenance",
  "selectItemArtwork",
  "unpublishItemArtwork",
  "loadCatalog",
  "loadItemDetail",
  "loadCatalogGovernance",
  "loadCatalogGovernanceItemDetail",
  "loadCatalogGovernanceProviderMappingReviewPlan",
  "reviewCatalogGovernanceProviderMapping",
  "loadPlaybackSessions",
  "loadPlaybackSupport",
  "loadPlaybackRuntimeSettings",
  "updatePlaybackRuntimeSettings",
  "loadSourceDuplicateReconciliationPlan",
  "applySourceDuplicateReconciliation",
  "loadStorageStaging",
  "loadVfsCacheRepairActionPlan",
  "loadVfsCacheRepairRemediationPlan",
  "loadVfsCacheRepairAutomationPlan",
  "loadVfsCacheRepairTargets",
  "refreshLatestVfsCacheRepair",
  "enqueueVfsCacheRepairTarget",
  "enqueueVfsCacheRepairAutomation",
  "executeVfsCacheRepairJob",
  "retryVfsCacheRepairJob",
  "updateLibraryMetadataProfile",
  "runLibraryCommand",
  "setAddonStatus",
  "checkAddonHealth",
  "diagnoseAddonResource",
  "registerAddonManifestJson",
  "issueAddonToken",
  "rotateAddonToken",
  "revokeAddonToken",
  "replaceAddonGrants",
] as const satisfies readonly AdminDataSourceMethod[];

export function createLazyAdminDataSource(
  options: AdminApiClientOptions = {},
): AdminDataSource {
  let dataSourcePromise: Promise<AdminDataSource> | null = null;
  const loadDataSource = () => {
    dataSourcePromise ??= import("./dataSource").then((module) =>
      module.createAdminDataSource(options),
    );
    return dataSourcePromise;
  };

  return Object.fromEntries(
    DEFAULT_ADMIN_DATA_SOURCE_METHODS.map((method) => [
      method,
      async (...args: unknown[]) => {
        const dataSource = await loadDataSource();
        const delegate = dataSource[method];

        if (typeof delegate !== "function") {
          throw new Error(`Admin data source method is unavailable: ${method}`);
        }

        return (delegate as (...args: unknown[]) => Promise<unknown>)(...args);
      },
    ]),
  ) as unknown as AdminDataSource;
}
