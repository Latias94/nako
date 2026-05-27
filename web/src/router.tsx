import { Link, Outlet, createRootRoute, createRoute, createRouter, redirect } from "@tanstack/react-router";
import {
  Boxes,
  Clapperboard,
  FileSearch,
  HardDrive,
  LibraryBig,
  ListChecks,
  PlayCircle,
  Puzzle,
  ScanSearch,
  ServerCog,
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
    count: "142 items",
    detail: "Local media, NFO import, provider mappings, and managed artwork.",
  },
  {
    id: "series",
    title: "Series",
    count: "37 shows",
    detail: "Season/episode browse is reserved for the first Media Web slice.",
  },
  {
    id: "anime",
    title: "Anime",
    count: "19 titles",
    detail: "Bangumi metadata and AniDB-style naming remain follow-on depth.",
  },
];

const routeTargets = [
  {
    icon: LibraryBig,
    title: "Libraries",
    to: "/media/libraries",
    description: "Browse first-party library routes instead of the old Admin prototype surface.",
  },
  {
    icon: ScanSearch,
    title: "Search",
    to: "/media/search",
    description: "Reserve a public-catalog search route for SDK-backed data later.",
  },
  {
    icon: PlayCircle,
    title: "Player",
    to: "/media/watch/example-item",
    description: "Keep the browser playback ticket seam visible without shipping mock streams.",
  },
];

