import type { CSSProperties, ReactNode } from "react";

import {
  Link,
  Outlet,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
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
  return (
    <SurfaceShell
      eyebrow="Nako Media"
      status="Server not connected"
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
        summary="Playback history and next episodes appear after a server connection and account session."
      >
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
          <EmptyPanel
            icon={PlayCircle}
            title="No playback state loaded"
            description="Connect a Nako server to show resumable titles for the current account."
            action={<InlineActionLink description="Open setup." icon={Settings2} label="Setup" to="/setup" />}
          />
          <PreviewRail />
        </div>
      </SectionCard>

      <MetricsGrid>
        <MetricCard
          label="Media Library"
          value="Awaiting server"
          detail="Libraries, collections, and source choices come from the Public Client API."
        />
        <MetricCard
          label="Playback"
          value="Ticketed"
          detail="Stream URLs stay behind short-lived server grants."
        />
        <MetricCard
          label="Admin Links"
          value="Role-gated"
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
  return (
    <SurfaceShell
      eyebrow="Media Library"
      status="Needs server"
      summary="Libraries are shown through the media API boundary, separate from operator-only scan and write controls."
      surface="media"
      title="Libraries"
    >
      <div className="grid gap-4 md:grid-cols-3">
        {libraryCards.map((library) => (
          <Link
            key={library.id}
            to="/media/libraries/$libraryId"
            params={{ libraryId: library.id }}
            className="grid min-h-40 gap-3 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--app-focus)]"
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--app-muted)]">
                  {library.state}
                </p>
                <h2 className="mt-2 text-lg font-semibold">{library.title}</h2>
              </div>
              <Badge className="border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)]">
                Media
              </Badge>
            </div>
            <p className="text-sm leading-6 text-[color:var(--app-muted)]">{library.detail}</p>
          </Link>
        ))}
      </div>
    </SurfaceShell>
  );
}

function MediaLibraryDetailPage() {
  const { libraryId } = mediaLibraryRoute.useParams();
  const title = libraryCards.find((library) => library.id === libraryId)?.title ?? libraryId;

  return (
    <SurfaceShell
      eyebrow="Library"
      status="No source loaded"
      summary="Library detail keeps media browsing separate from source paths, scan actions, and file-write authority."
      surface="media"
      title={title}
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
          <MetricCard label="Metadata" value="Canonical" detail="Provider mappings remain behind Nako records." />
          <MetricCard label="Sources" value="Redacted" detail="Local paths and source locators stay out of media views." />
        </div>
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

  return (
    <SurfaceShell
      eyebrow="Media Item"
      status="Public view"
      summary="Item detail shows media facts without exposing admin diagnostics, source locators, or local file paths."
      surface="media"
      title={`Item ${itemId}`}
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
          <MetricCard label="Artwork" value="Server served" detail="Images should come through Nako routes." />
          <MetricCard label="Version" value="Selectable" detail="Source and edition selection belong to media." />
          <MetricCard label="Management" value="Role-gated" detail="Admin context links require explicit permission." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaWatchPage() {
  const { itemId } = mediaWatchRoute.useParams();

  return (
    <SurfaceShell
      eyebrow="Playback"
      status="Waiting for grant"
      summary="Browser playback starts only after the server grants temporary access for the selected media item."
      surface="media"
      title={`Watch ${itemId}`}
    >
      <section className="grid gap-4">
        <div className="grid aspect-video place-items-center rounded-lg border border-[color:var(--app-line)] bg-[color:oklch(12%_0.022_245)]">
          <div className="grid gap-2 px-4 text-center">
            <Clapperboard className="mx-auto h-10 w-10 text-[color:var(--app-accent)]" />
            <p className="text-sm font-semibold">No stream selected</p>
            <p className="max-w-md text-sm leading-6 text-[color:var(--app-muted)]">
              Open an item from a connected library to request playback access.
            </p>
          </div>
        </div>
      </section>
    </SurfaceShell>
  );
}

function AdminOverviewPage() {
  return (
    <SurfaceShell
      eyebrow="Operator Console"
      status="Admin surface"
      summary="Monitor library health, runtime work, Addons, network exposure, and configuration without revealing secrets or raw paths."
      surface="admin"
      title="Overview"
    >
      <MetricsGrid>
        <MetricCard label="Libraries" value="Governed" detail="Scan, metadata, artwork, and write policy stay in admin." />
        <MetricCard label="Playback" value="Observable" detail="Runtime diagnostics stay separate from viewer pages." />
        <MetricCard label="Addons" value="Sandboxed" detail="Sidecar code and hosted pages remain outside trusted admin UI." />
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
  return (
    <SurfaceShell
      eyebrow="Library Management"
      status="Admin-only"
      summary="Scan controls, NFO writes, metadata authority, and destructive file operations stay under the Admin API."
      surface="admin"
      title="Libraries"
    >
      <SectionCard title="Library Operations">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Scan" value="Confirm first" detail="Broad library operations need dry-run and review states." />
          <MetricCard label="NFO" value="Controlled" detail="Sidecar writes are explicit Library File Writes." />
          <MetricCard label="Metadata" value="Auditable" detail="Canonical Metadata records show source and confidence." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminJobsPage() {
  return (
    <SurfaceShell
      eyebrow="Runtime"
      status="Read model"
      summary="Jobs and sessions give operators a view of work in progress without leaking viewer-only media state."
      surface="admin"
      title="Jobs"
    >
      <SectionCard title="Runtime Queues">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Library scan" value="No server" detail="Queue state loads after connection." />
          <MetricCard label="Artwork ingest" value="No server" detail="Managed Artwork jobs stay server-owned." />
          <MetricCard label="Transcode" value="No server" detail="Hardware and session diagnostics stay admin-owned." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminAddonsPage() {
  return (
    <SurfaceShell
      eyebrow="Addon Operations"
      status="Separate trust"
      summary="Addon Sidecars stay explicit: grants, hosted pages, and lifecycle actions are reviewed by the operator."
      surface="admin"
      title="Addons"
    >
      <SectionCard title="Addon Boundary">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Protocol" value="Sidecar" detail="External code stays outside the server process." />
          <MetricCard label="Grants" value="Scoped" detail="Secrets and tokens are represented through safe references." />
          <MetricCard label="Lifecycle" value="Reviewed" detail="Install, update, and remove need explicit operator approval." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminSettingsPage() {
  return (
    <SurfaceShell
      eyebrow="Configuration"
      status="Authority view"
      summary="Settings surfaces show server-owned policy while keeping secrets, local paths, and unsafe responses redacted."
      surface="admin"
      title="Settings"
    >
      <SectionCard title="Settings Families">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Network" value="Exposure" detail="Remote access needs clear ownership and risk state." />
          <MetricCard label="Playback" value="Policy" detail="Device profiles and hardware selection stay backend-owned." />
          <MetricCard label="Storage" value="Redacted" detail="Source Locators and raw paths stay out of ordinary views." />
        </div>
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
