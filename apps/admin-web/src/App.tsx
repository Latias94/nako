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
  FileJson,
  Film,
  HardDrive,
  Inbox,
  Images,
  Library,
  ListChecks,
  PlayCircle,
  Puzzle,
  Settings,
  Sparkles,
  UsersRound,
} from "lucide-react";
import { Suspense, lazy, useMemo, useState } from "react";

import type { AdminDataSource } from "./adminApi/dataSource";
import {
  AdminShell,
  type AdminShellNavItem,
} from "./components/layout/AdminShell";
import { I18nProvider, useI18n } from "./i18n/I18nProvider";
import type { AdminLocale, MessageId } from "./i18n/messages";
import type { AddonStatus } from "./adminApi/types";
import type { AcquisitionIntakeSearch } from "./features/acquisition/AcquisitionIntakePage";
import type { AddonsSearch } from "./features/addons/AddonsPage";
import type { GeneratedArtifactReviewSearch } from "./features/automation/GeneratedArtifactReviewPage";
import type { GeneratedArtifactsSearch } from "./features/automation/GeneratedArtifactsPage";
import type { ManagedArtworkMaintenanceSearch } from "./features/artwork/ManagedArtworkMaintenancePage";
import type { CatalogSearch } from "./features/catalog/CatalogBrowsePage";
import type { CatalogGovernanceSearch } from "./features/catalog/CatalogGovernancePage";
import type { CatalogGovernanceRepairSearch } from "./features/catalog/CatalogGovernanceRepairPage";
import type { EventsSearch } from "./features/events/EventsPage";
import type { ItemArtworkGallerySearch } from "./features/items/ItemArtworkGalleryPage";
import type { SourceDuplicateReconciliationSearch } from "./features/items/SourceDuplicateReconciliationPage";
import type { JobsSearch } from "./features/jobs/JobsPage";
import type { PlaybackSessionsSearch } from "./features/playback/PlaybackSessionsPage";
import type { PlaybackSupportSearch } from "./features/playback/PlaybackSupportPage";
import type { StorageStagingSearch } from "./features/storage/StorageStagingPage";
import { MediaShell } from "./surfaces/media/MediaShell";
import { MediaSessionProvider } from "./surfaces/media/MediaSession";
import type {
  MediaItemSearch,
  MediaItemsBrowseSearch,
  MediaPageSearch,
  MediaSearchRouteSearch,
} from "./surfaces/media/MediaCore";
import type {
  MediaConnection,
  MediaDataSourceFactory,
} from "./surfaces/media/mediaDataSource";

