import type { CSSProperties, ReactNode } from "react";

import { useQuery } from "@tanstack/react-query";
import {
  Link,
  Outlet,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
  useRouterState,
} from "@tanstack/react-router";
import {
  Boxes,
  Clapperboard,
  FileSearch,
  HardDrive,
  LibraryBig,
  PlayCircle,
  Puzzle,
  Search,
  ServerCog,
  Settings2,
  ShieldCheck,
  SlidersHorizontal,
  Workflow,
} from "lucide-react";

import { adminApi, mediaApi } from "@/api/runtime";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  InlineActionLink,
  MetricCard,
  MetricsGrid,
  RouteLinkCard,
  SectionCard,
  SurfaceShell,
} from "@/components/shell/surface-shell";
import { capabilityGaps } from "@/lib/navigation";

const rootRoute = createRootRoute({
  component: RootLayout,
  notFoundComponent: NotFoundPage,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/media" });
  },
});

const mediaRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "media",
  component: MediaLayout,
});

const mediaIndexRoute = createRoute({
  getParentRoute: () => mediaRoute,
  path: "/",
  component: MediaHomePage,
});

const mediaLibrariesRoute = createRoute({
  getParentRoute: () => mediaRoute,
  path: "libraries",
  component: MediaLibrariesPage,
});

const mediaLibraryRoute = createRoute({
  getParentRoute: () => mediaRoute,
  path: "libraries/$libraryId",
  component: MediaLibraryDetailPage,
});

const mediaSearchRoute = createRoute({
  getParentRoute: () => mediaRoute,
  path: "search",
  component: MediaSearchPage,
});

const mediaItemRoute = createRoute({
  getParentRoute: () => mediaRoute,
  path: "items/$itemId",
  component: MediaItemDetailPage,
});

const mediaWatchRoute = createRoute({
  getParentRoute: () => mediaRoute,
  path: "watch/$itemId",
  component: MediaWatchPage,
});

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "admin",
  component: AdminLayout,
});

const adminIndexRoute = createRoute({
  getParentRoute: () => adminRoute,
  path: "/",
  component: AdminOverviewPage,
});

const adminLibrariesRoute = createRoute({
  getParentRoute: () => adminRoute,
  path: "libraries",
  component: AdminLibrariesPage,
});

const adminJobsRoute = createRoute({
  getParentRoute: () => adminRoute,
  path: "jobs",
  component: AdminJobsPage,
});

const adminAddonsRoute = createRoute({
  getParentRoute: () => adminRoute,
  path: "addons",
  component: AdminAddonsPage,
});

const adminSettingsRoute = createRoute({
  getParentRoute: () => adminRoute,
  path: "settings",
  component: AdminSettingsPage,
});

const adminItemRoute = createRoute({
  getParentRoute: () => adminRoute,
  path: "items/$itemId",
  component: AdminItemPage,
});

const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "setup",
  component: SetupPage,
});

const accountRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "account",
  component: AccountPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  mediaRoute.addChildren([
    mediaIndexRoute,
    mediaLibrariesRoute,
    mediaLibraryRoute,
    mediaSearchRoute,
    mediaItemRoute,
    mediaWatchRoute,
  ]),
  adminRoute.addChildren([
    adminIndexRoute,
    adminLibrariesRoute,
    adminJobsRoute,
    adminAddonsRoute,
    adminSettingsRoute,
    adminItemRoute,
  ]),
  setupRoute,
  accountRoute,
]);

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  defaultPreloadStaleTime: 15_000,
});

const libraryCards = [
  {
    id: "movies",
    title: "Movies",
    state: "Connect server",
    detail: "Films, editions, local artwork, and NFO-backed metadata appear here.",
  },
  {
    id: "series",
    title: "Series",
    state: "Connect server",
    detail: "Shows, seasons, episodes, watch state, and source choices appear here.",
  },
  {
    id: "anime",
    title: "Anime",
    state: "Connect server",
    detail: "Anime libraries can combine local inference, NFO, and provider mappings.",
  },
];

const mediaRoutes = [
  {
    icon: LibraryBig,
    title: "Libraries",
    to: "/media/libraries",
    description: "Open the media libraries available to this account.",
  },
  {
    icon: Search,
    title: "Search",
    to: "/media/search",
    description: "Find local media by title, people, provider mapping, or collection.",
  },
  {
    icon: PlayCircle,
    title: "Playback",
    to: "/media/watch/example-item",
    description: "Start browser playback after the server grants access for an item.",
  },
];

