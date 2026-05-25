import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  Outlet,
  Link,
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
  Library,
  ListChecks,
  PlayCircle,
  Puzzle,
  Settings,
} from "lucide-react";
import { useMemo, useState } from "react";

import type { AdminDataSource } from "./adminApi/dataSource";
import { JobsPage, type JobsSearch } from "./features/jobs/JobsPage";
import { LegacyDashboard } from "./legacy/LegacyDashboard";

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
    throw redirect({ to: "/jobs", search: { limit: 20, offset: 0 } });
  },
});

const jobsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/jobs",
  validateSearch: validateJobsSearch,
  component: JobsRoute,
});

const overviewRoute = placeholderRoute("/overview", "Overview", "Server summary cards will move here after the route shell is proven.");
const librariesRoute = placeholderRoute("/libraries", "Media Libraries", "Library list and metadata-profile workflows are next after Jobs.");
const catalogRoute = placeholderRoute("/catalog/governance", "Catalog Governance", "Unknown and low-confidence Media Items remain available in the legacy dashboard for now.");
const playbackRoute = placeholderRoute("/playback/sessions", "Playback Sessions", "Session filters and support evidence will become route-owned after Jobs.");
const storageRoute = placeholderRoute("/storage/staging", "Storage Staging", "Staging diagnostics will reuse the Jobs route table pattern.");
const addonsRoute = placeholderRoute("/addons", "Addons", "Addon onboarding and operations stay in the legacy dashboard until split into routes.");
const settingsRoute = placeholderRoute("/settings", "Settings", "Settings remain read-only diagnostics until mutation semantics are designed.");

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
  playbackRoute,
  storageRoute,
  addonsRoute,
  settingsRoute,
  legacyRoute,
]);

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
  const navItems = [
    { to: "/overview", label: "Overview", icon: Activity },
    { to: "/jobs", label: "Jobs", icon: ListChecks },
    { to: "/libraries", label: "Media Libraries", icon: Library },
    { to: "/catalog/governance", label: "Catalog", icon: Film },
    { to: "/playback/sessions", label: "Playback", icon: PlayCircle },
    { to: "/storage/staging", label: "Storage", icon: HardDrive },
    { to: "/addons", label: "Addons", icon: Puzzle },
    { to: "/settings", label: "Settings", icon: Settings },
    { to: "/legacy", label: "Legacy Console", icon: Database },
  ] as const;

  return (
    <div className="adminRouteShell">
      <aside className="routeSidebar" aria-label="Primary navigation">
        <div className="routeBrand">
          <img src="/nako-app-icon-1024.png" alt="" />
          <div>
            <strong>Nako</strong>
            <span>Admin Web V2</span>
          </div>
        </div>
        <nav className="routeNav">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <Link
                className={pathname === item.to ? "routeNavItem active" : "routeNavItem"}
                key={item.to}
                to={item.to}
              >
                <Icon size={17} />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
      </aside>
      <main className="routeMain">
        <Outlet />
      </main>
    </div>
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

function LegacyRoute() {
  const { dataSource } = legacyRoute.useRouteContext();
  return <LegacyDashboard dataSource={dataSource} />;
}

function PlaceholderRoute({ title, description }: { title: string; description: string }) {
  return (
    <section className="routePage" aria-labelledby={`${title}-title`}>
      <div className="routeHeader">
        <div>
          <p className="routeKicker">Planned route</p>
          <h1 id={`${title}-title`}>{title}</h1>
          <p>{description}</p>
        </div>
      </div>
      <div className="emptyRouteState">
        This route is intentionally empty until its workflow is migrated from the
        legacy console.
      </div>
    </section>
  );
}

function placeholderRoute<const TPath extends string>(path: TPath, title: string, description: string) {
  return createRoute({
    getParentRoute: () => rootRoute,
    path,
    component: () => <PlaceholderRoute description={description} title={title} />,
  });
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

function stringSearch(value: unknown) {
  return typeof value === "string" ? value : undefined;
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