const AcquisitionIntakeRouteModule = lazy(() =>
  import("./routes/AcquisitionIntakeRouteModule").then((module) => ({
    default: module.AcquisitionIntakeRouteModule,
  })),
);
const AccessRouteModule = lazy(() =>
  import("./routes/AccessRouteModule").then((module) => ({
    default: module.AccessRouteModule,
  })),
);
const AddonsRouteModule = lazy(() =>
  import("./routes/AddonsRouteModule").then((module) => ({
    default: module.AddonsRouteModule,
  })),
);
const GeneratedArtifactsRouteModule = lazy(() =>
  import("./routes/GeneratedArtifactsRouteModule").then((module) => ({
    default: module.GeneratedArtifactsRouteModule,
  })),
);
const GeneratedArtifactReviewRouteModule = lazy(() =>
  import("./routes/GeneratedArtifactReviewRouteModule").then((module) => ({
    default: module.GeneratedArtifactReviewRouteModule,
  })),
);
const CatalogBrowseRouteModule = lazy(() =>
  import("./routes/CatalogBrowseRouteModule").then((module) => ({
    default: module.CatalogBrowseRouteModule,
  })),
);
const CatalogGovernanceRouteModule = lazy(() =>
  import("./routes/CatalogGovernanceRouteModule").then((module) => ({
    default: module.CatalogGovernanceRouteModule,
  })),
);
const CatalogGovernanceRepairRouteModule = lazy(() =>
  import("./routes/CatalogGovernanceRepairRouteModule").then((module) => ({
    default: module.CatalogGovernanceRepairRouteModule,
  })),
);
const EventsRouteModule = lazy(() =>
  import("./routes/EventsRouteModule").then((module) => ({
    default: module.EventsRouteModule,
  })),
);
const IncidentBundleRouteModule = lazy(() =>
  import("./routes/IncidentBundleRouteModule").then((module) => ({
    default: module.IncidentBundleRouteModule,
  })),
);
const JobsRouteModule = lazy(() =>
  import("./routes/JobsRouteModule").then((module) => ({
    default: module.JobsRouteModule,
  })),
);
const ManagedArtworkMaintenanceRouteModule = lazy(() =>
  import("./routes/ManagedArtworkMaintenanceRouteModule").then((module) => ({
    default: module.ManagedArtworkMaintenanceRouteModule,
  })),
);
const ItemArtworkGalleryRouteModule = lazy(() =>
  import("./routes/ItemArtworkGalleryRouteModule").then((module) => ({
    default: module.ItemArtworkGalleryRouteModule,
  })),
);
const ItemDetailRouteModule = lazy(() =>
  import("./routes/ItemDetailRouteModule").then((module) => ({
    default: module.ItemDetailRouteModule,
  })),
);
const SourceDuplicateReconciliationRouteModule = lazy(() =>
  import("./routes/SourceDuplicateReconciliationRouteModule").then(
    (module) => ({
      default: module.SourceDuplicateReconciliationRouteModule,
    }),
  ),
);
const LibraryDetailRouteModule = lazy(() =>
  import("./routes/LibraryDetailRouteModule").then((module) => ({
    default: module.LibraryDetailRouteModule,
  })),
);
const LibrariesRouteModule = lazy(() =>
  import("./routes/LibrariesRouteModule").then((module) => ({
    default: module.LibrariesRouteModule,
  })),
);
const OverviewRouteModule = lazy(() =>
  import("./routes/OverviewRouteModule").then((module) => ({
    default: module.OverviewRouteModule,
  })),
);
const PlaybackSessionsRouteModule = lazy(() =>
  import("./routes/PlaybackSessionsRouteModule").then((module) => ({
    default: module.PlaybackSessionsRouteModule,
  })),
);
const PlaybackSupportRouteModule = lazy(() =>
  import("./routes/PlaybackSupportRouteModule").then((module) => ({
    default: module.PlaybackSupportRouteModule,
  })),
);
const StorageStagingRouteModule = lazy(() =>
  import("./routes/StorageStagingRouteModule").then((module) => ({
    default: module.StorageStagingRouteModule,
  })),
);
const SettingsRouteModule = lazy(() =>
  import("./routes/SettingsRouteModule").then((module) => ({
    default: module.SettingsRouteModule,
  })),
);
const LegacyRouteModule = lazy(() =>
  import("./routes/LegacyRouteModule").then((module) => ({
    default: module.LegacyRouteModule,
  })),
);
const MediaConnectRouteModule = lazy(() =>
  import("./routes/MediaConnectRouteModule").then((module) => ({
    default: module.MediaConnectRouteModule,
  })),
);
const MediaHomeRouteModule = lazy(() =>
  import("./routes/MediaHomeRouteModule").then((module) => ({
    default: module.MediaHomeRouteModule,
  })),
);
const MediaItemsRouteModule = lazy(() =>
  import("./routes/MediaItemsRouteModule").then((module) => ({
    default: module.MediaItemsRouteModule,
  })),
);
const MediaItemDetailRouteModule = lazy(() =>
  import("./routes/MediaItemDetailRouteModule").then((module) => ({
    default: module.MediaItemDetailRouteModule,
  })),
);
const MediaLibraryDetailRouteModule = lazy(() =>
  import("./routes/MediaLibraryDetailRouteModule").then((module) => ({
    default: module.MediaLibraryDetailRouteModule,
  })),
);
const MediaLibrariesRouteModule = lazy(() =>
  import("./routes/MediaLibrariesRouteModule").then((module) => ({
    default: module.MediaLibrariesRouteModule,
  })),
);
const MediaSearchRouteModule = lazy(() =>
  import("./routes/MediaSearchRouteModule").then((module) => ({
    default: module.MediaSearchRouteModule,
  })),
);
const MediaWatchRouteModule = lazy(() =>
  import("./routes/MediaWatchRouteModule").then((module) => ({
    default: module.MediaWatchRouteModule,
  })),
);

