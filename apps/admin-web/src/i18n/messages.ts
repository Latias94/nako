import type { enBaseMessages } from "./catalogs/base";
import type { enOverviewMessages } from "./catalogs/overview";
import type { enIncidentBundleMessages } from "./catalogs/incidentBundle";
import type { enOperatorReadinessMessages } from "./catalogs/operatorReadiness";
import type { enAccessMessages } from "./catalogs/access";
import type { enSettingsMessages } from "./catalogs/settings";
import type { enCatalogGovernanceMessages } from "./catalogs/catalogGovernance";
import type { enItemDetailMessages } from "./catalogs/itemDetail";
import type { enSourceDuplicateMessages } from "./catalogs/sourceDuplicate";
import type { enItemArtworkMessages } from "./catalogs/itemArtwork";
import type { enArtworkMaintenanceMessages } from "./catalogs/artworkMaintenance";
import type { enGeneratedArtifactReviewMessages } from "./catalogs/generatedArtifactReview";
import type { enJobsMessages } from "./catalogs/jobs";
import type { enPlaybackMessages } from "./catalogs/playback";
import type { enPlaybackSupportMessages } from "./catalogs/playbackSupport";
import type { enStorageMessages } from "./catalogs/storage";
import type { enCatalogBrowseMessages } from "./catalogs/catalogBrowse";
import type { enAcquisitionMessages } from "./catalogs/acquisition";
import type { enGeneratedArtifactsMessages } from "./catalogs/generatedArtifacts";
import type { enAddonsMessages } from "./catalogs/addons";
import type { enLibrariesMessages } from "./catalogs/libraries";
import type { enLibraryDetailMessages } from "./catalogs/libraryDetail";
import type { enEventsMessages } from "./catalogs/events";

export type AdminLocale = "en-US" | "zh-Hans";

export type I18nNamespace =
  | "overview"
  | "incidentBundle"
  | "operatorReadiness"
  | "access"
  | "settings"
  | "catalogGovernance"
  | "itemDetail"
  | "sourceDuplicate"
  | "itemArtwork"
  | "artworkMaintenance"
  | "generatedArtifactReview"
  | "jobs"
  | "playback"
  | "playbackSupport"
  | "storage"
  | "catalogBrowse"
  | "acquisition"
  | "generatedArtifacts"
  | "addons"
  | "libraries"
  | "libraryDetail"
  | "events";

export type MessageId =
  | keyof typeof enBaseMessages
  | keyof typeof enOverviewMessages
  | keyof typeof enIncidentBundleMessages
  | keyof typeof enOperatorReadinessMessages
  | keyof typeof enAccessMessages
  | keyof typeof enSettingsMessages
  | keyof typeof enCatalogGovernanceMessages
  | keyof typeof enItemDetailMessages
  | keyof typeof enSourceDuplicateMessages
  | keyof typeof enItemArtworkMessages
  | keyof typeof enArtworkMaintenanceMessages
  | keyof typeof enGeneratedArtifactReviewMessages
  | keyof typeof enJobsMessages
  | keyof typeof enPlaybackMessages
  | keyof typeof enPlaybackSupportMessages
  | keyof typeof enStorageMessages
  | keyof typeof enCatalogBrowseMessages
  | keyof typeof enAcquisitionMessages
  | keyof typeof enGeneratedArtifactsMessages
  | keyof typeof enAddonsMessages
  | keyof typeof enLibrariesMessages
  | keyof typeof enLibraryDetailMessages
  | keyof typeof enEventsMessages;
