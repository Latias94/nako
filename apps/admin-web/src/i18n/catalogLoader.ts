import { baseMessageCatalogs } from "./catalogs/base";
import type { AdminLocale, I18nNamespace, MessageId } from "./messages";

export type MessageCatalog = Readonly<Partial<Record<MessageId, string>>>;
export type LocaleMessageCatalogs = Readonly<
  Record<AdminLocale, MessageCatalog>
>;

export const baseCatalogs: LocaleMessageCatalogs = baseMessageCatalogs;

const namespaceCatalogLoaders = {
  overview: () =>
    import("./catalogs/overview").then(
      (module) => module.overviewMessageCatalogs,
    ),
  incidentBundle: () =>
    import("./catalogs/incidentBundle").then(
      (module) => module.incidentBundleMessageCatalogs,
    ),
  operatorReadiness: () =>
    import("./catalogs/operatorReadiness").then(
      (module) => module.operatorReadinessMessageCatalogs,
    ),
  access: () =>
    import("./catalogs/access").then((module) => module.accessMessageCatalogs),
  settings: () =>
    import("./catalogs/settings").then(
      (module) => module.settingsMessageCatalogs,
    ),
  catalogGovernance: () =>
    import("./catalogs/catalogGovernance").then(
      (module) => module.catalogGovernanceMessageCatalogs,
    ),
  itemDetail: () =>
    import("./catalogs/itemDetail").then(
      (module) => module.itemDetailMessageCatalogs,
    ),
  sourceDuplicate: () =>
    import("./catalogs/sourceDuplicate").then(
      (module) => module.sourceDuplicateMessageCatalogs,
    ),
  itemArtwork: () =>
    import("./catalogs/itemArtwork").then(
      (module) => module.itemArtworkMessageCatalogs,
    ),
  artworkMaintenance: () =>
    import("./catalogs/artworkMaintenance").then(
      (module) => module.artworkMaintenanceMessageCatalogs,
    ),
  generatedArtifactReview: () =>
    import("./catalogs/generatedArtifactReview").then(
      (module) => module.generatedArtifactReviewMessageCatalogs,
    ),
  jobs: () =>
    import("./catalogs/jobs").then((module) => module.jobsMessageCatalogs),
  playback: () =>
    import("./catalogs/playback").then(
      (module) => module.playbackMessageCatalogs,
    ),
  playbackSupport: () =>
    import("./catalogs/playbackSupport").then(
      (module) => module.playbackSupportMessageCatalogs,
    ),
  storage: () =>
    import("./catalogs/storage").then(
      (module) => module.storageMessageCatalogs,
    ),
  catalogBrowse: () =>
    import("./catalogs/catalogBrowse").then(
      (module) => module.catalogBrowseMessageCatalogs,
    ),
  acquisition: () =>
    import("./catalogs/acquisition").then(
      (module) => module.acquisitionMessageCatalogs,
    ),
  generatedArtifacts: () =>
    import("./catalogs/generatedArtifacts").then(
      (module) => module.generatedArtifactsMessageCatalogs,
    ),
  addons: () =>
    import("./catalogs/addons").then((module) => module.addonsMessageCatalogs),
  libraries: () =>
    import("./catalogs/libraries").then(
      (module) => module.librariesMessageCatalogs,
    ),
  libraryDetail: () =>
    import("./catalogs/libraryDetail").then(
      (module) => module.libraryDetailMessageCatalogs,
    ),
  events: () =>
    import("./catalogs/events").then((module) => module.eventsMessageCatalogs),
} satisfies Record<I18nNamespace, () => Promise<LocaleMessageCatalogs>>;

export function loadCatalogNamespace(namespace: I18nNamespace) {
  return namespaceCatalogLoaders[namespace]();
}