type RouterContext = {
  dataSource: AdminDataSource;
  initialMediaConnection: MediaConnection | null;
  mediaDataSourceFactory?: MediaDataSourceFactory;
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

const eventsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/events",
  validateSearch: validateEventsSearch,
  component: EventsRoute,
});

const accessRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/access",
  component: AccessRoute,
});

const incidentBundleRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/diagnostics/incident-bundle",
  component: IncidentBundleRoute,
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
const managedArtworkMaintenanceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/artwork/maintenance",
  validateSearch: validateManagedArtworkMaintenanceSearch,
  component: ManagedArtworkMaintenanceRoute,
});
const sourceDuplicateReconciliationRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items/$itemId/sources/$sourceId/duplicates",
  validateSearch: validateSourceDuplicateReconciliationSearch,
  component: SourceDuplicateReconciliationRoute,
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
const playbackSupportRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/playback/support",
  validateSearch: validatePlaybackSupportSearch,
  component: PlaybackSupportRoute,
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
  validateSearch: validateMediaPageSearch,
  component: MediaLibrariesRoute,
});

const mediaItemsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/items",
  validateSearch: validateMediaItemsBrowseSearch,
  component: MediaItemsRoute,
});

const mediaLibraryDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/libraries/$libraryId",
  validateSearch: validateMediaLibraryItemsBrowseSearch,
  component: MediaLibraryDetailRoute,
});

const mediaSearchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/search",
  validateSearch: validateMediaSearch,
  component: MediaSearchRoute,
});

const mediaItemDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/items/$itemId",
  validateSearch: validateMediaItemSearch,
  component: MediaItemDetailRoute,
});

const mediaWatchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/watch/$itemId",
  validateSearch: validateMediaItemSearch,
  component: MediaWatchRoute,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  jobsRoute,
  eventsRoute,
  accessRoute,
  incidentBundleRoute,
  overviewRoute,
  librariesRoute,
  libraryDetailRoute,
  catalogBrowseRoute,
  catalogGovernanceRoute,
  catalogGovernanceRepairRoute,
  itemDetailRoute,
  itemArtworkGalleryRoute,
  managedArtworkMaintenanceRoute,
  sourceDuplicateReconciliationRoute,
  acquisitionIntakeRoute,
  generatedArtifactsRoute,
  generatedArtifactReviewRoute,
  playbackRoute,
  playbackSupportRoute,
  storageRoute,
  addonsRoute,
  settingsRoute,
  legacyRoute,
  mediaRoute,
  mediaConnectRoute,
  mediaLibrariesRoute,
  mediaItemsRoute,
  mediaLibraryDetailRoute,
  mediaSearchRoute,
  mediaItemDetailRoute,
  mediaWatchRoute,
]);

const adminNavItems = [
  { to: "/overview", labelId: "nav.overview", icon: Activity },
  { to: "/jobs", labelId: "nav.jobs", icon: ListChecks },
  { to: "/events", labelId: "nav.events", icon: Activity },
  { to: "/access", labelId: "nav.access", icon: UsersRound },
  {
    to: "/diagnostics/incident-bundle",
    labelId: "nav.incidentBundle",
    icon: FileJson,
  },
  { to: "/libraries", labelId: "nav.libraries", icon: Library },
  { to: "/catalog", labelId: "nav.catalog", icon: Film },
  {
    to: "/artwork/maintenance",
    labelId: "nav.artworkMaintenance",
    icon: Images,
  },
  { to: "/acquisition/intake", labelId: "nav.intake", icon: Inbox },
  {
    to: "/automation/generated-artifacts",
    labelId: "nav.automation",
    icon: Sparkles,
  },
  { to: "/playback/sessions", labelId: "nav.playback", icon: PlayCircle },
  { to: "/storage/staging", labelId: "nav.storage", icon: HardDrive },
  { to: "/addons", labelId: "nav.addons", icon: Puzzle },
  { to: "/settings", labelId: "nav.settings", icon: Settings },
  { to: "/legacy", labelId: "nav.legacy", icon: Database },
] satisfies ReadonlyArray<
  Omit<AdminShellNavItem, "label"> & { labelId: MessageId }
