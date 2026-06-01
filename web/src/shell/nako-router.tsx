"use client"

import {
  createContext,
  lazy,
  Suspense,
  useContext,
  useRef,
  type RefObject,
} from "react"
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { SurfaceSwitcher } from "@/src/shell/surface-switcher"
import type { MediaSurfaceRef, MediaSurfaceRouteView } from "@/src/features/media"
import type {
  AdminAcquisitionIntakeRouteState,
  AdminGeneratedArtifactMetadataApplyRouteState,
  AdminGeneratedArtifactReviewRouteState,
  AdminGeneratedArtifactsRouteState,
  AdminLogsRouteState,
  AdminLogsTab,
  AdminSurfaceSection,
  LogLevel,
  LogSource,
} from "@/src/features/admin"
import {
  adminManagementContextFromSearch,
  adminManagementContextRouteSearchFromSearch,
} from "@/src/features/admin/admin-management-context-state"

const MediaSurface = lazy(() =>
  import("@/src/features/media").then((module) => ({
    default: module.MediaSurface,
  })),
)

const AdminSurface = lazy(() =>
  import("@/src/features/admin").then((module) => ({
    default: module.AdminSurface,
  })),
)

const NotificationCenter = lazy(() =>
  import("@/src/features/notifications").then((module) => ({
    default: module.NotificationCenter,
  })),
)

const SettingsPage = lazy(() =>
  import("@/src/features/settings").then((module) => ({
    default: module.SettingsPage,
  })),
)

const SetupWizard = lazy(() =>
  import("@/src/features/setup").then((module) => ({
    default: module.SetupWizard,
  })),
)

const UserSelectPage = lazy(() =>
  import("@/src/features/account").then((module) => ({
    default: module.UserSelectPage,
  })),
)

const TVSurface = lazy(() =>
  import("@/src/features/tv").then((module) => ({
    default: module.TVSurface,
  })),
)

const MediaSurfaceRefContext = createContext<RefObject<MediaSurfaceRef | null> | null>(null)

function RouteFallback({ chrome = true }: { chrome?: boolean }) {
  return (
    <div className={chrome ? "grid h-[calc(100vh-3.5rem)] place-items-center" : "grid min-h-screen place-items-center"}>
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-muted border-t-primary" />
    </div>
  )
}

function RootRoute() {
  const navigate = useNavigate()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const mediaSurfaceRef = useRef<MediaSurfaceRef>(null)
  const isChromeHidden = pathname === "/setup" || pathname === "/account" || pathname === "/tv"

  if (isChromeHidden) {
    return (
      <Suspense fallback={<RouteFallback chrome={false} />}>
        <Outlet />
      </Suspense>
    )
  }

  const currentSurface = pathname.startsWith("/admin") ? "admin" : "media"

  const goToMediaSearch = () => {
    if ((pathname === "/" || pathname === "/media") && mediaSurfaceRef.current) {
      mediaSurfaceRef.current.openSearch()
      return
    }

    void navigate({ to: "/media" })
  }

  return (
    <MediaSurfaceRefContext.Provider value={mediaSurfaceRef}>
      <div className="min-h-screen bg-background">
        <SurfaceSwitcher
          currentSurface={currentSurface}
          onSurfaceChange={(surface) => {
            void navigate({ to: surface === "media" ? "/media" : "/admin" })
          }}
          onSearchClick={goToMediaSearch}
          onSettingsClick={() => {
            void navigate({ to: "/settings" })
          }}
          onSwitchUserClick={() => {
            void navigate({ to: "/account" })
          }}
          onNotificationsClick={() => {
            void navigate({ to: "/notifications" })
          }}
        />
        <Suspense fallback={<RouteFallback />}>
          <Outlet />
        </Suspense>
      </div>
    </MediaSurfaceRefContext.Provider>
  )
}

function MediaRoute() {
  const mediaSurfaceRef = useContext(MediaSurfaceRefContext)
  const navigate = useNavigate()

  return (
    <MediaSurface
      ref={mediaSurfaceRef}
      onRouteNavigate={(view) => {
        void navigate(toMediaRoute(view))
      }}
    />
  )
}

