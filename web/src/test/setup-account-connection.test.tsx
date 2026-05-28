import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactElement } from "react"
import { describe, expect, it, vi } from "vitest"
import { ThemeProvider } from "@/components/theme-provider"
import { UserSelectPage } from "@/src/features/account"
import { SetupWizard } from "@/src/features/setup"
import {
  CONNECTION_PROFILE_STORAGE_KEY,
  CONNECTION_SESSION_STORAGE_KEY,
  createBrowserConnectionProfileStore,
  createLiveConnectionProfile,
  loadConnectionState,
  saveConnectionState,
} from "@/src/api/connection-profile"

function createMemoryStorage() {
  const entries = new Map<string, string>()

  return {
    getItem(key: string) {
      return entries.get(key) ?? null
    },
    setItem(key: string, value: string) {
      entries.set(key, value)
    },
    removeItem(key: string) {
      entries.delete(key)
    },
  }
}

function renderWithTheme(ui: ReactElement) {
  return render(<ThemeProvider defaultTheme="dark">{ui}</ThemeProvider>)
}

describe("setup/account connection wiring", () => {
  it("writes setup connection and admin session state without storing the password", async () => {
    const user = userEvent.setup()
    const profileStorage = createMemoryStorage()
    const sessionStorage = createMemoryStorage()
    const store = createBrowserConnectionProfileStore({ profileStorage, sessionStorage })

    renderWithTheme(
      <SetupWizard
        onComplete={vi.fn()}
        connectionStore={store}
        testConnection={vi.fn(async () => undefined)}
      />,
    )

    await user.click(screen.getByRole("button", { name: /Get Started/ }))
    await user.type(screen.getByLabelText("Server URL"), "nako.test")
    await user.click(screen.getByRole("button", { name: /Test Connection/ }))
    expect(await screen.findByText("Connected to Server")).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /Continue/ }))

    await user.click(screen.getByRole("button", { name: /Movies/ }))
    await user.type(screen.getByLabelText("Folder Path"), "/media/movies")
    await user.click(screen.getByRole("button", { name: "Add Library" }))
    await user.click(screen.getByRole("button", { name: /Continue/ }))

    await user.type(screen.getByLabelText("Username"), "admin")
    await user.type(screen.getByLabelText("Password"), "secret123")
    await user.click(screen.getByRole("button", { name: /Continue/ }))
    await user.click(screen.getByRole("button", { name: /Complete Setup/ }))

    await waitFor(() => {
      expect(loadConnectionState(store)).toMatchObject({
        profile: {
          mode: "live",
          baseUrl: "http://nako.test:8096",
        },
        session: {
          principalId: "admin",
          selectedUserId: "admin",
        },
      })
    })
    expect(profileStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)).not.toContain("secret123")
    expect(sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)).not.toContain("secret123")
  })

  it("writes selected account state into the shared session boundary", async () => {
    const user = userEvent.setup()
    const store = createBrowserConnectionProfileStore({
      profileStorage: createMemoryStorage(),
      sessionStorage: createMemoryStorage(),
    })
    saveConnectionState(
      {
        profile: createLiveConnectionProfile({ baseUrl: "http://nako.test" }),
        session: null,
      },
      store,
    )
    const onSelectUser = vi.fn()

    renderWithTheme(<UserSelectPage onSelectUser={onSelectUser} connectionStore={store} />)

    await user.click(screen.getByRole("button", { name: /Admin/ }))

    expect(onSelectUser).toHaveBeenCalledWith("1")
    expect(loadConnectionState(store)).toMatchObject({
      session: {
        principalId: "Admin",
        selectedUserId: "1",
      },
    })
  })
})