const adminTargets = [
  {
    icon: LibraryBig,
    title: "Libraries",
    to: "/admin/libraries",
    description: "Operator library controls stay in the Admin API route family.",
  },
  {
    icon: Workflow,
    title: "Jobs",
    to: "/admin/jobs",
    description: "Task runtime and playback-session diagnostics remain operator-only.",
  },
  {
    icon: Puzzle,
    title: "Addons",
    to: "/admin/addons",
    description: "Addon lifecycle UI is tracked separately from media playback routes.",
  },
];

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
      eyebrow="Media Web"
      status="Product line"
      summary="The new web frontend starts from media consumption and keeps admin operations behind explicit routes."
      surface="media"
      title="Media"
      actions={
        <InlineActionLink
          description="Open the current admin surface."
          icon={ServerCog}
          label="Admin"
          to="/admin"
        />
      }
    >
      <MetricsGrid>
        <MetricCard
          label="Frontend"
          value="web/"
          detail="Vite React shell for browser release and Tauri packaging."
        />
        <MetricCard
          label="API boundary"
          value="Public"
          detail="Media routes are reserved for Public Client API and SDK data."
        />
        <MetricCard
          label="Validation"
          value="Kept"
          detail="The old Admin Web remains a contract smoke surface until parity."
        />
      </MetricsGrid>

      <SectionCard title="Route launch points" summary="Surface switching is route-owned from the first scaffold.">
        <div className="grid gap-3 md:grid-cols-3">
          {routeTargets.map((target) => (
            <RouteLinkCard key={target.to} {...target} />
          ))}
        </div>
      </SectionCard>

      <SectionCard title="Tauri target" summary="The desktop shell uses this same frontend package.">
        <div className="grid gap-3 md:grid-cols-2">
          <Card className="bg-[color:var(--app-panel-soft)]">
            <CardHeader>
              <CardTitle>WebView tier</CardTitle>
              <CardDescription>
                The initial Tauri shell packages the new frontend without pretending WebView playback is final.
              </CardDescription>
            </CardHeader>
          </Card>
          <Card className="bg-[color:var(--app-panel-soft)]">
            <CardHeader>
              <CardTitle>Native player tier</CardTitle>
              <CardDescription>
                The serious desktop target remains a Rust/native playback core split to a follow-on.
              </CardDescription>
            </CardHeader>
          </Card>
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaLibrariesPage() {
  return (
    <SurfaceShell
      eyebrow="Public client routes"
      status="Fixture scaffold"
      summary="Libraries are shown as route placeholders until the SDK-backed data source is moved into web/."
      surface="media"
      title="Libraries"
    >
      <div className="grid gap-4 md:grid-cols-3">
        {libraryCards.map((library) => (
          <Link
            key={library.id}
            to="/media/libraries/$libraryId"
            params={{ libraryId: library.id }}
            className="grid min-h-40 gap-3 rounded-xl border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)]"
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--app-muted)]">
                  {library.count}
                </p>
                <h2 className="mt-2 text-lg font-semibold">{library.title}</h2>
              </div>
              <Badge className="border-transparent bg-white/10">Public</Badge>
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

  return (
    <SurfaceShell
      eyebrow="Library detail"
      status="Context reserved"
      summary="The route owns a stable library id now; API-backed source inventory arrives in a follow-on task."
      surface="media"
      title={`Library: ${libraryId}`}
      actions={
        <InlineActionLink
          description="Open the matching admin library route when the link matrix is ready."
          icon={ServerCog}
          label="Manage"
          to="/admin/libraries"
        />
      }
    >
      <SectionCard title="Library route contract">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Stable id" value={libraryId} detail="Route state is URL-owned." />
          <MetricCard label="Sources" value="Deferred" detail="SDK source inventory moves here after WMFT-040." />
          <MetricCard label="Management" value="Gated" detail="Admin links must use role and Library Access checks." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaSearchPage() {
  return (
    <SurfaceShell
      eyebrow="Catalog search"
      status="Shell"
      summary="Search is a first-class Media route, but real results must come from the Public Client SDK."
      surface="media"
      title="Search"
    >
      <SectionCard title="Search entry">
        <label className="grid gap-2 text-sm font-semibold">
          Query
          <input
            className="min-h-11 rounded-lg border border-[color:var(--app-line)] bg-[color:var(--app-panel-soft)] px-3 text-[color:var(--app-fg)] outline-none focus:border-[color:var(--app-accent)]"
            placeholder="Search local media"
          />
        </label>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaItemDetailPage() {
  const { itemId } = mediaItemRoute.useParams();

  return (
    <SurfaceShell
      eyebrow="Media item"
      status="Public DTO only"
      summary="Item detail keeps media facts separate from Admin diagnostics and local file paths."
      surface="media"
      title={`Item: ${itemId}`}
      actions={
        <InlineActionLink
          description="Open the safe player route for this item."
          icon={PlayCircle}
          label="Watch"
          to={`/media/watch/${itemId}`}
        />
      }
    >
      <SectionCard title="Item shell">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Artwork" value="Managed" detail="Images must come from Nako routes, not bundled posters." />
          <MetricCard label="Sources" value="Picker" detail="Source/version selection belongs to the media surface." />
          <MetricCard label="Admin link" value="Gated" detail="Management links must be permission-gated." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function MediaWatchPage() {
  const { itemId } = mediaWatchRoute.useParams();

  return (
    <SurfaceShell
      eyebrow="Browser playback"
      status="Ticket seam"
      summary="The shell reserves the browser playback route while keeping real stream URLs behind short-lived tickets."
      surface="media"
      title={`Watch: ${itemId}`}
    >
      <section className="grid gap-4">
        <div className="grid aspect-video place-items-center rounded-xl border border-[color:var(--app-line)] bg-black">
          <div className="grid gap-2 text-center">
            <Clapperboard className="mx-auto h-10 w-10 text-[color:var(--app-accent)]" />
            <p className="text-sm font-semibold">Playback route ready</p>
            <p className="max-w-md text-sm text-[color:var(--app-muted)]">
              The actual video element lands when Public Client ticket transport is moved into web/.
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
      eyebrow="Operator console"
      status="Validation retained"
      summary="The new Admin route family stays operational and redaction-safe while old apps/admin-web continues as validation."
      surface="admin"
      title="Overview"
    >
      <MetricsGrid>
        <MetricCard label="Libraries" value="Route-first" detail="Admin library controls stay outside Media routes." />
        <MetricCard label="Playback" value="Diagnostics" detail="Runtime evidence remains admin-only." />
        <MetricCard label="Addons" value="Boundary" detail="Addon lifecycle breadth is split from the shell." />
      </MetricsGrid>

      <SectionCard title="Server capability gaps" summary="Tracked here so the frontend shell does not invent unsupported behavior.">
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          {capabilityGaps.map((gap) => (
            <Card key={gap.area} className="bg-[color:var(--app-panel-soft)]">
              <CardHeader>
                <div className="flex items-start justify-between gap-3">
                  <CardTitle>{gap.area}</CardTitle>
                  <Badge className="border-[color:var(--app-line)]">{gap.status}</Badge>
                </div>
                <CardDescription>{gap.note}</CardDescription>
              </CardHeader>
            </Card>
          ))}
        </div>
      </SectionCard>

      <SectionCard title="Admin launch points">
        <div className="grid gap-3 md:grid-cols-3">
          {adminTargets.map((target) => (
            <RouteLinkCard key={target.to} {...target} />
          ))}
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminLibrariesPage() {
  return (
    <SurfaceShell
      eyebrow="Library management"
      status="Admin API only"
      summary="The product frontend keeps scan, metadata, NFO, and policy controls under /admin/*."
      surface="admin"
      title="Libraries"
    >
      <SectionCard title="Library operations">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Scan" value="Planned" detail="Command buttons require real Admin API authority." />
          <MetricCard label="NFO" value="Controlled" detail="Sidecar writes remain Library File Writes." />
          <MetricCard label="Metadata" value="Profiled" detail="Profile mutation must use accepted settings authority." />
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
      summary="Jobs and sessions are visible through operator routes, not hidden inside media playback pages."
      surface="admin"
      title="Jobs"
    >
      <SectionCard title="Runtime queues">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Library scan" value="Queued" detail="Fixture row until live Admin API wiring lands." />
          <MetricCard label="Artwork ingest" value="Idle" detail="Managed Artwork jobs stay server-owned." />
          <MetricCard label="Transcode" value="Ready" detail="Playback runtime diagnostics remain admin-owned." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminAddonsPage() {
  return (
    <SurfaceShell
      eyebrow="Addon operations"
      status="Low priority"
      summary="Addon workflows remain explicit and host-owned; the release shell only reserves the route family."
      surface="admin"
      title="Addons"
    >
      <SectionCard title="Addon boundary">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Protocol" value="Sidecar" detail="External code stays outside the server process." />
          <MetricCard label="Grants" value="Scoped" detail="Secrets and tokens must stay redaction-safe." />
          <MetricCard label="Lifecycle" value="Follow-on" detail="Install/update/remove UI is not part of WMFT-020." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminSettingsPage() {
  return (
    <SurfaceShell
      eyebrow="Configuration authority"
      status="Readiness"
      summary="Settings routes should reflect backend authority instead of rendering fake save controls."
      surface="admin"
      title="Settings"
    >
      <SectionCard title="Settings families">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Network" value="Remote" detail="Tunnel and exposure UX still need release frontend routes." />
          <MetricCard label="Playback" value="Policy" detail="Device profiles and hardware selection stay backend-owned." />
          <MetricCard label="Storage" value="VFS" detail="Raw paths and Source Locators must stay redacted." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AdminItemPage() {
  const { itemId } = adminItemRoute.useParams();

  return (
    <SurfaceShell
      eyebrow="Management context"
      status="Gated"
      summary="Admin item routes can inspect governance facts without becoming the playback client."
      surface="admin"
      title={`Admin item: ${itemId}`}
    >
      <SectionCard title="Item governance">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Metadata" value="Review" detail="Canonical Metadata remains server-owned." />
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
      status="Bootstrap"
      summary="The setup route is part of the product frontend so Tauri and browser users share the same first-run shell."
      surface="admin"
      title="Setup"
    >
      <SectionCard title="Connection bootstrap">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Server" value="Local" detail="Default to self-hosted server connection." />
          <MetricCard label="Auth" value="Session" detail="Use backend credential/session authority." />
          <MetricCard label="Shell" value="Tauri-ready" detail="The same web route runs in desktop shell later." />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}

function AccountPage() {
  return (
    <SurfaceShell
      eyebrow="Account"
      status="Session shell"
      summary="Account switching must use backend sessions and roles; this route is only the product placeholder."
      surface="media"
      title="Account"
    >
      <SectionCard title="Current principal">
        <div className="grid gap-3 md:grid-cols-3">
          <MetricCard label="Role" value="Admin" detail="Fixture label until live account summary is wired." />
          <MetricCard label="Library Access" value="All" detail="Effective access must come from Public Client API." />
          <MetricCard label="Switching" value="Follow-on" detail="No frontend-only account model." />
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
      summary="The requested route is not part of the current product frontend scaffold."
      surface="media"
      title="Not found"
      actions={
        <Link
          to="/media"
          className="inline-flex items-center gap-2 rounded-full border border-[color:var(--app-line)] px-3 py-1.5 text-xs font-semibold"
        >
          <Boxes className="h-3.5 w-3.5" />
          Media home
        </Link>
      }
    >
      <SectionCard title="Available surfaces">
        <div className="grid gap-3 md:grid-cols-2">
          <RouteLinkCard
            description="Return to the media product shell."
            icon={PlayCircle}
            title="Media"
            to="/media"
          />
          <RouteLinkCard
            description="Open the admin product shell."
            icon={ShieldCheck}
            title="Admin"
            to="/admin"
          />
        </div>
      </SectionCard>
    </SurfaceShell>
  );
}