function MediaSearchRoute() {
  const mediaSurfaceRef = useContext(MediaSurfaceRefContext)
  const navigate = useNavigate()
  const search = mediaSearchRoute.useSearch()
  const query = typeof search.q === "string" ? search.q : undefined
  const initialView: MediaSurfaceRouteView = { type: "search", query }

  return (
    <MediaSurface
      ref={mediaSurfaceRef}
      initialView={initialView}
      routeKey={`search:${query ?? ""}`}
      onRouteNavigate={(view) => {
        void navigate(toMediaRoute(view))
      }}
    />
  )
}

function MediaDetailRoute() {
  const mediaSurfaceRef = useContext(MediaSurfaceRefContext)
  const navigate = useNavigate()
  const search = mediaDetailRoute.useSearch()
  const mediaId = typeof search.id === "string" && search.id.trim() ? search.id : "1"
  const mediaType = search.type === "series" ? "series" : "movie"
  const initialView: MediaSurfaceRouteView = { type: "detail", mediaId, mediaType }

  return (
    <MediaSurface
      ref={mediaSurfaceRef}
      initialView={initialView}
      routeKey={`detail:${mediaId}:${mediaType}`}
      onRouteNavigate={(view) => {
        void navigate(toMediaRoute(view))
      }}
    />
  )
}

function MediaLibraryRoute() {
  const mediaSurfaceRef = useContext(MediaSurfaceRefContext)
  const navigate = useNavigate()
  const search = mediaLibraryRoute.useSearch()
  const libraryId = typeof search.id === "string" && search.id.trim() ? search.id : "movies"
  const initialView: MediaSurfaceRouteView = {
    type: "library",
    libraryId,
    state: {
      viewMode: search.view,
      quickFilter: search.filter,
      sortBy: search.sort,
      sortOrder: search.order,
    },
  }

  return (
    <MediaSurface
      ref={mediaSurfaceRef}
      initialView={initialView}
      routeKey={`library:${libraryId}:${search.view ?? "grid"}:${search.filter ?? "all"}:${search.sort ?? "addedAt"}:${search.order ?? "desc"}`}
      onRouteNavigate={(view) => {
        void navigate(toMediaRoute(view))
      }}
    />
  )
}

function MediaMyListRoute() {
  const mediaSurfaceRef = useContext(MediaSurfaceRefContext)
  const navigate = useNavigate()
  const search = mediaMyListRoute.useSearch()
  const initialView: MediaSurfaceRouteView = {
    type: "my-list",
    playlistId: typeof search.playlist === "string" && search.playlist.trim() ? search.playlist : undefined,
    viewMode: mediaListView(search.view),
  }

  return (
    <MediaSurface
      ref={mediaSurfaceRef}
      initialView={initialView}
      routeKey={`my-list:${search.playlist ?? ""}:${search.view ?? "grid"}`}
      onRouteNavigate={(view) => {
        void navigate(toMediaRoute(view))
      }}
    />
  )
}

function AdminRoute() {
  const navigate = useNavigate()

  return (
    <AdminSurface
      activeSection="dashboard"
      onSectionNavigate={(section) => {
        void navigate(toAdminRoute(section))
      }}
    />
  )
}

