import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  Outlet,
  RouterProvider,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  redirect,
  useRouterState,
} from "@tanstack/react-router";
import {
  Activity,
  Database,
  Film,
  HardDrive,
  Inbox,
  Library,
  ListChecks,
  PlayCircle,
  Puzzle,
  Settings,
  Sparkles,
  UsersRound,
} from "lucide-react";
import { useMemo, useState } from "react";

import type { AdminDataSource } from "./adminApi/dataSource";
import { AdminShell, type AdminShellNavItem } from "./components/layout/AdminShell";
import { I18nProvider, useI18n } from "./i18n/I18nProvider";
import type { AdminLocale, MessageId } from "./i18n/messages";
import {
  AcquisitionIntakePage,
  type AcquisitionIntakeSearch,
} from "./features/acquisition/AcquisitionIntakePage";
import { AccessPage } from "./features/access/AccessPage";
import { AddonsPage, type AddonsSearch } from "./features/addons/AddonsPage";
import {
  GeneratedArtifactsPage,
  type GeneratedArtifactsSearch,
} from "./features/automation/GeneratedArtifactsPage";
import {
  GeneratedArtifactReviewPage,
  type GeneratedArtifactReviewSearch,
} from "./features/automation/GeneratedArtifactReviewPage";
import {
  CatalogBrowsePage,
  type CatalogSearch,
} from "./features/catalog/CatalogBrowsePage";
import {
  CatalogGovernancePage,
  type CatalogGovernanceSearch,
} from "./features/catalog/CatalogGovernancePage";
import {
  CatalogGovernanceRepairPage,
  type CatalogGovernanceRepairSearch,
} from "./features/catalog/CatalogGovernanceRepairPage";
import { JobsPage, type JobsSearch } from "./features/jobs/JobsPage";
import {
  ItemArtworkGalleryPage,
  type ItemArtworkGallerySearch,
} from "./features/items/ItemArtworkGalleryPage";
import { ItemDetailPage } from "./features/items/ItemDetailPage";
import { LibraryDetailPage } from "./features/libraries/LibraryDetailPage";
import { LibrariesPage } from "./features/libraries/LibrariesPage";
import { OverviewPage } from "./features/overview/OverviewPage";
import {
  PlaybackSessionsPage,
  type PlaybackSessionsSearch,
} from "./features/playback/PlaybackSessionsPage";
import {
  StorageStagingPage,
  type StorageStagingSearch,
} from "./features/storage/StorageStagingPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { LegacyDashboard } from "./legacy/LegacyDashboard";
import type { AddonStatus } from "./adminApi/types";
import { MediaShell } from "./surfaces/media/MediaShell";
import { MediaSessionProvider } from "./surfaces/media/MediaSession";
import {
  MediaConnectPage,
  MediaHomePage,
  MediaLibrariesPage,
  MediaSearchPage,
} from "./surfaces/media/MediaPages";
import {
  createMediaWebDataSource,
  type MediaDataSourceFactory,
} from "./surfaces/media/mediaDataSource";

type RouterContext = {
  dataSource: AdminDataSource;
  mediaDataSourceFactory: MediaDataSourceFactory;
};

const rootRoute = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/overview" });
  },
});

const jobsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/jobs",
  validateSearch: validateJobsSearch,
  component: JobsRoute,
});

const accessRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/access",
  component: AccessRoute,
});

const overviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/overview",
  component: OverviewRoute,
});
const librariesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/libraries",
  component: LibrariesRoute,
});
const libraryDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/libraries/$libraryId",
  component: LibraryDetailRoute,
});
const catalogBrowseRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/catalog",
  validateSearch: validateCatalogSearch,
  component: CatalogBrowseRoute,
});
const catalogGovernanceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/catalog/governance",
  validateSearch: validateCatalogGovernanceSearch,
  component: CatalogGovernanceRoute,
});
const catalogGovernanceRepairRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/catalog/governance/$itemId",
  validateSearch: validateCatalogGovernanceRepairSearch,
  component: CatalogGovernanceRepairRoute,
});
const itemDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items/$itemId",
  component: ItemDetailRoute,
});
const itemArtworkGalleryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items/$itemId/artwork",
  validateSearch: validateItemArtworkGallerySearch,
  component: ItemArtworkGalleryRoute,
});
const acquisitionIntakeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/acquisition/intake",
  validateSearch: validateAcquisitionIntakeSearch,
  component: AcquisitionIntakeRoute,
});
const generatedArtifactsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/automation/generated-artifacts",
  validateSearch: validateGeneratedArtifactsSearch,
  component: GeneratedArtifactsRoute,
});
const generatedArtifactReviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/automation/generated-artifacts/$artifactId/review",
  validateSearch: validateGeneratedArtifactReviewSearch,
  component: GeneratedArtifactReviewRoute,
});
const playbackRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/playback/sessions",
  validateSearch: validatePlaybackSessionsSearch,
  component: PlaybackSessionsRoute,
});
const storageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/storage/staging",
  validateSearch: validateStorageStagingSearch,
  component: StorageStagingRoute,
});
const addonsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/addons",
  validateSearch: validateAddonsSearch,
  component: AddonsRoute,
});
const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: SettingsRoute,
});

const legacyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/legacy",
  component: LegacyRoute,
});

const mediaRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media",
  component: MediaHomeRoute,
});

const mediaConnectRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/connect",
  component: MediaConnectRoute,
});

const mediaLibrariesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/libraries",
  component: MediaLibrariesRoute,
});

const mediaSearchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/search",
  component: MediaSearchRoute,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  jobsRoute,
  accessRoute,
  overviewRoute,
  librariesRoute,
  libraryDetailRoute,
  catalogBrowseRoute,
  catalogGovernanceRoute,
  catalogGovernanceRepairRoute,
  itemDetailRoute,
  itemArtworkGalleryRoute,
  acquisitionIntakeRoute,
  generatedArtifactsRoute,
  generatedArtifactReviewRoute,
  playbackRoute,
  storageRoute,
  addonsRoute,
  settingsRoute,
  legacyRoute,
  mediaRoute,
  mediaConnectRoute,
  mediaLibrariesRoute,
  mediaSearchRoute,
]);

const adminNavItems = [
  { to: "/overview", labelId: "nav.overview", icon: Activity },
  { to: "/jobs", labelId: "nav.jobs", icon: ListChecks },
  { to: "/access", labelId: "nav.access", icon: UsersRound },
  { to: "/libraries", labelId: "nav.libraries", icon: Library },
  { to: "/catalog", labelId: "nav.catalog", icon: Film },
  { to: "/acquisition/intake", labelId: "nav.intake", icon: Inbox },
  { to: "/automation/generated-artifacts", labelId: "nav.automation", icon: Sparkles },
  { to: "/playback/sessions", labelId: "nav.playback", icon: PlayCircle },
  { to: "/storage/staging", labelId: "nav.storage", icon: HardDrive },
  { to: "/addons", labelId: "nav.addons", icon: Puzzle },
  { to: "/settings", labelId: "nav.settings", icon: Settings },
  { to: "/legacy", labelId: "nav.legacy", icon: Database },
] satisfies ReadonlyArray<Omit<AdminShellNavItem, "label"> & { labelId: MessageId }>;

function createAppRouter(context: RouterContext) {
  return createRouter({
    routeTree,
    context,
    defaultPreload: "intent",
  });
}

type AppRouter = ReturnType<typeof createAppRouter>;

declare module "@tanstack/react-router" {
  interface Register {
    router: AppRouter;
  }
}

export function App({
  dataSource,
  initialLocale,
  mediaDataSourceFactory = createMediaWebDataSource,
}: {
  dataSource: AdminDataSource;
  initialLocale?: AdminLocale;
  mediaDataSourceFactory?: MediaDataSourceFactory;
}) {
  const [queryClient] = useState(() => new QueryClient());
  const router = useMemo(
    () => createAppRouter({ dataSource, mediaDataSourceFactory }),
    [dataSource, mediaDataSourceFactory],
  );

  return (
    <QueryClientProvider client={queryClient}>
      <I18nProvider initialLocale={initialLocale}>
        <RouterProvider router={router} />
      </I18nProvider>
    </QueryClientProvider>
  );
}

