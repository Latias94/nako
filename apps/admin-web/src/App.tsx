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
} from "lucide-react";
import { useMemo, useState } from "react";

import type { AdminDataSource } from "./adminApi/dataSource";
import { AdminShell, type AdminShellNavItem } from "./components/layout/AdminShell";
import {
  AcquisitionIntakePage,
  type AcquisitionIntakeSearch,
} from "./features/acquisition/AcquisitionIntakePage";
import { AddonsPage, type AddonsSearch } from "./features/addons/AddonsPage";
import {
  GeneratedArtifactsPage,
  type GeneratedArtifactsSearch,
} from "./features/automation/GeneratedArtifactsPage";
import {
  CatalogGovernancePage,
  type CatalogGovernanceSearch,
} from "./features/catalog/CatalogGovernancePage";
import { JobsPage, type JobsSearch } from "./features/jobs/JobsPage";
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

type RouterContext = {
  dataSource: AdminDataSource;
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
const catalogRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/catalog/governance",
  validateSearch: validateCatalogGovernanceSearch,
  component: CatalogGovernanceRoute,
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

const routeTree = rootRoute.addChildren([
  indexRoute,
  jobsRoute,
  overviewRoute,
  librariesRoute,
  catalogRoute,
  acquisitionIntakeRoute,
  generatedArtifactsRoute,
  playbackRoute,
  storageRoute,
  addonsRoute,
  settingsRoute,
  legacyRoute,
]);

const adminNavItems = [
  { to: "/overview", label: "Overview", icon: Activity },
  { to: "/jobs", label: "Jobs", icon: ListChecks },
  { to: "/libraries", label: "Media Libraries", icon: Library },
  { to: "/catalog/governance", label: "Catalog", icon: Film },
  { to: "/acquisition/intake", label: "Intake", icon: Inbox },
  { to: "/automation/generated-artifacts", label: "Automation", icon: Sparkles },
  { to: "/playback/sessions", label: "Playback", icon: PlayCircle },
  { to: "/storage/staging", label: "Storage", icon: HardDrive },
  { to: "/addons", label: "Addons", icon: Puzzle },
  { to: "/settings", label: "Settings", icon: Settings },
  { to: "/legacy", label: "Legacy Console", icon: Database },
] satisfies readonly AdminShellNavItem[];

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

export function App({ dataSource }: { dataSource: AdminDataSource }) {
  const [queryClient] = useState(() => new QueryClient());
  const router = useMemo(() => createAppRouter({ dataSource }), [dataSource]);

  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}

function RootLayout() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return (
    <AdminShell activePathname={pathname} navItems={adminNavItems}>
      <Outlet />
    </AdminShell>
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

function OverviewRoute() {
  const { dataSource } = overviewRoute.useRouteContext();
  return <OverviewPage dataSource={dataSource} />;
}

function LibrariesRoute() {
  const { dataSource } = librariesRoute.useRouteContext();
  return <LibrariesPage dataSource={dataSource} />;
}

function CatalogGovernanceRoute() {
  const { dataSource } = catalogRoute.useRouteContext();
  const search = catalogRoute.useSearch();
  const navigate = catalogRoute.useNavigate();

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