function AdminLibrariesRoute() {
  const navigate = useNavigate()
  const search = adminLibrariesRoute.useSearch()
  const managementContextState = adminManagementContextFromSearch(search)

  return (
    <AdminSurface
      activeSection="libraries"
      managementContextState={managementContextState}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminUsersRoute() {
  const navigate = useNavigate()
  const search = adminUsersRoute.useSearch()
  const managementContextState = adminManagementContextFromSearch(search)

  return (
    <AdminSurface
      activeSection="users"
      managementContextState={managementContextState}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminTasksRoute() {
  const navigate = useNavigate()
  const search = adminTasksRoute.useSearch()
  const managementContextState = adminManagementContextFromSearch(search)

  return (
    <AdminSurface
      activeSection="scheduled-tasks"
      managementContextState={managementContextState}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminTranscodingRoute() {
  const navigate = useNavigate()
  const search = adminTranscodingRoute.useSearch()
  const managementContextState = adminManagementContextFromSearch(search)

  return (
    <AdminSurface
      activeSection="transcoding"
      managementContextState={managementContextState}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminSectionRoute({ section }: { section: AdminSurfaceSection }) {
  const navigate = useNavigate()

  return (
    <AdminSurface
      activeSection={section}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminLogsRoute() {
  const navigate = useNavigate()
  const search = adminLogsRoute.useSearch()

  return (
    <AdminSurface
      activeSection="activity"
      adminLogsState={adminLogsStateFromSearch(search)}
      onAdminLogsStateChange={(state) => {
        void navigate({ to: "/admin/logs", search: toAdminLogsSearch(state), replace: true })
      }}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminAcquisitionIntakeRoute() {
  const navigate = useNavigate()
  const search = adminAcquisitionIntakeRoute.useSearch()

  return (
    <AdminSurface
      activeSection="acquisition-intake"
      acquisitionIntakeState={adminAcquisitionIntakeStateFromSearch(search)}
      onAcquisitionIntakeStateChange={(state) => {
        void navigate({
          to: "/admin/acquisition/intake",
          search: toAdminAcquisitionIntakeSearch(state),
          replace: true,
        })
      }}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminGeneratedArtifactsRoute() {
  const navigate = useNavigate()
  const search = adminGeneratedArtifactsRoute.useSearch()

  return (
    <AdminSurface
      activeSection="generated-artifacts"
      generatedArtifactsState={adminGeneratedArtifactsStateFromSearch(search)}
      onGeneratedArtifactsStateChange={(state) => {
        void navigate({
          to: "/admin/automation/generated-artifacts",
          search: toAdminGeneratedArtifactsSearch(state),
          replace: true,
        })
      }}
      onGeneratedArtifactReviewRequest={(artifactId, decision) => {
        void navigate({
          to: "/admin/automation/generated-artifacts/review",
          search: {
            artifact_id: artifactId,
            decision,
          },
        })
      }}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminGeneratedArtifactReviewRoute() {
  const navigate = useNavigate()
  const search = adminGeneratedArtifactReviewRoute.useSearch()

  return (
    <AdminSurface
      activeSection="generated-artifact-review"
      generatedArtifactReviewState={adminGeneratedArtifactReviewStateFromSearch(search)}
      onGeneratedArtifactReviewStateChange={(state) => {
        void navigate({
          to: "/admin/automation/generated-artifacts/review",
          search: toAdminGeneratedArtifactReviewSearch(state),
          replace: true,
        })
      }}
      onGeneratedArtifactReviewBack={() => {
        void navigate({ to: "/admin/automation/generated-artifacts" })
      }}
      onGeneratedArtifactMetadataApplyRequest={(artifactId) => {
        void navigate({
          to: "/admin/automation/generated-artifacts/metadata-apply",
          search: {
            artifact_id: artifactId,
          },
        })
      }}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function AdminGeneratedArtifactMetadataApplyRoute() {
  const navigate = useNavigate()
  const search = adminGeneratedArtifactMetadataApplyRoute.useSearch()

  return (
    <AdminSurface
      activeSection="generated-artifact-metadata-apply"
      generatedArtifactMetadataApplyState={adminGeneratedArtifactMetadataApplyStateFromSearch(search)}
      onGeneratedArtifactMetadataApplyBack={() => {
        void navigate({ to: "/admin/automation/generated-artifacts" })
      }}
      onSectionNavigate={(nextSection) => {
        void navigate(toAdminRoute(nextSection))
      }}
    />
  )
}

function NotificationsRoute() {
  const navigate = useNavigate()

  return (
    <div className="h-[calc(100vh-3.5rem)]">
      <NotificationCenter
        onBack={() => {
          void navigate({ to: "/media" })
        }}
      />
    </div>
  )
}

function SettingsRoute() {
  const navigate = useNavigate()

  return (
    <div className="h-[calc(100vh-3.5rem)]">
      <SettingsPage
        onBack={() => {
          void navigate({ to: "/media" })
        }}
      />
    </div>
  )
}

function SetupRoute() {
  const navigate = useNavigate()

  return (
    <SetupWizard
      onComplete={() => {
        void navigate({ to: "/media" })
      }}
    />
  )
}

function AccountRoute() {
  const navigate = useNavigate()

  return (
    <UserSelectPage
      onSelectUser={() => {
        void navigate({ to: "/media" })
      }}
      onBack={() => {
        void navigate({ to: "/media" })
      }}
    />
  )
}

function TVRoute() {
  return <TVSurface />
}

const rootRoute = createRootRoute({
  component: RootRoute,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: MediaRoute,
})

const mediaRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media",
  component: MediaRoute,
})

const mediaSearchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/search",
  validateSearch: (search: Record<string, unknown>) => ({
    q: typeof search.q === "string" ? search.q : undefined,
  }),
  component: MediaSearchRoute,
})

const mediaDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/detail",
  validateSearch: (search: Record<string, unknown>) => ({
    id: typeof search.id === "string" ? search.id : undefined,
    type: search.type === "series" ? "series" : "movie",
  }),
  component: MediaDetailRoute,
})

const mediaLibraryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/library",
  validateSearch: (search: Record<string, unknown>) => ({
    id: typeof search.id === "string" ? search.id : undefined,
    view: mediaLibraryView(search.view),
    filter: typeof search.filter === "string" ? search.filter : undefined,
    sort: typeof search.sort === "string" ? search.sort : undefined,
    order: mediaLibrarySortOrder(search.order),
  }),
  component: MediaLibraryRoute,
})

const mediaMyListRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/media/my-list",
  validateSearch: (search: Record<string, unknown>) => ({
    playlist: typeof search.playlist === "string" ? search.playlist : undefined,
    view: mediaListView(search.view),
  }),
  component: MediaMyListRoute,
})

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin",
  component: AdminRoute,
})

const adminLibrariesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/libraries",
  validateSearch: validateAdminManagementSearch,
  component: AdminLibrariesRoute,
})

const adminUsersRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/users",
  validateSearch: validateAdminManagementSearch,
  component: AdminUsersRoute,
})

const adminTasksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/tasks",
  validateSearch: validateAdminManagementSearch,
  component: AdminTasksRoute,
})

const adminLogsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/logs",
  validateSearch: validateAdminLogsSearch,
  component: AdminLogsRoute,
})

const adminAcquisitionIntakeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/acquisition/intake",
  validateSearch: validateAdminAcquisitionIntakeSearch,
  component: AdminAcquisitionIntakeRoute,
})

const adminGeneratedArtifactsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/automation/generated-artifacts",
  validateSearch: validateAdminGeneratedArtifactsSearch,
  component: AdminGeneratedArtifactsRoute,
})

const adminGeneratedArtifactReviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/automation/generated-artifacts/review",
  validateSearch: validateAdminGeneratedArtifactReviewSearch,
  component: AdminGeneratedArtifactReviewRoute,
})

const adminGeneratedArtifactMetadataApplyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/automation/generated-artifacts/metadata-apply",
  validateSearch: validateAdminGeneratedArtifactMetadataApplySearch,
  component: AdminGeneratedArtifactMetadataApplyRoute,
})

const adminSettingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/settings",
  component: () => <AdminSectionRoute section="advanced" />,
})

const adminDlnaRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/dlna",
  component: () => <AdminSectionRoute section="dlna" />,
})

const adminRemoteAccessRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/remote-access",
  component: () => <AdminSectionRoute section="remote-access" />,
})

const adminTranscodingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/transcoding",
  validateSearch: validateAdminManagementSearch,
  component: AdminTranscodingRoute,
})

const adminNetworkRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/network",
  component: () => <AdminSectionRoute section="network" />,
})

const adminPluginsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/plugins",
  component: () => <AdminSectionRoute section="plugins" />,
})

const adminNotificationsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/notifications",
  component: () => <AdminSectionRoute section="notifications" />,
})

const adminBackupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/backup",
  component: () => <AdminSectionRoute section="backup" />,
})

const adminAboutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/about",
  component: () => <AdminSectionRoute section="about" />,
})

const notificationsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/notifications",
  component: NotificationsRoute,
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: SettingsRoute,
})

