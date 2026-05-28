"use client"

import { useState, useRef } from "react"
import { AdminSurface } from "@/components/nako/admin-surface"
import { MediaSurface, MediaSurfaceRef } from "@/components/nako/media-surface"
import { SurfaceSwitcher } from "@/components/nako/surface-switcher"
import { NotificationCenter } from "@/components/nako/notification-center"
import { SetupWizard } from "@/components/nako/setup-wizard"
import { UserSelectPage } from "@/components/nako/user-select-page"
import { SettingsPage } from "@/components/nako/settings-page"

type ViewState = 
  | { type: "main" }
  | { type: "notifications" }
  | { type: "settings" }
  | { type: "user-select" }
  | { type: "setup" }

export default function NakoHome() {
  const [surface, setSurface] = useState<"media" | "admin">("media")
  const [viewState, setViewState] = useState<ViewState>({ type: "main" })
  const mediaSurfaceRef = useRef<MediaSurfaceRef>(null)

  const handleSearchClick = () => {
    if (surface === "media" && mediaSurfaceRef.current) {
      mediaSurfaceRef.current.openSearch()
    }
  }

  const handleSettingsClick = () => {
    setViewState({ type: "settings" })
  }

  const handleSwitchUserClick = () => {
    setViewState({ type: "user-select" })
  }

  const handleNotificationsClick = () => {
    setViewState({ type: "notifications" })
  }

  const handleBackToMain = () => {
    setViewState({ type: "main" })
  }

  // 全屏页面（不显示顶部导航）
  if (viewState.type === "setup") {
    return <SetupWizard onComplete={handleBackToMain} />
  }

  if (viewState.type === "user-select") {
    return (
      <UserSelectPage 
        onSelectUser={handleBackToMain}
        onBack={handleBackToMain}
      />
    )
  }

  // 带导航的页面
  return (
    <div className="min-h-screen bg-background">
      <SurfaceSwitcher 
        currentSurface={surface} 
        onSurfaceChange={setSurface}
        onSearchClick={handleSearchClick}
        onSettingsClick={handleSettingsClick}
        onSwitchUserClick={handleSwitchUserClick}
        onNotificationsClick={handleNotificationsClick}
      />
      
      {viewState.type === "notifications" ? (
        <div className="h-[calc(100vh-3.5rem)]">
          <NotificationCenter onBack={handleBackToMain} />
        </div>
      ) : viewState.type === "settings" ? (
        <div className="h-[calc(100vh-3.5rem)]">
          <SettingsPage onBack={handleBackToMain} />
        </div>
      ) : (
        surface === "media" ? <MediaSurface ref={mediaSurfaceRef} /> : <AdminSurface />
      )}
    </div>
  )
}
