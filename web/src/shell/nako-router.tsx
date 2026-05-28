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
import type { AdminSurfaceSection } from "@/src/features/admin"

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
  const initialView: MediaSurfaceRouteView = { type: "library", libraryId }

  return (
    <MediaSurface
      ref={mediaSurfaceRef}
      initialView={initialView}
      routeKey={`library:${libraryId}`}
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
  }),
  component: MediaLibraryRoute,
})

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin",
  component: AdminRoute,
})

const adminLibrariesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/libraries",
  component: () => <AdminSectionRoute section="libraries" />,
})

const adminUsersRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/users",
  component: () => <AdminSectionRoute section="users" />,
})

const adminTasksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/tasks",
  component: () => <AdminSectionRoute section="scheduled-tasks" />,
})

const adminLogsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/logs",
  component: () => <AdminSectionRoute section="activity" />,
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
  component: () => <AdminSectionRoute section="transcoding" />,
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
  adminRoute,
  adminLibrariesRoute,
  adminUsersRoute,
  adminTasksRoute,
  adminLogsRoute,
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
        },
      } as const
    case "browse":
      return { to: "/media" } as const
  }
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