const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/setup",
  component: SetupRoute,
})

const accountRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/account",
  component: AccountRoute,
})

const tvRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/tv",
  component: TVRoute,
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  mediaRoute,
  mediaSearchRoute,
  mediaDetailRoute,
  mediaLibraryRoute,
  mediaMyListRoute,
  adminRoute,
  adminLibrariesRoute,
  adminUsersRoute,
  adminTasksRoute,
  adminLogsRoute,
  adminAcquisitionIntakeRoute,
  adminGeneratedArtifactsRoute,
  adminGeneratedArtifactReviewRoute,
  adminGeneratedArtifactMetadataApplyRoute,
  adminSettingsRoute,
  adminDlnaRoute,
  adminRemoteAccessRoute,
  adminTranscodingRoute,
  adminNetworkRoute,
  adminPluginsRoute,
  adminNotificationsRoute,
  adminBackupRoute,
  adminAboutRoute,
  notificationsRoute,
  settingsRoute,
  setupRoute,
  accountRoute,
  tvRoute,
])

function toMediaRoute(view: MediaSurfaceRouteView) {
  switch (view.type) {
    case "search":
      return {
        to: "/media/search",
        search: {
          q: view.query,
        },
      } as const
    case "detail":
      return {
        to: "/media/detail",
        search: {
          id: view.mediaId,
          type: view.mediaType,
        },
      } as const
    case "library":
      return {
        to: "/media/library",
        search: {
          id: view.libraryId,
          view: view.state?.viewMode === "grid" ? undefined : view.state?.viewMode,
          filter: view.state?.quickFilter === "all" ? undefined : view.state?.quickFilter,
          sort: view.state?.sortBy === "addedAt" ? undefined : view.state?.sortBy,
          order: view.state?.sortOrder === "desc" ? undefined : view.state?.sortOrder,
        },
      } as const
    case "my-list":
      return {
        to: "/media/my-list",
        search: {
          playlist: view.playlistId,
          view: view.viewMode === "grid" ? undefined : view.viewMode,
        },
      } as const
    case "browse":
      return { to: "/media" } as const
  }
}

function mediaLibraryView(value: unknown): "grid" | "detail" | "table" | undefined {
  return value === "detail" || value === "table" || value === "grid" ? value : undefined
}

function mediaLibrarySortOrder(value: unknown): "asc" | "desc" | undefined {
  return value === "asc" || value === "desc" ? value : undefined
}

function mediaListView(value: unknown): "grid" | "list" | undefined {
  return value === "list" || value === "grid" ? value : undefined
}

function toAdminRoute(section: AdminSurfaceSection) {
  switch (section) {
    case "dashboard":
      return { to: "/admin" } as const
    case "libraries":
      return { to: "/admin/libraries" } as const
    case "users":
      return { to: "/admin/users" } as const
    case "scheduled-tasks":
      return { to: "/admin/tasks" } as const
    case "activity":
      return { to: "/admin/logs" } as const
    case "acquisition-intake":
      return { to: "/admin/acquisition/intake" } as const
    case "generated-artifacts":
      return { to: "/admin/automation/generated-artifacts" } as const
    case "generated-artifact-review":
      return { to: "/admin/automation/generated-artifacts" } as const
    case "generated-artifact-metadata-apply":
      return { to: "/admin/automation/generated-artifacts" } as const
    case "advanced":
      return { to: "/admin/settings" } as const
    case "dlna":
      return { to: "/admin/dlna" } as const
    case "remote-access":
      return { to: "/admin/remote-access" } as const
    case "transcoding":
      return { to: "/admin/transcoding" } as const
    case "network":
      return { to: "/admin/network" } as const
    case "plugins":
      return { to: "/admin/plugins" } as const
    case "notifications":
      return { to: "/admin/notifications" } as const
    case "backup":
      return { to: "/admin/backup" } as const
    case "about":
      return { to: "/admin/about" } as const
  }
}