const adminRoutes = [
  {
    icon: LibraryBig,
    title: "Libraries",
    to: "/admin/libraries",
    description: "Review scan state, metadata authority, artwork, and file-write policy.",
  },
  {
    icon: Workflow,
    title: "Jobs",
    to: "/admin/jobs",
    description: "Inspect queues, active sessions, and playback-runtime work.",
  },
  {
    icon: Puzzle,
    title: "Addons",
    to: "/admin/addons",
    description: "Manage sidecar grants, hosted entry points, and operator trust boundaries.",
  },
];

const previewTiles = [
  {
    title: "Movie",
    meta: "Artwork",
    tone: ["oklch(42% 0.11 246)", "oklch(72% 0.14 165)"] as const,
  },
  {
    title: "Episode",
    meta: "Progress",
    tone: ["oklch(39% 0.12 285)", "oklch(75% 0.15 62)"] as const,
  },
  {
    title: "Anime",
    meta: "Mapping",
    tone: ["oklch(40% 0.14 18)", "oklch(74% 0.12 190)"] as const,
  },
];

const readinessLabels = {
  missing: "Needs design",
  partial: "Partly covered",
  planned: "Planned",
} as const;

function RootLayout() {
  return <Outlet />;
}

function MediaLayout() {
  return <Outlet />;
}

function AdminLayout() {
  return <Outlet />;
}

function MediaHomePage() {
  const continueWatching = useQuery({
    queryKey: ["media", "continue-watching"],
    queryFn: () => mediaApi.listContinueWatching(),
  });
  const libraries = useQuery({
    queryKey: ["media", "libraries"],
    queryFn: () => mediaApi.listLibraries(),
  });
  const continueItems = continueWatching.data?.value.items ?? [];
  const libraryCount = libraries.data?.value.libraries.length ?? 0;

  return (
    <SurfaceShell
      eyebrow="Nako Media"
      status={dataStatus(continueWatching.data?.source)}
      summary="Browse and watch media from a self-hosted Nako server. Admin tools stay behind role-gated routes."
      surface="media"
      title="Home"
      actions={
        <InlineActionLink
          description="Open the operator console."
          icon={ServerCog}
          label="Admin"
          to="/admin"
        />
      }
    >
      <SectionCard
        title="Continue Watching"
        summary={continueWatching.data?.error ?? "Playback history and next episodes for the current account."}
      >
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
          {continueWatching.isPending ? (
            <EmptyPanel
              icon={PlayCircle}
              title="Loading playback state"
              description="Nako is reading the current account's playback state."
            />
          ) : continueItems.length > 0 ? (
            <div className="grid gap-3 md:grid-cols-2">
              {continueItems.map((entry) => (
                <Link
                  key={entry.item.id}
                  to="/media/items/$itemId"
                  params={{ itemId: entry.item.id }}
                  className="grid gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--app-focus)]"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <p className="truncate text-sm font-semibold">{entry.item.metadata.title}</p>
                      <p className="mt-1 text-xs text-[color:var(--app-muted)]">
                        {formatProgress(entry.state.progress_percent)}
                      </p>
                    </div>
                    <PlayCircle className="h-4 w-4 text-[color:var(--app-accent)]" />
                  </div>
                  <div className="h-1.5 overflow-hidden rounded-md bg-[color:var(--app-panel-soft)]">
                    <div
                      className="h-full rounded-md bg-[color:var(--app-accent)]"
                      style={{ width: `${Math.round((entry.state.progress_percent ?? 0) * 100)}%` }}
                    />
                  </div>
                </Link>
              ))}
            </div>
          ) : (
            <EmptyPanel
              icon={PlayCircle}
              title="No playback state loaded"
              description="Connect a Nako server to show resumable titles for the current account."
              action={<InlineActionLink description="Open setup." icon={Settings2} label="Setup" to="/setup" />}
            />
          )}
          <PreviewRail />
        </div>
      </SectionCard>

      <MetricsGrid>
        <MetricCard
          label="Media Library"
          value={libraries.isPending ? "Loading" : `${libraryCount}`}
          detail="Libraries, collections, and source choices come from the Public Client API."
        />
        <MetricCard
          label="Playback"
          value={continueWatching.data?.source === "live" ? "Live" : "Ticketed"}
          detail="Stream URLs stay behind short-lived server grants."
        />
        <MetricCard
          label="Admin Links"
          value={continueWatching.data?.source === "fixture" ? "Fixture" : "Role-gated"}
          detail="Management context is only exposed to authorized principals."
        />
      </MetricsGrid>

      <SectionCard title="Browse" summary="These routes form the first media surface before live data is wired.">
        <div className="grid gap-3 md:grid-cols-3">
          {mediaRoutes.map((target) => (
            <RouteLinkCard key={target.to} {...target} />
          ))}
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaLibrariesPage() {
  const libraries = useQuery({
    queryKey: ["media", "libraries"],
    queryFn: () => mediaApi.listLibraries(),
  });
  const rows = libraries.data?.value.libraries ?? [];

  return (
    <SurfaceShell
      eyebrow="Media Library"
      status={dataStatus(libraries.data?.source)}
      summary="Libraries are shown through the media API boundary, separate from operator-only scan and write controls."
      surface="media"
      title="Libraries"
    >
      {libraries.isPending ? (
        <EmptyPanel
          icon={LibraryBig}
          title="Loading libraries"
          description="Nako is reading the media libraries visible to this account."
        />
      ) : rows.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-3">
          {rows.map((library) => (
            <Link
              key={library.id}
              to="/media/libraries/$libraryId"
              params={{ libraryId: library.id }}
              className="grid min-h-40 gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--app-focus)]"
            >
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--app-muted)]">
                    {library.options.preset}
                  </p>
                  <h2 className="mt-2 text-lg font-semibold">{library.name}</h2>
                </div>
                <Badge className="border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)]">
                  {library.options.domain}
                </Badge>
              </div>
              <p className="text-sm leading-6 text-[color:var(--app-muted)]">
                {library.options.metadata_profile.local_metadata_policy} metadata,{" "}
                {library.options.scan.realtime_monitor ? "watching enabled" : "manual scan"}.
              </p>
            </Link>
          ))}
        </div>
      ) : (
        <EmptyPanel
          icon={LibraryBig}
          title="No media libraries"
          description="Libraries appear here after they are configured on the connected server."
        />
      )}
    </SurfaceShell>
  );
}