>;

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
  initialMediaConnection = null,
  initialLocale,
  mediaDataSourceFactory,
}: {
  dataSource: AdminDataSource;
  initialMediaConnection?: MediaConnection | null;
  initialLocale?: AdminLocale;
  mediaDataSourceFactory?: MediaDataSourceFactory;
}) {
  const [queryClient] = useState(() => new QueryClient());
  const router = useMemo(
    () =>
      createAppRouter({
        dataSource,
        initialMediaConnection,
        mediaDataSourceFactory,
      }),
    [dataSource, initialMediaConnection, mediaDataSourceFactory],
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
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  });
  const { initialMediaConnection, mediaDataSourceFactory } =
    rootRoute.useRouteContext();
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
    <MediaSessionProvider
      dataSourceFactory={mediaDataSourceFactory}
      initialConnection={initialMediaConnection}
    >
      {pathname.startsWith("/media") ? (
        <MediaShell activePathname={pathname}>
          <Suspense fallback={null}>
            <Outlet />
          </Suspense>
        </MediaShell>
      ) : (
        <AdminShell
          activePathname={pathname}
          locale={locale}
          navItems={navItems}
          onLocaleChange={setLocale}
        >
          <Suspense fallback={null}>
            <Outlet />
          </Suspense>
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
    <JobsRouteModule
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

function EventsRoute() {
  const { dataSource } = eventsRoute.useRouteContext();
  const search = eventsRoute.useSearch();
  const navigate = eventsRoute.useNavigate();

  return (
    <EventsRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeEventsSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function AccessRoute() {
  const { dataSource } = accessRoute.useRouteContext();
  return <AccessRouteModule dataSource={dataSource} />;
}

function IncidentBundleRoute() {
  const { dataSource } = incidentBundleRoute.useRouteContext();
  return <IncidentBundleRouteModule dataSource={dataSource} />;
}

function OverviewRoute() {
  const { dataSource } = overviewRoute.useRouteContext();
  return <OverviewRouteModule dataSource={dataSource} />;
}

function LibrariesRoute() {
  const { dataSource } = librariesRoute.useRouteContext();
  return <LibrariesRouteModule dataSource={dataSource} />;
}

function LibraryDetailRoute() {
  const { dataSource } = libraryDetailRoute.useRouteContext();
  const { libraryId } = libraryDetailRoute.useParams();
  return <LibraryDetailRouteModule dataSource={dataSource} libraryId={libraryId} />;
}

function CatalogBrowseRoute() {
  const { dataSource } = catalogBrowseRoute.useRouteContext();
  const search = catalogBrowseRoute.useSearch();
  const navigate = catalogBrowseRoute.useNavigate();

  return (
    <CatalogBrowseRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeCatalogSearch({ ...current, ...next }),
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
    <CatalogGovernanceRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeCatalogGovernanceSearch({ ...current, ...next }),
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
    <CatalogGovernanceRepairRouteModule
      dataSource={dataSource}
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeCatalogGovernanceRepairSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function ItemDetailRoute() {
  const { dataSource } = itemDetailRoute.useRouteContext();
  const { itemId } = itemDetailRoute.useParams();

  return <ItemDetailRouteModule dataSource={dataSource} itemId={itemId} />;
}

function ItemArtworkGalleryRoute() {
  const { dataSource } = itemArtworkGalleryRoute.useRouteContext();
  const { itemId } = itemArtworkGalleryRoute.useParams();
  const search = itemArtworkGalleryRoute.useSearch();
  const navigate = itemArtworkGalleryRoute.useNavigate();

  return (
    <ItemArtworkGalleryRouteModule
      dataSource={dataSource}
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeItemArtworkGallerySearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function ManagedArtworkMaintenanceRoute() {
  const { dataSource } = managedArtworkMaintenanceRoute.useRouteContext();
  const search = managedArtworkMaintenanceRoute.useSearch();
  const navigate = managedArtworkMaintenanceRoute.useNavigate();

  return (
    <ManagedArtworkMaintenanceRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeManagedArtworkMaintenanceSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function SourceDuplicateReconciliationRoute() {
  const { dataSource } = sourceDuplicateReconciliationRoute.useRouteContext();
  const { itemId, sourceId } = sourceDuplicateReconciliationRoute.useParams();
  const search = sourceDuplicateReconciliationRoute.useSearch();
  const navigate = sourceDuplicateReconciliationRoute.useNavigate();

  return (
    <SourceDuplicateReconciliationRouteModule
      dataSource={dataSource}
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeSourceDuplicateReconciliationSearch({
              ...current,
              ...next,
            }),
        });
      }}
      search={search}
      sourceId={sourceId}
    />
  );
}

function AcquisitionIntakeRoute() {
  const { dataSource } = acquisitionIntakeRoute.useRouteContext();
  const search = acquisitionIntakeRoute.useSearch();
  const navigate = acquisitionIntakeRoute.useNavigate();

  return (
    <AcquisitionIntakeRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeAcquisitionIntakeSearch({ ...current, ...next }),
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
    <GeneratedArtifactsRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeGeneratedArtifactsSearch({ ...current, ...next }),
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
    <GeneratedArtifactReviewRouteModule
      artifactId={artifactId}
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeGeneratedArtifactReviewSearch({ ...current, ...next }),
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
    <PlaybackSessionsRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizePlaybackSessionsSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function PlaybackSupportRoute() {
  const { dataSource } = playbackSupportRoute.useRouteContext();
  const search = playbackSupportRoute.useSearch();

  return (
    <PlaybackSupportRouteModule dataSource={dataSource} search={search} />
  );
}

function StorageStagingRoute() {
  const { dataSource } = storageRoute.useRouteContext();
  const search = storageRoute.useSearch();
  const navigate = storageRoute.useNavigate();

  return (
    <StorageStagingRouteModule
      dataSource={dataSource}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeStorageStagingSearch({ ...current, ...next }),
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
    <AddonsRouteModule
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
  return <SettingsRouteModule dataSource={dataSource} />;
}

function LegacyRoute() {
  const { dataSource } = legacyRoute.useRouteContext();
  return <LegacyRouteModule dataSource={dataSource} />;
}

function MediaHomeRoute() {
  return <MediaHomeRouteModule />;
}

function MediaConnectRoute() {
  return <MediaConnectRouteModule />;
}

function MediaLibrariesRoute() {
  const search = mediaLibrariesRoute.useSearch();
  const navigate = mediaLibrariesRoute.useNavigate();

  return (
    <MediaLibrariesRouteModule
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeMediaPageSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function MediaItemsRoute() {
  const search = mediaItemsRoute.useSearch();
  const navigate = mediaItemsRoute.useNavigate();

  return (
    <MediaItemsRouteModule
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeMediaItemsBrowseSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function MediaLibraryDetailRoute() {
  const { libraryId } = mediaLibraryDetailRoute.useParams();
  const search = mediaLibraryDetailRoute.useSearch();
  const navigate = mediaLibraryDetailRoute.useNavigate();

  return (
    <MediaLibraryDetailRouteModule
      libraryId={libraryId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeMediaLibraryItemsBrowseSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function MediaSearchRoute() {
  const search = mediaSearchRoute.useSearch();
  const navigate = mediaSearchRoute.useNavigate();

  return (
    <MediaSearchRouteModule
      onSearchChange={(next) => {
        void navigate({
          search: (current) => normalizeMediaSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function MediaItemDetailRoute() {
  const { itemId } = mediaItemDetailRoute.useParams();
  const search = mediaItemDetailRoute.useSearch();
  const navigate = mediaItemDetailRoute.useNavigate();

  return (
    <MediaItemDetailRouteModule
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeMediaItemSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function MediaWatchRoute() {
  const { itemId } = mediaWatchRoute.useParams();
  const search = mediaWatchRoute.useSearch();
  const navigate = mediaWatchRoute.useNavigate();

  return (
    <MediaWatchRouteModule
      itemId={itemId}
      onSearchChange={(next) => {
        void navigate({
          search: (current) =>
            normalizeMediaItemSearch({ ...current, ...next }),
        });
      }}
      search={search}
    />
  );
}

function validateMediaPageSearch(
  search: Record<string, unknown>,
): MediaPageSearch {
  return normalizeMediaPageSearch({
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeMediaPageSearch(
  search: Partial<MediaPageSearch>,
): MediaPageSearch {
  return {
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateMediaSearch(
  search: Record<string, unknown>,
): MediaSearchRouteSearch {
  return normalizeMediaSearch({
    facet: stringSearch(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    q: stringSearch(search.q),
  });
}

function normalizeMediaSearch(
  search: Partial<MediaSearchRouteSearch>,
): MediaSearchRouteSearch {
  return {
    facet: emptyToUndefined(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    q: emptyToUndefined(search.q),
  };
}

function validateMediaItemSearch(
  search: Record<string, unknown>,
): MediaItemSearch {
  return normalizeMediaItemSearch({
    source_id: stringSearch(search.source_id),
  });
}

function normalizeMediaItemSearch(
  search: Partial<MediaItemSearch>,
): MediaItemSearch {
  return {
    source_id: emptyToUndefined(search.source_id),
  };
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

function validateEventsSearch(search: Record<string, unknown>): EventsSearch {
  return normalizeEventsSearch({
    status: stringSearch(search.status),
    kind: stringSearch(search.kind),
    library_id: stringSearch(search.library_id),
    source_id: stringSearch(search.source_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeEventsSearch(search: Partial<EventsSearch>): EventsSearch {
  return {
    status: emptyToUndefined(search.status),
    kind: emptyToUndefined(search.kind),
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

function validateCatalogGovernanceSearch(
  search: Record<string, unknown>,
): CatalogGovernanceSearch {
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

function validateAcquisitionIntakeSearch(
  search: Record<string, unknown>,
): AcquisitionIntakeSearch {
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
    managed_import_artifact_id: emptyToUndefined(
      search.managed_import_artifact_id,
    ),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateGeneratedArtifactsSearch(
  search: Record<string, unknown>,
): GeneratedArtifactsSearch {
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

function validateItemArtworkGallerySearch(
  search: Record<string, unknown>,
): ItemArtworkGallerySearch {
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

function validateManagedArtworkMaintenanceSearch(
  search: Record<string, unknown>,
): ManagedArtworkMaintenanceSearch {
  return normalizeManagedArtworkMaintenanceSearch({
    cleanup_candidates_only: booleanSearch(search.cleanup_candidates_only),
    file_scan_limit: positiveIntSearch(search.file_scan_limit, 500),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeManagedArtworkMaintenanceSearch(
  search: Partial<ManagedArtworkMaintenanceSearch>,
): ManagedArtworkMaintenanceSearch {
  return {
    cleanup_candidates_only: booleanSearch(search.cleanup_candidates_only),
    file_scan_limit: positiveIntSearch(search.file_scan_limit, 500),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validateSourceDuplicateReconciliationSearch(
  search: Record<string, unknown>,
): SourceDuplicateReconciliationSearch {
  return normalizeSourceDuplicateReconciliationSearch({
    action: sourceDuplicateActionFilterSearch(search.action),
    freshness: sourceDuplicateFreshnessFilterSearch(search.freshness),
    library_id: stringSearch(search.library_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    status: sourceDuplicateStatusFilterSearch(search.status),
  });
}

function normalizeSourceDuplicateReconciliationSearch(
  search: Partial<SourceDuplicateReconciliationSearch>,
): SourceDuplicateReconciliationSearch {
  return {
    action: sourceDuplicateActionFilterSearch(search.action),
    freshness: sourceDuplicateFreshnessFilterSearch(search.freshness),
    library_id: emptyToUndefined(search.library_id),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    status: sourceDuplicateStatusFilterSearch(search.status),
  };
}

function validateMediaItemsBrowseSearch(
  search: Record<string, unknown>,
): MediaItemsBrowseSearch {
  return normalizeMediaItemsBrowseSearch({
    facet: stringSearch(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    order: mediaItemsOrderSearch(search.order),
    q: stringSearch(search.q),
    sort: mediaItemsSortSearch(search.sort),
    watch_state: mediaItemsWatchStateSearch(search.watch_state),
  });
}

function normalizeMediaItemsBrowseSearch(
  search: Partial<MediaItemsBrowseSearch>,
): MediaItemsBrowseSearch {
  return {
    facet: emptyToUndefined(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    order: mediaItemsOrderSearch(search.order),
    q: emptyToUndefined(search.q),
    sort: mediaItemsSortSearch(search.sort),
    watch_state: mediaItemsWatchStateSearch(search.watch_state),
  };
}

function validateMediaLibraryItemsBrowseSearch(
  search: Record<string, unknown>,
): MediaItemsBrowseSearch {
  return normalizeMediaLibraryItemsBrowseSearch({
    facet: stringSearch(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    order: mediaItemsOrderSearch(search.order),
    sort: mediaItemsSortSearch(search.sort),
    watch_state: mediaItemsWatchStateSearch(search.watch_state),
  });
}

function normalizeMediaLibraryItemsBrowseSearch(
  search: Partial<MediaItemsBrowseSearch>,
): MediaItemsBrowseSearch {
  return {
    facet: emptyToUndefined(search.facet),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
    order: mediaItemsOrderSearch(search.order),
    sort: mediaItemsSortSearch(search.sort),
    watch_state: mediaItemsWatchStateSearch(search.watch_state),
  };
}

function validatePlaybackSessionsSearch(
  search: Record<string, unknown>,
): PlaybackSessionsSearch {
  return normalizePlaybackSessionsSearch({
    source_id: stringSearch(search.source_id),
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
    state: emptyToUndefined(search.state),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  };
}

function validatePlaybackSupportSearch(
  search: Record<string, unknown>,
): PlaybackSupportSearch {
  return normalizePlaybackSupportSearch({
    session_id: stringSearch(search.session_id),
    source_id: stringSearch(search.source_id),
  });
}

function normalizePlaybackSupportSearch(
  search: Partial<PlaybackSupportSearch>,
): PlaybackSupportSearch {
  return {
    session_id: emptyToUndefined(search.session_id),
    source_id: emptyToUndefined(search.source_id),
  };
}

function validateStorageStagingSearch(
  search: Record<string, unknown>,
): StorageStagingSearch {
  return normalizeStorageStagingSearch({
    purpose: stringSearch(search.purpose),
    state: stringSearch(search.state),
    limit: positiveIntSearch(search.limit, 20),
    offset: nonNegativeIntSearch(search.offset, 0),
  });
}

function normalizeStorageStagingSearch(
  search: Partial<StorageStagingSearch>,
): StorageStagingSearch {
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

function mediaItemsSortSearch(value: unknown) {
  return value === "title" ||
    value === "release_date" ||
    value === "date_added" ||
    value === "last_played"
    ? value
    : undefined;
}

function mediaItemsOrderSearch(value: unknown) {
  return value === "asc" || value === "desc" ? value : undefined;
}

function mediaItemsWatchStateSearch(value: unknown) {
  return value === "watched" ||
    value === "unwatched" ||
    value === "in_progress"
    ? value
    : undefined;
}

function booleanSearch(value: unknown) {
  if (value === true || value === "true" || value === "1") {
    return true;
  }

  if (value === false || value === "false" || value === "0") {
    return false;
  }

  return false;
}

function addonStatusSearch(value: unknown): AddonStatus | undefined {
  if (value === "enabled" || value === "disabled" || value === "unregistered") {
    return value;
  }

  return undefined;
}

function sourceDuplicateStatusFilterSearch(
  value: unknown,
): SourceDuplicateReconciliationSearch["status"] {
  if (
    value === "none" ||
    value === "suggested" ||
    value === "confirmed" ||
    value === "rejected"
  ) {
    return value;
  }

  return undefined;
}

function sourceDuplicateActionFilterSearch(
  value: unknown,
): SourceDuplicateReconciliationSearch["action"] {
  if (
    value === "suggest_relationship" ||
    value === "preserve_suggested" ||
    value === "preserve_confirmed" ||
    value === "preserve_rejected" ||
    value === "refresh_source_fingerprint"
  ) {
    return value;
  }

  return undefined;
}

function sourceDuplicateFreshnessFilterSearch(
  value: unknown,
): SourceDuplicateReconciliationSearch["freshness"] {
  if (value === "current" || value === "stale") {
    return value;
  }

  return undefined;
}

function reviewDecisionSearch(
  value: unknown,
): GeneratedArtifactReviewSearch["decision"] {
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
  return Number.isInteger(parsed) && parsed >= 0 && parsed <= 1000
    ? parsed
    : undefined;
}