const ADMIN_LOG_LEVELS: LogLevel[] = ["error", "warn", "info", "debug"]
const ADMIN_LOG_SOURCES: LogSource[] = ["server", "auth", "database", "api", "playback", "scanner"]
const ADMIN_LOG_TABS: AdminLogsTab[] = ["all", "errors", "warnings"]
const ADMIN_LOG_TIME_RANGES = ["1h", "6h", "24h", "7d", "30d", "custom"]

interface AdminLogsRouteSearch {
  q?: string
  levels?: string
  sources?: string
  tab?: AdminLogsTab
  time?: string
}

interface AdminAcquisitionIntakeRouteSearch {
  library_id?: string
  state?: string
  source_kind?: string
  managed_import_artifact_id?: string
  limit?: number
  offset?: number
}

interface AdminGeneratedArtifactsRouteSearch {
  limit?: number
  offset?: number
}

interface AdminGeneratedArtifactReviewRouteSearch {
  artifact_id?: string
  decision?: "accept" | "reject"
}

interface AdminGeneratedArtifactMetadataApplyRouteSearch {
  artifact_id?: string
}

function validateAdminLogsSearch(search: Record<string, unknown>): AdminLogsRouteSearch {
  const levels = parseAdminLogList(search.levels, ADMIN_LOG_LEVELS)
  const sources = parseAdminLogList(search.sources, ADMIN_LOG_SOURCES)

  return {
    q: typeof search.q === "string" ? search.q : undefined,
    levels: levels ? levels.join(",") : undefined,
    sources: sources ? sources.join(",") : undefined,
    tab: parseAdminLogValue(search.tab, ADMIN_LOG_TABS),
    time: parseAdminLogValue(search.time, ADMIN_LOG_TIME_RANGES),
  }
}

function validateAdminAcquisitionIntakeSearch(
  search: Record<string, unknown>,
): AdminAcquisitionIntakeRouteSearch {
  return {
    library_id: parseSearchString(search.library_id),
    state: parseSearchString(search.state),
    source_kind: parseSearchString(search.source_kind),
    managed_import_artifact_id: parseSearchString(search.managed_import_artifact_id),
    limit: parsePositiveInteger(search.limit),
    offset: parseNonNegativeInteger(search.offset),
  }
}

function validateAdminGeneratedArtifactsSearch(
  search: Record<string, unknown>,
): AdminGeneratedArtifactsRouteSearch {
  return {
    limit: parsePositiveInteger(search.limit),
    offset: parseNonNegativeInteger(search.offset),
  }
}

function validateAdminGeneratedArtifactReviewSearch(
  search: Record<string, unknown>,
): AdminGeneratedArtifactReviewRouteSearch {
  return {
    artifact_id: parseSearchString(search.artifact_id),
    decision: search.decision === "reject" ? "reject" : "accept",
  }
}

function validateAdminGeneratedArtifactMetadataApplySearch(
  search: Record<string, unknown>,
): AdminGeneratedArtifactMetadataApplyRouteSearch {
  return {
    artifact_id: parseSearchString(search.artifact_id),
  }
}

function validateAdminManagementSearch(search: Record<string, unknown>) {
  return adminManagementContextRouteSearchFromSearch(search)
}

function parseAdminLogList<T extends string>(value: unknown, allowed: T[]): T[] | undefined {
  if (typeof value !== "string") return undefined
  if (value.trim() === "") return []

  const allowedSet = new Set(allowed)
  const parsed = value
    .split(",")
    .map((item) => item.trim())
    .filter((item): item is T => allowedSet.has(item as T))

  return parsed.length > 0 ? parsed : undefined
}

function parseSearchString(value: unknown) {
  if (typeof value !== "string") {
    return undefined
  }

  const trimmed = value.trim()
  return trimmed ? trimmed : undefined
}

function parsePositiveInteger(value: unknown) {
  const parsed = parseIntegerSearchValue(value)
  return parsed && parsed > 0 ? parsed : undefined
}

function parseNonNegativeInteger(value: unknown) {
  const parsed = parseIntegerSearchValue(value)
  return parsed !== undefined && parsed >= 0 ? parsed : undefined
}

function parseIntegerSearchValue(value: unknown) {
  if (typeof value === "number" && Number.isInteger(value)) {
    return value
  }

  if (typeof value !== "string" || !/^\d+$/.test(value.trim())) {
    return undefined
  }

  return Number.parseInt(value, 10)
}

