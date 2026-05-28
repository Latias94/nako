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
import type { MediaSurfaceRef } from "@/src/features/media"

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

  return <MediaSurface ref={mediaSurfaceRef} />
}

function AdminRoute() {
  return <AdminSurface />
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

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin",
  component: AdminRoute,
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
  adminRoute,
  notificationsRoute,
  settingsRoute,
  setupRoute,
  accountRoute,
  tvRoute,
])

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