function RootLayout() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const { mediaDataSourceFactory } = rootRoute.useRouteContext();
  const { locale, setLocale, t } = useI18n();
  const navItems = useMemo(
    () =>
      adminNavItems.map((item) => ({
        ...item,
        label: t(item.labelId),
      })),
    [t],
  );

  return (
    <MediaSessionProvider dataSourceFactory={mediaDataSourceFactory}>
      {pathname.startsWith("/media") ? (
        <MediaShell activePathname={pathname}>
          <Outlet />
        </MediaShell>
      ) : (
        <AdminShell
          activePathname={pathname}
          locale={locale}
          navItems={navItems}
          onLocaleChange={setLocale}
        >
          <Outlet />
        </AdminShell>
      )}
    </MediaSessionProvider>
  );
}

function JobsRoute() {
  const { dataSource } = jobsRoute.useRouteContext();
  const search = jobsRoute.useSearch();
  const navigate = jobsRoute.useNavigate();

  return (
    <JobsPage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeJobsSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function AccessRoute() {
  const { dataSource } = accessRoute.useRouteContext();
  return <AccessPage dataSource={dataSource} />;
}

function OverviewRoute() {
  const { dataSource } = overviewRoute.useRouteContext();
  return <OverviewPage dataSource={dataSource} />;
}

function LibrariesRoute() {
  const { dataSource } = librariesRoute.useRouteContext();
  return <LibrariesPage dataSource={dataSource} />;
}

function LibraryDetailRoute() {
  const { dataSource } = libraryDetailRoute.useRouteContext();
  const { libraryId } = libraryDetailRoute.useParams();
  return <LibraryDetailPage dataSource={dataSource} libraryId={libraryId} />;
}

function CatalogBrowseRoute() {
  const { dataSource } = catalogBrowseRoute.useRouteContext();
  const search = catalogBrowseRoute.useSearch();
  const navigate = catalogBrowseRoute.useNavigate();

  return (
    <CatalogBrowsePage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeCatalogSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function CatalogGovernanceRoute() {
  const { dataSource } = catalogGovernanceRoute.useRouteContext();
  const search = catalogGovernanceRoute.useSearch();
  const navigate = catalogGovernanceRoute.useNavigate();

  return (
    <CatalogGovernancePage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeCatalogGovernanceSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function CatalogGovernanceRepairRoute() {
  const { dataSource } = catalogGovernanceRepairRoute.useRouteContext();
  const { itemId } = catalogGovernanceRepairRoute.useParams();
  const search = catalogGovernanceRepairRoute.useSearch();
  const navigate = catalogGovernanceRepairRoute.useNavigate();

  return (
    <CatalogGovernanceRepairPage
      dataSource={dataSource}
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeCatalogGovernanceRepairSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function ItemDetailRoute() {
  const { dataSource } = itemDetailRoute.useRouteContext();
  const { itemId } = itemDetailRoute.useParams();

  return <ItemDetailPage dataSource={dataSource} itemId={itemId} />;
}

function ItemArtworkGalleryRoute() {
  const { dataSource } = itemArtworkGalleryRoute.useRouteContext();
  const { itemId } = itemArtworkGalleryRoute.useParams();
  const search = itemArtworkGalleryRoute.useSearch();
  const navigate = itemArtworkGalleryRoute.useNavigate();

  return (
    <ItemArtworkGalleryPage
      dataSource={dataSource}
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeItemArtworkGallerySearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function AcquisitionIntakeRoute() {
  const { dataSource } = acquisitionIntakeRoute.useRouteContext();
  const search = acquisitionIntakeRoute.useSearch();
  const navigate = acquisitionIntakeRoute.useNavigate();

  return (
    <AcquisitionIntakePage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeAcquisitionIntakeSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function GeneratedArtifactsRoute() {
  const { dataSource } = generatedArtifactsRoute.useRouteContext();
  const search = generatedArtifactsRoute.useSearch();
  const navigate = generatedArtifactsRoute.useNavigate();

  return (
    <GeneratedArtifactsPage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeGeneratedArtifactsSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function GeneratedArtifactReviewRoute() {
  const { dataSource } = generatedArtifactReviewRoute.useRouteContext();
  const { artifactId } = generatedArtifactReviewRoute.useParams();
  const search = generatedArtifactReviewRoute.useSearch();
  const navigate = generatedArtifactReviewRoute.useNavigate();

  return (
    <GeneratedArtifactReviewPage
      artifactId={artifactId}
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeGeneratedArtifactReviewSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function PlaybackSessionsRoute() {
  const { dataSource } = playbackRoute.useRouteContext();
  const search = playbackRoute.useSearch();
  const navigate = playbackRoute.useNavigate();

  return (
    <PlaybackSessionsPage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizePlaybackSessionsSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function StorageStagingRoute() {
  const { dataSource } = storageRoute.useRouteContext();
  const search = storageRoute.useSearch();
  const navigate = storageRoute.useNavigate();

  return (
    <StorageStagingPage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeStorageStagingSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function AddonsRoute() {
  const { dataSource } = addonsRoute.useRouteContext();
  const search = addonsRoute.useSearch();
  const navigate = addonsRoute.useNavigate();

  return (
    <AddonsPage
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeAddonsSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function SettingsRoute() {
  const { dataSource } = settingsRoute.useRouteContext();
  return <SettingsPage dataSource={dataSource} />;
}

function LegacyRoute() {
  const { dataSource } = legacyRoute.useRouteContext();
  return <LegacyDashboard dataSource={dataSource} />;
}

function MediaHomeRoute() {
  return <MediaHomePage />;
}

function MediaConnectRoute() {
  return <MediaConnectPage />;
}

function MediaLibrariesRoute() {
  return <MediaLibrariesPage />;
}

function MediaSearchRoute() {
  return <MediaSearchPage />;
}

function validateJobsSearch(search: Record<string, unknown>): JobsSearch {
  return normalizeJobsSearch({
    status: stringSearch(search.status),
    kind: stringSearch(search.kind),
    resource_class: stringSearch(search.resource_class),
    library_id: stringSearch(search.library_id),
    source_id: stringSearch(search.source_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeJobsSearch(search: Partial<JobsSearch>): JobsSearch {
  return {
    status: emptyToUndefined(search.status),
    kind: emptyToUndefined(search.kind),
    resource_class: emptyToUndefined(search.resource_class),
    library_id: emptyToUndefined(search.library_id),
    source_id: emptyToUndefined(search.source_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateCatalogSearch(search: Record<string, unknown>): CatalogSearch {
  return normalizeCatalogSearch({
    q: stringSearch(search.q),
    facet: stringSearch(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeCatalogSearch(search: Partial<CatalogSearch>): CatalogSearch {
  return {
    q: emptyToUndefined(search.q),
    facet: emptyToUndefined(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateCatalogGovernanceSearch(search: Record<string, unknown>): CatalogGovernanceSearch {
  return normalizeCatalogGovernanceSearch({
    library_id: stringSearch(search.library_id),
    max_confidence_milli: milliSearch(search.max_confidence_milli),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeCatalogGovernanceSearch(
  search: Partial<CatalogGovernanceSearch>,
): CatalogGovernanceSearch {
  return {
    library_id: emptyToUndefined(search.library_id),
    max_confidence_milli: milliSearch(search.max_confidence_milli),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateCatalogGovernanceRepairSearch(
  search: Record<string, unknown>,
): CatalogGovernanceRepairSearch {
  return normalizeCatalogGovernanceRepairSearch({
    mapping_id: stringSearch(search.mapping_id),
    decision: catalogGovernanceReviewDecisionSearch(search.decision),
  });
}

function normalizeCatalogGovernanceRepairSearch(
  search: Partial<CatalogGovernanceRepairSearch>,
): CatalogGovernanceRepairSearch {
  return {
    mapping_id: emptyToUndefined(search.mapping_id),
    decision: catalogGovernanceReviewDecisionSearch(search.decision),
  };
}

function validateAcquisitionIntakeSearch(search: Record<string, unknown>): AcquisitionIntakeSearch {
  return normalizeAcquisitionIntakeSearch({
    library_id: stringSearch(search.library_id),
    state: stringSearch(search.state),
    source_kind: stringSearch(search.source_kind),
    managed_import_artifact_id: stringSearch(search.managed_import_artifact_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeAcquisitionIntakeSearch(
  search: Partial<AcquisitionIntakeSearch>,
): AcquisitionIntakeSearch {
  return {
    library_id: emptyToUndefined(search.library_id),
    state: emptyToUndefined(search.state),
    source_kind: emptyToUndefined(search.source_kind),
    managed_import_artifact_id: emptyToUndefined(search.managed_import_artifact_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateGeneratedArtifactsSearch(search: Record<string, unknown>): GeneratedArtifactsSearch {
  return normalizeGeneratedArtifactsSearch({
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeGeneratedArtifactsSearch(
  search: Partial<GeneratedArtifactsSearch>,
): GeneratedArtifactsSearch {
  return {
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateGeneratedArtifactReviewSearch(
  search: Record<string, unknown>,
): GeneratedArtifactReviewSearch {
  return normalizeGeneratedArtifactReviewSearch({
    decision: reviewDecisionSearch(search.decision),
  });
}

function normalizeGeneratedArtifactReviewSearch(
  search: Partial<GeneratedArtifactReviewSearch>,
): GeneratedArtifactReviewSearch {
  return {
    decision: reviewDecisionSearch(search.decision),
  };
}

function validateItemArtworkGallerySearch(search: Record<string, unknown>): ItemArtworkGallerySearch {
  return normalizeItemArtworkGallerySearch({
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeItemArtworkGallerySearch(
  search: Partial<ItemArtworkGallerySearch>,
): ItemArtworkGallerySearch {
  return {
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validatePlaybackSessionsSearch(search: Record<string, unknown>): PlaybackSessionsSearch {
  return normalizePlaybackSessionsSearch({
    source_id: stringSearch(search.source_id),
    kind: stringSearch(search.kind),
    state: stringSearch(search.state),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizePlaybackSessionsSearch(
  search: Partial<PlaybackSessionsSearch>,
): PlaybackSessionsSearch {
  return {
    source_id: emptyToUndefined(search.source_id),
    kind: emptyToUndefined(search.kind),
    state: emptyToUndefined(search.state),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateStorageStagingSearch(search: Record<string, unknown>): StorageStagingSearch {
  return normalizeStorageStagingSearch({
    purpose: stringSearch(search.purpose),
    state: stringSearch(search.state),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeStorageStagingSearch(search: Partial<StorageStagingSearch>): StorageStagingSearch {
  return {
    purpose: emptyToUndefined(search.purpose),
    state: emptyToUndefined(search.state),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateAddonsSearch(search: Record<string, unknown>): AddonsSearch {
  return normalizeAddonsSearch({
    status: addonStatusSearch(search.status),
  });
}

function normalizeAddonsSearch(search: Partial<AddonsSearch>): AddonsSearch {
  return {
    status: addonStatusSearch(search.status),
  };
}

function stringSearch(value: unknown) {
  return typeof value === "string" ? value : undefined;
}

function addonStatusSearch(value: unknown): AddonStatus | undefined {
  if (value === "enabled" || value === "disabled" || value === "unregistered") {
    return value;
  }

  return undefined;
}

function reviewDecisionSearch(value: unknown): GeneratedArtifactReviewSearch["decision"] {
  return value === "reject" ? "reject" : "accept";
}

function catalogGovernanceReviewDecisionSearch(
  value: unknown,
): CatalogGovernanceRepairSearch["decision"] {
  return value === "reject" ? "reject" : "accept";
}

function emptyToUndefined(value: string | undefined) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function positiveIntSearch(value: unknown, fallback: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : fallback;
}

function nonNegativeIntSearch(value: unknown, fallback: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : fallback;
}

function milliSearch(value: unknown) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 && parsed <= 1000 ? parsed : undefined;
}