function MediaLibraryDetailPage() {
  const { libraryId } = mediaLibraryRoute.useParams();
  const library = useQuery({
    queryKey: ["media", "library", libraryId],
    queryFn: () => mediaApi.getLibrary(libraryId),
  });
  const sources = useQuery({
    queryKey: ["media", "library", libraryId, "sources"],
    queryFn: () => mediaApi.listLibrarySources(libraryId),
  });
  const sourceRows = sources.data?.value.sources ?? [];
  const libraryRecord = library.data?.value.library;

  return (
    <SurfaceShell
      eyebrow="Library"
      status={dataStatus(sources.data?.source ?? library.data?.source)}
      summary="Library detail keeps media browsing separate from source paths, scan actions, and file-write authority."
      surface="media"
      title={libraryRecord?.name ?? libraryId}
      actions={
        <InlineActionLink
          description="Open library management."
          icon={ServerCog}
          label="Manage"
          to="/admin/libraries"
        />
      }
    >
      <SectionCard title="Library state" summary="Only public media facts belong on this route.">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Visibility" value="Account scoped" detail="Access comes from the active session." />
          <MetricCard
            label="Metadata"
            value={libraryRecord?.options.metadata_profile.local_metadata_policy ?? "Loading"}
            detail="Provider mappings remain behind Nako records."
          />
          <MetricCard
            label="Sources"
            value={sources.isPending ? "Loading" : `${sourceRows.length}`}
            detail="Local paths and source locators stay out of media views."
          />
        </div>
      </SectionCard>

      <SectionCard title="Sources" summary={sources.data?.error ?? "Playable sources and their public media facts."}>
        {sources.isPending ? (
          <EmptyPanel icon={HardDrive} title="Loading sources" description="Nako is reading library sources." />
        ) : sourceRows.length > 0 ? (
          <div className="grid gap-3">
            {sourceRows.map((entry) => (
              <Link
                key={entry.source.id}
                to="/media/items/$itemId"
                params={{ itemId: entry.source.item_id }}
                className="grid gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--app-focus)] md:grid-cols-[minmax(0,1fr)_180px]"
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold">
                    {entry.item?.metadata.title ?? entry.source.file_name}
                  </p>
                  <p className="mt-1 truncate text-sm text-[color:var(--app-muted)]">
                    {entry.source.file_name}
                  </p>
                </div>
                <div className="text-sm text-[color:var(--app-muted)] md:text-right">
                  {formatProbe(entry.probe)}
                </div>
              </Link>
            ))}
          </div>
        ) : (
          <EmptyPanel
            icon={HardDrive}
            title="No sources visible"
            description="Sources appear after a connected server returns library inventory."
          />
        )}
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaSearchPage() {
  return (
    <SurfaceShell
      eyebrow="Catalog Search"
      status="Needs server"
      summary="Search reads the public catalog surface and keeps operator diagnostics out of result records."
      surface="media"
      title="Search"
    >
      <SectionCard title="Find media">
        <div className="grid gap-4 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4">
          <label className="grid gap-2 text-sm font-semibold">
            Query
            <input
              className="min-h-11 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)] px-3 text-[color:var(--app-fg)] outline-none transition-colors placeholder:text-[color:var(--app-muted)] focus:border-[color:var(--app-accent)] focus:ring-2 focus:ring-[color:var(--app-focus)]"
              placeholder="Search local media"
            />
          </label>
          <EmptyPanel
            icon={FileSearch}
            title="No catalog connected"
            description="Results appear here after the Public Client API is connected."
          />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaItemDetailPage() {
  const { itemId } = mediaItemRoute.useParams();
  const item = useQuery({
    queryKey: ["media", "item", itemId],
    queryFn: () => mediaApi.getItem(itemId),
  });
  const detail = item.data?.value;
  const title = detail?.item.metadata.title ?? `Item ${itemId}`;

  return (
    <SurfaceShell
      eyebrow="Media Item"
      status={dataStatus(item.data?.source)}
      summary="Item detail shows media facts without exposing admin diagnostics, source locators, or local file paths."
      surface="media"
      title={title}
      actions={
        <InlineActionLink
          description="Open playback."
          icon={PlayCircle}
          label="Watch"
          to={`/media/watch/${itemId}`}
        />
      }
    >
      <SectionCard title="Item state">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard
            label="Kind"
            value={detail?.item.kind ?? "Loading"}
            detail="Canonical media kind comes from the Public Client API."
          />
          <MetricCard
            label="Sources"
            value={item.isPending ? "Loading" : `${detail?.sources.length ?? 0}`}
            detail="Source and edition selection belong to media."
          />
          <MetricCard label="Management" value="Role-gated" detail="Admin context links require explicit permission." />
        </div>
      </SectionCard>

      <SectionCard title="Source Picker" summary={item.data?.error ?? "Choose a source before requesting playback."}>
        {item.isPending ? (
          <EmptyPanel icon={HardDrive} title="Loading sources" description="Nako is reading item sources." />
        ) : detail && detail.sources.length > 0 ? (
          <div className="grid gap-3 md:grid-cols-2">
            {detail.sources.map((source) => (
              <Link
                key={source.id}
                to="/media/watch/$itemId"
                params={{ itemId: detail.item.id }}
                search={{ sourceId: source.id }}
                className="grid gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--app-focus)]"
              >
                <div className="flex items-center justify-between gap-3">
                  <p className="truncate text-sm font-semibold">{source.file_name}</p>
                  <PlayCircle className="h-4 w-4 text-[color:var(--app-accent)]" />
                </div>
                <p className="text-sm text-[color:var(--app-muted)]">
                  {formatBytes(source.size_bytes)}. Source references stay server-side.
                </p>
              </Link>
            ))}
          </div>
        ) : (
          <EmptyPanel
            icon={HardDrive}
            title="No playable sources"
            description="This item has no visible source for the current account."
          />
        )}
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaWatchPage() {
  const { itemId } = mediaWatchRoute.useParams();
  const sourceIdFromSearch = useRouterState({
    select: (state) => (state.location.search as { sourceId?: string }).sourceId,
  });
  const item = useQuery({
    queryKey: ["media", "item", itemId],
    queryFn: () => mediaApi.getItem(itemId),
  });
  const source =
    item.data?.value.sources.find((candidate) => candidate.id === sourceIdFromSearch) ??
    item.data?.value.sources[0];
  const ticket = useQuery({
    enabled: Boolean(source?.id),
    queryKey: ["media", "item", itemId, "browser-ticket", source?.id],
    queryFn: () => mediaApi.createBrowserPlaybackTicket(source!.id, { mode: "direct" }),
  });
  const title = item.data?.value.item.metadata.title ?? itemId;
  const playbackUrl = ticket.data?.value.urls[0]?.url;
  const canRenderVideo = ticket.data?.source === "live" && playbackUrl;

  return (
    <SurfaceShell
      eyebrow="Playback"
      status={dataStatus(ticket.data?.source ?? item.data?.source)}
      summary="Browser playback starts only after the server grants temporary access for the selected media item."
      surface="media"
      title={`Watch ${title}`}
    >
      <section className="grid gap-4">
        <div className="grid aspect-video place-items-center rounded-lg border border-[color:var(--app-line)] bg-[color:oklch(12%_0.022_245)]">
          {canRenderVideo ? (
            <video className="h-full w-full rounded-lg" controls src={playbackUrl} />
          ) : (
            <div className="grid gap-2 px-4 text-center">
              <Clapperboard className="mx-auto h-10 w-10 text-[color:var(--app-accent)]" />
              <p className="text-sm font-semibold">
                {ticket.isPending || item.isPending ? "Requesting playback access" : "Playback ticket ready"}
              </p>
              <p className="max-w-md text-sm leading-6 text-[color:var(--app-muted)]">
                {ticket.data?.source === "fixture"
                  ? "Fixture tickets are not attached to the video element."
                  : "Open an item from a connected library to request playback access."}
              </p>
            </div>
          )}
        </div>
        <SectionCard title="Playback Source" summary={ticket.data?.error ?? "Temporary stream access is scoped to the selected source."}>
          <div className="grid gap-3 md:grid-cols-3">
            <MetricCard label="Item" value={title} detail="Playback is requested for this media item." />
            <MetricCard
              label="Source"
              value={source?.file_name ?? "Loading"}
              detail="The source id stays in the API boundary."
            />
            <MetricCard
              label="Transport"
              value={ticket.data?.value.mode ?? "Pending"}
              detail="Browser media uses a short-lived playback ticket."
            />
          </div>
        </SectionCard>
      </section>
    </SurfaceShell>
  );
}

function AdminOverviewPage() {
  const overview = useQuery({
    queryKey: ["admin", "overview"],
    queryFn: () => adminApi.getOverview(),
  });
  const access = useQuery({
    queryKey: ["admin", "access-summary"],
    queryFn: () => adminApi.getAccessSummary(),
  });
  const overviewValue = overview.data?.value;
  const accessValue = access.data?.value;

  return (
    <SurfaceShell
      eyebrow="Operator Console"
      status={dataStatus(overview.data?.source)}
      summary="Monitor library health, runtime work, Addons, network exposure, and configuration without revealing secrets or raw paths."
      surface="admin"
      title="Overview"
    >
      <MetricsGrid>
        <MetricCard
          label="Libraries"
          value={access.isPending ? "Loading" : `${accessValue?.library_access.configured_libraries ?? 0}`}
          detail="Scan, metadata, artwork, and write policy stay in admin."
        />
        <MetricCard
          label="Runtime"
          value={overview.isPending ? "Loading" : `${overviewValue?.runtime.active_tasks ?? 0}`}
          detail="Active task count comes from the Admin overview contract."
        />
        <MetricCard
          label="Status"
          value={overviewValue?.status ?? "Loading"}
          detail={overview.data?.error ?? "Admin diagnostics stay separate from viewer pages."}
        />
      </MetricsGrid>

      <SectionCard title="Operational Areas" summary="The first release surface keeps each workflow behind a stable route.">
        <div className="grid gap-3 md:grid-cols-3">
          {adminRoutes.map((target) => (
            <RouteLinkCard key={target.to} {...target} />
          ))}
        </div>
      </SectionCard>

      <SectionCard title="Readiness Board" summary="Known server and product gaps that affect a self-hosted release.">
        <ReadinessBoard />
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminLibrariesPage() {
  const access = useQuery({
    queryKey: ["admin", "access-summary"],
    queryFn: () => adminApi.getAccessSummary(),
  });
  const libraries = access.data?.value.library_access.libraries ?? [];

  return (
    <SurfaceShell
      eyebrow="Library Management"
      status={dataStatus(access.data?.source)}
      summary="Scan controls, NFO writes, metadata authority, and destructive file operations stay under the Admin API."
      surface="admin"
      title="Libraries"
    >
      <SectionCard title="Library Operations">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard
            label="Visible Libraries"
            value={access.isPending ? "Loading" : `${libraries.length}`}
            detail="Admin API decides which libraries this principal can manage."
          />
          <MetricCard label="NFO" value="Controlled" detail="Sidecar writes are explicit Library File Writes." />
          <MetricCard label="Metadata" value="Auditable" detail="Canonical Metadata records show source and confidence." />
        </div>
      </SectionCard>

      <SectionCard title="Managed Libraries" summary={access.data?.error ?? "Library access is read from the Admin API."}>
        {access.isPending ? (
          <EmptyPanel icon={LibraryBig} title="Loading libraries" description="Nako is reading admin library access." />
        ) : libraries.length > 0 ? (
          <div className="grid gap-3">
            {libraries.map((library) => (
              <div
                key={library.library_id}
                className="grid gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 md:grid-cols-[minmax(0,1fr)_160px]"
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold">{library.library_name}</p>
                  <p className="mt-1 text-sm text-[color:var(--app-muted)]">
                    {library.preset} · {library.backend_kind}
                  </p>
                </div>
                <Badge className="border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)] md:justify-self-end">
                  {library.access}
                </Badge>
              </div>
            ))}
          </div>
        ) : (
          <EmptyPanel
            icon={LibraryBig}
            title="No managed libraries"
            description="Managed library rows appear after the Admin API reports access."
          />
        )}
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminJobsPage() {
  const jobs = useQuery({
    queryKey: ["admin", "jobs"],
    queryFn: () => adminApi.getJobs(),
  });
  const rows = jobs.data?.value.jobs ?? [];

  return (
    <SurfaceShell
      eyebrow="Runtime"
      status={dataStatus(jobs.data?.source)}
      summary="Jobs and sessions give operators a view of work in progress without leaking viewer-only media state."
      surface="admin"
      title="Jobs"
    >
      <SectionCard title="Runtime Queues">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard
            label="Jobs"
            value={jobs.isPending ? "Loading" : `${rows.length}`}
            detail="Queue state loads from the Admin API."
          />
          <MetricCard
            label="Failures"
            value={jobs.isPending ? "Loading" : `${rows.filter((job) => job.has_error).length}`}
            detail="Error detail stays redaction-safe."
          />
          <MetricCard label="Sessions" value="Split" detail="Playback sessions remain a dedicated admin route." />
        </div>
      </SectionCard>

      <SectionCard title="Job Rows" summary={jobs.data?.error ?? "Recent jobs from the Admin API."}>
        {jobs.isPending ? (
          <EmptyPanel icon={Workflow} title="Loading jobs" description="Nako is reading runtime work." />
        ) : rows.length > 0 ? (
          <div className="overflow-hidden rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)]">
            {rows.map((job) => (
              <div
                key={job.id}
                className="grid gap-2 border-b border-[color:var(--app-line)] px-4 py-3 last:border-b-0 md:grid-cols-[minmax(0,1fr)_140px_120px]"
              >
                <span className="truncate text-sm font-semibold">{job.kind}</span>
                <span className="text-sm text-[color:var(--app-muted)]">{job.status}</span>
                <span className="text-sm text-[color:var(--app-muted)]">{job.resource_class}</span>
              </div>
            ))}
          </div>
        ) : (
          <EmptyPanel icon={Workflow} title="No jobs reported" description="Runtime work appears after the server reports jobs." />
        )}
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminAddonsPage() {
  const addons = useQuery({
    queryKey: ["admin", "addons"],
    queryFn: () => adminApi.getAddons(),
  });
  const rows = addons.data?.value.addons ?? [];

  return (
    <SurfaceShell
      eyebrow="Addon Operations"
      status={dataStatus(addons.data?.source)}
      summary="Addon Sidecars stay explicit: grants, hosted pages, and lifecycle actions are reviewed by the operator."
      surface="admin"
      title="Addons"
    >
      <SectionCard title="Addon Boundary">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Protocol" value="Sidecar" detail="External code stays outside the server process." />
          <MetricCard
            label="Registrations"
            value={addons.isPending ? "Loading" : `${rows.length}`}
            detail="Addon rows come from the Admin API."
          />
          <MetricCard label="Lifecycle" value="Reviewed" detail="Install, update, and remove need explicit operator approval." />
        </div>
      </SectionCard>

      <SectionCard title="Addon Registrations" summary={addons.data?.error ?? "Registered Addon Sidecars and their safe grant summaries."}>
        {addons.isPending ? (
          <EmptyPanel icon={Puzzle} title="Loading Addons" description="Nako is reading Addon registrations." />
        ) : rows.length > 0 ? (
          <div className="grid gap-3">
            {rows.map((addon) => (
              <div
                key={addon.id}
                className="grid gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 md:grid-cols-[minmax(0,1fr)_140px]"
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold">{addon.name}</p>
                  <p className="mt-1 text-sm text-[color:var(--app-muted)]">
                    {addon.manifest_id} · {addon.granted_scopes.length} scopes
                  </p>
                </div>
                <Badge className="border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)] md:justify-self-end">
                  {addon.status}
                </Badge>
              </div>
            ))}
          </div>
        ) : (
          <EmptyPanel
            icon={Puzzle}
            title="No Addons registered"
            description="Addon registrations appear after the Admin API reports sidecars."
          />
        )}
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminSettingsPage() {
  const access = useQuery({
    queryKey: ["admin", "access-summary"],
    queryFn: () => adminApi.getAccessSummary(),
  });
  const readiness = access.data?.value.readiness;

  return (
    <SurfaceShell
      eyebrow="Configuration"
      status={dataStatus(access.data?.source)}
      summary="Settings surfaces show server-owned policy while keeping secrets, local paths, and unsafe responses redacted."
      surface="admin"
      title="Settings"
    >
      <SectionCard title="Settings Families">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard
            label="Accounts"
            value={readiness?.user_accounts ?? "Loading"}
            detail="Account readiness comes from Admin access summary."
          />
          <MetricCard
            label="Roles"
            value={readiness?.roles ?? "Loading"}
            detail="Role capability is reported by the server."
          />
          <MetricCard
            label="Library Access"
            value={readiness?.library_access_policy ?? "Loading"}
            detail="Source Locators and raw paths stay out of ordinary views."
          />
        </div>
      </SectionCard>

      <SectionCard title="Readiness" summary={access.data?.error ?? "Configuration readiness without secrets or raw credentials."}>
        {access.isPending ? (
          <EmptyPanel icon={Settings2} title="Loading readiness" description="Nako is reading configuration authority." />
        ) : readiness ? (
          <div className="grid gap-3 md:grid-cols-2">
            {Object.entries(readiness).map(([name, state]) => (
              <div
                key={name}
                className="flex items-center justify-between gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4"
              >
                <span className="text-sm font-semibold">{formatSettingName(name)}</span>
                <Badge className="border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)]">
                  {state}
                </Badge>
              </div>
            ))}
          </div>
        ) : (
          <EmptyPanel
            icon={Settings2}
            title="No readiness data"
            description="Settings readiness appears after the Admin API responds."
          />
        )}
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminItemPage() {
  const { itemId } = adminItemRoute.useParams();

  return (
    <SurfaceShell
      eyebrow="Management Context"
      status="Role-gated"
      summary="Admin item routes inspect governance facts without becoming the playback client."
      surface="admin"
      title={`Item ${itemId}`}
    >
      <SectionCard title="Item Governance">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Metadata" value="Reviewable" detail="Canonical Metadata remains server-owned." />
          <MetricCard label="Artwork" value="Managed" detail="Selected Artwork changes need confirmed operations." />
          <MetricCard label="Playback" value="Evidence" detail="Diagnostics stay redaction-safe." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function SetupPage() {
  return (
    <SurfaceShell
      eyebrow="Setup"
      status="First run"
      summary="Connect this frontend to a self-hosted Nako server, then sign in with server-backed credentials."
      surface="admin"
      title="Server Connection"
    >
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <Card>
          <CardHeader>
            <CardTitle>Connection</CardTitle>
            <CardDescription>
              The initial release should verify reachability before storing a server profile.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4">
            <label className="grid gap-2 text-sm font-semibold">
              Server URL
              <input
                className="min-h-11 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)] px-3 text-[color:var(--app-fg)] outline-none transition-colors placeholder:text-[color:var(--app-muted)] focus:border-[color:var(--app-accent)] focus:ring-2 focus:ring-[color:var(--app-focus)]"
                placeholder="http://127.0.0.1:7833"
              />
            </label>
            <EmptyPanel
              icon={ShieldCheck}
              title="Credential flow pending"
              description="The UI route is ready; session creation must come from the server authority."
            />
          </CardContent>
        </Card>
        <Card className="bg-[color:var(--app-panel-soft)]">
          <CardHeader>
            <CardTitle>Desktop Package</CardTitle>
            <CardDescription>
              Tauri uses this same route surface for connection bootstrap and account setup.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3 text-sm text-[color:var(--app-muted)]">
              <span className="inline-flex items-center gap-2">
                <HardDrive className="h-4 w-4 text-[color:var(--app-accent)]" />
                Local server profiles stay on the device.
              </span>
              <span className="inline-flex items-center gap-2">
                <SlidersHorizontal className="h-4 w-4 text-[color:var(--app-accent)]" />
                Playback policy stays server-owned.
              </span>
            </div>
          </CardContent>
        </Card>
      </div>
    </SurfaceShell>
  );
}

function AccountPage() {
  return (
    <SurfaceShell
      eyebrow="Account"
      status="No session"
      summary="Account state must come from the connected server, including roles, library access, and active sessions."
      surface="media"
      title="Account"
    >
      <SectionCard title="Current Principal">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Role" value="Not signed in" detail="Role claims load after authentication." />
          <MetricCard label="Library Access" value="Not loaded" detail="Effective access comes from the server." />
          <MetricCard label="Sessions" value="No session" detail="Switching uses backend session authority." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function NotFoundPage() {
  return (
    <SurfaceShell
      eyebrow="Route"
      status="Not found"
      summary="This address is outside the current Nako frontend routes."
      surface="media"
      title="Not Found"
      actions={
        <Link
          to="/media"
          className="inline-flex min-h-8 items-center gap-2 rounded-md border border-[color:var(--app-line)] px-3 text-xs font-semibold focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--app-focus)]"
        >
          <Boxes className="h-3.5 w-3.5" />
          Media
        </Link>
      }
    >
      <SectionCard title="Available Surfaces">
        <div className="grid gap-3 md:grid-cols-2">
          <RouteLinkCard
            description="Return to media browsing and playback."
            icon={PlayCircle}
            title="Media"
            to="/media"
          />
          <RouteLinkCard
            description="Open the operator console."
            icon={ShieldCheck}
            title="Admin"
            to="/admin"
          />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function dataStatus(source: "fixture" | "live" | undefined): string {
  if (source === "live") {
    return "Live data";
  }

  if (source === "fixture") {
    return "Fixture data";
  }

  return "Loading";
}

function formatProgress(progress: number | null): string {
  if (progress === null) {
    return "Progress not reported";
  }

  return `${Math.round(progress * 100)}% watched`;
}

function formatBytes(sizeBytes: number | null): string {
  if (sizeBytes === null) {
    return "Size not reported";
  }

  const gib = sizeBytes / 1024 / 1024 / 1024;
  if (gib >= 1) {
    return `${gib.toFixed(1)} GiB`;
  }

  const mib = sizeBytes / 1024 / 1024;
  return `${Math.round(mib)} MiB`;
}

function formatProbe(
  probe: {
    container: string | null;
    duration_ms: number | null;
    streams: Array<{
      height: number | null;
      kind: string;
      width: number | null;
    }>;
  } | null,
): string {
  if (!probe) {
    return "Probe pending";
  }

  const video = probe.streams.find((stream) => stream.kind === "video");
  const resolution = video?.width && video.height ? `${video.width}x${video.height}` : "resolution unknown";
  const minutes = probe.duration_ms ? `${Math.round(probe.duration_ms / 60_000)} min` : "duration unknown";

  return `${probe.container ?? "container unknown"} · ${resolution} · ${minutes}`;
}

function formatSettingName(name: string): string {
  return name
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function EmptyPanel({
  action,
  description,
  icon: Icon,
  title,
}: {
  action?: ReactNode;
  description: string;
  icon: typeof PlayCircle;
  title: string;
}) {
  return (
    <div className="grid min-h-40 place-items-center rounded-lg border border-dashed border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-5 text-center">
      <div className="grid max-w-md gap-3 justify-items-center">
        <Icon className="h-8 w-8 text-[color:var(--app-accent)]" />
        <div className="grid gap-1">
          <p className="text-sm font-semibold">{title}</p>
          <p className="text-sm leading-6 text-[color:var(--app-muted)]">{description}</p>
        </div>
        {action}
      </div>
    </div>
  );
}

function PreviewRail() {
  return (
    <Card className="overflow-hidden">
      <CardHeader>
        <CardTitle>Library Preview</CardTitle>
        <CardDescription>Nako-owned artwork slots before server images are loaded.</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-3">
          {previewTiles.map((tile) => (
            <PreviewTile key={tile.title} {...tile} />
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function PreviewTile({
  meta,
  title,
  tone,
}: {
  meta: string;
  title: string;
  tone: readonly [string, string];
}) {
  return (
    <figure className="grid min-w-0 gap-2">
      <div
        className="grid aspect-[2/3] place-items-end rounded-lg border border-[color:var(--app-line)] p-2"
        style={
          {
            background: `linear-gradient(145deg, ${tone[0]}, ${tone[1]})`,
          } as CSSProperties
        }
      >
        <div className="flex w-full items-center justify-between gap-2 rounded-md bg-[color:oklch(16%_0.026_245_/_82%)] px-2 py-1.5 text-[color:oklch(94%_0.014_232)]">
          <Clapperboard className="h-3.5 w-3.5 shrink-0" />
          <span className="truncate text-xs font-semibold">{title}</span>
        </div>
      </div>
      <figcaption className="truncate text-xs text-[color:var(--app-muted)]">{meta}</figcaption>
    </figure>
  );
}

function ReadinessBoard() {
  return (
    <div className="overflow-hidden rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)]">
      <div className="grid grid-cols-[minmax(0,1fr)_120px] border-b border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)] px-4 py-3 text-xs font-semibold uppercase tracking-[0.14em] text-[color:var(--app-muted)]">
        <span>Area</span>
        <span>Status</span>
      </div>
      <div className="divide-y divide-[color:var(--app-line)]">
        {capabilityGaps.map((gap) => (
          <div key={gap.area} className="grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_120px]">
            <div className="grid gap-1">
              <p className="text-sm font-semibold">{gap.area}</p>
              <p className="text-sm leading-6 text-[color:var(--app-muted)]">{gap.note}</p>
            </div>
            <div className="md:justify-self-start">
              <Badge className="border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)]">
                {readinessLabels[gap.status]}
              </Badge>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