function parseAdminLogValue<T extends string>(value: unknown, allowed: T[]): T | undefined {
  return typeof value === "string" && allowed.includes(value as T) ? (value as T) : undefined
}

function adminLogsStateFromSearch(search: AdminLogsRouteSearch): AdminLogsRouteState {
  return {
    query: search.q,
    levels: parseAdminLogList(search.levels, ADMIN_LOG_LEVELS),
    sources: parseAdminLogList(search.sources, ADMIN_LOG_SOURCES),
    tab: search.tab,
    timeRange: search.time,
  }
}

function adminAcquisitionIntakeStateFromSearch(
  search: AdminAcquisitionIntakeRouteSearch,
): AdminAcquisitionIntakeRouteState {
  return {
    libraryId: search.library_id,
    state: search.state,
    sourceKind: search.source_kind,
    managedImportArtifactId: search.managed_import_artifact_id,
    limit: search.limit,
    offset: search.offset,
  }
}

function adminGeneratedArtifactsStateFromSearch(
  search: AdminGeneratedArtifactsRouteSearch,
): AdminGeneratedArtifactsRouteState {
  return {
    limit: search.limit,
    offset: search.offset,
  }
}

function adminGeneratedArtifactReviewStateFromSearch(
  search: AdminGeneratedArtifactReviewRouteSearch,
): AdminGeneratedArtifactReviewRouteState {
  return {
    artifactId: search.artifact_id,
    decision: search.decision,
  }
}

function adminGeneratedArtifactMetadataApplyStateFromSearch(
  search: AdminGeneratedArtifactMetadataApplyRouteSearch,
): AdminGeneratedArtifactMetadataApplyRouteState {
  return {
    artifactId: search.artifact_id,
  }
}

function toAdminLogsSearch(state: AdminLogsRouteState) {
  return {
    q: state.query || undefined,
    levels: isDefaultAdminLogSet(state.levels, ADMIN_LOG_LEVELS) ? undefined : state.levels?.join(","),
    sources: isDefaultAdminLogSet(state.sources, ADMIN_LOG_SOURCES) ? undefined : state.sources?.join(","),
    tab: state.tab && state.tab !== "all" ? state.tab : undefined,
    time: state.timeRange && state.timeRange !== "24h" ? state.timeRange : undefined,
  }
}

function toAdminAcquisitionIntakeSearch(state: AdminAcquisitionIntakeRouteState) {
  return {
    library_id: state.libraryId || undefined,
    state: state.state || undefined,
    source_kind: state.sourceKind || undefined,
    managed_import_artifact_id: state.managedImportArtifactId || undefined,
    limit: state.limit && state.limit !== 50 ? state.limit : undefined,
    offset: state.offset && state.offset > 0 ? state.offset : undefined,
  }
}

function toAdminGeneratedArtifactsSearch(state: AdminGeneratedArtifactsRouteState) {
  return {
    limit: state.limit && state.limit !== 50 ? state.limit : undefined,
    offset: state.offset && state.offset > 0 ? state.offset : undefined,
  }
}

function toAdminGeneratedArtifactReviewSearch(state: AdminGeneratedArtifactReviewRouteState) {
  return {
    artifact_id: state.artifactId || undefined,
    decision: state.decision && state.decision !== "accept" ? state.decision : undefined,
  }
}

function isDefaultAdminLogSet<T extends string>(value: T[] | undefined, defaults: T[]) {
  if (!value) return true
  if (value.length !== defaults.length) return false

  const defaultSet = new Set(defaults)
  return value.every((item) => defaultSet.has(item))
}

interface NakoRouterOptions {
  history?: ReturnType<typeof createMemoryHistory>
}

export function createNakoRouter(options: NakoRouterOptions = {}) {
  if (options.history) {
    return createRouter({ routeTree, history: options.history })
  }

  return createRouter({ routeTree })
}

const router = createNakoRouter()

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}

export function NakoRouter({ router: activeRouter = router }: { router?: typeof router }) {
  return <RouterProvider router={activeRouter} />
}
