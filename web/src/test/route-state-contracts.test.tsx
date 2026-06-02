import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"
import { CONNECTION_PROFILE_STORAGE_KEY, CONNECTION_SESSION_STORAGE_KEY } from "@/src/api/connection-profile"

function renderRoute(path: string) {
  const router = createNakoRouter({
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  const result = render(
    <ThemeProvider defaultTheme="dark">
      <QueryProvider>
        <NakoRouter router={router} />
      </QueryProvider>
    </ThemeProvider>,
  )

  return { router, ...result }
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe("route state contracts", () => {
  it("writes Admin log search state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/logs")

    const input = await screen.findByPlaceholderText("搜索日志内容...", {}, { timeout: 10000 })
    await user.type(input, "database")

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ q: "database" })
    })
  })

  it("renders live Admin acquisition intake candidates without exposing raw sensitive fields", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{ path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/acquisition/intake/candidates") {
        return jsonResponse(adminAcquisitionIntakeCandidatesResponse())
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute(
        "/admin/acquisition/intake?library_id=library-a&state=ready&source_kind=watch_folder&managed_import_artifact_id=artifact-live&limit=25&offset=50",
      )

      expect(await screen.findByText("candidate-live", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("file://<redacted>/Live.mkv")).toBeInTheDocument()
      expect(screen.queryByText("/mnt/private/raw/Live.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()
      expect(calls).toContainEqual({
        path: [
          "/admin/v1/acquisition/intake/candidates",
          "?library_id=library-a&state=ready&source_kind=watch_folder",
          "&managed_import_artifact_id=artifact-live&limit=25&offset=50",
        ].join(""),
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("writes Admin acquisition intake filter state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/acquisition/intake")

    await user.type(await screen.findByLabelText("媒体库 ID", {}, { timeout: 10000 }), "library-anime")
    await user.click(screen.getByRole("button", { name: "应用筛选" }))

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/admin/acquisition/intake")
      expect(router.state.location.search).toMatchObject({ library_id: "library-anime" })
    })
  })

  it("renders live Admin generated artifact proposals without exposing raw sensitive fields", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{ path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifacts/proposals") {
        return jsonResponse(adminGeneratedArtifactProposalsResponse())
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts?limit=25&offset=50")

      expect(await screen.findByText("artifact-live", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("Live Automation Provider")).toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe generated payload title")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("F:\\private\\source\\Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("file:///mnt/private/source/Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()
      expect(calls).toContainEqual({
        path: "/admin/v1/automation/generated-artifacts/proposals?limit=25&offset=50",
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("renders live Admin generated artifact recovery queue without exposing raw sensitive fields", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{ path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifact-apply-recovery") {
        return jsonResponse(adminGeneratedArtifactMetadataApplyRecoveryResponse())
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts/recovery?attention=needs_repair&limit=25&offset=50")

      expect(await screen.findByRole("heading", { name: "生成产物恢复" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText(/outcome-live/, {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("target_stale")).toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe generated payload title")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("F:\\private\\source\\Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("file:///mnt/private/source/Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe-recovery-idempotency")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()
      expect(calls).toContainEqual({
        path: "/admin/v1/automation/generated-artifact-apply-recovery?attention=needs_repair&limit=25&offset=50",
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("prepares recovery repairs through the current Metadata Authority apply plan", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method,
        path: `${url.pathname}${url.search}`,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "GET /admin/v1/automation/generated-artifact-apply-recovery":
          return jsonResponse(adminGeneratedArtifactMetadataApplyRecoveryResponse())
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/metadata-apply-plan":
          return jsonResponse(adminGeneratedArtifactMetadataApplyPlanResponse("artifact-live"))
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/metadata-apply":
          return jsonResponse(adminGeneratedArtifactMetadataApplyResponse("artifact-live"))
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      const { router } = renderRoute("/admin/automation/generated-artifacts/recovery?attention=needs_repair")

      expect(await screen.findByRole("heading", { name: "生成产物恢复" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText(/outcome-live/, {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.queryByText("unsafe-recovery-idempotency")).not.toBeInTheDocument()

      await user.click(screen.getByRole("button", { name: "应用计划" }))

      await waitFor(() => {
        expect(router.state.location.pathname).toBe("/admin/automation/generated-artifacts/metadata-apply")
        expect(router.state.location.search).toMatchObject({ artifact_id: "artifact-live" })
      })
      expect(await screen.findByRole("heading", { name: "Metadata Authority apply" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("title", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(calls).toContainEqual({
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact-live/metadata-apply-plan",
        body: undefined,
        authorization: "Bearer admin-token",
      })
      expect(
        calls.some(
          (call) =>
            call.method === "POST" &&
            call.path === "/admin/v1/automation/generated-artifacts/artifact-live/metadata-apply",
        ),
      ).toBe(false)

      await user.click(screen.getByRole("button", { name: "准备应用" }))
      await user.click(screen.getByRole("button", { name: "确认应用" }))

      const applyCall = calls.find(
        (call) =>
          call.method === "POST" &&
          call.path === "/admin/v1/automation/generated-artifacts/artifact-live/metadata-apply",
      )
      expect(applyCall).toBeDefined()
      expect(applyCall?.body).toMatchObject({
        idempotency_key: expect.stringMatching(/^web-generated-artifact-metadata-apply:artifact-live:/),
      })
      expect(JSON.stringify(applyCall?.body)).not.toContain("unsafe-recovery-idempotency")
      expect(applyCall?.authorization).toBe("Bearer admin-token")
      expect(await screen.findByText("元数据应用结果已存在，已返回幂等结果", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.queryByText("unsafe-recovery-idempotency")).not.toBeInTheDocument()
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("writes Admin generated artifact pagination state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/automation/generated-artifacts?limit=1")

    expect(await screen.findByText("fixture-generated-artifact-1", {}, { timeout: 10000 })).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /下一页/ }))

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/admin/automation/generated-artifacts")
      expect(router.state.location.search).toMatchObject({ limit: 1, offset: 1 })
    })
  })

  it("writes Admin generated artifact recovery pagination state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/automation/generated-artifacts/recovery?limit=1")

    expect(await screen.findByText(/fixture-generated-outcome-1/, {}, { timeout: 10000 })).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /下一页/ }))

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/admin/automation/generated-artifacts/recovery")
      expect(router.state.location.search).toMatchObject({ limit: 1, offset: 1 })
    })
  })

  it("writes Admin generated artifact review requests to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/automation/generated-artifacts")

    expect(await screen.findByText("fixture-generated-artifact-1", {}, { timeout: 10000 })).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /查看接受计划 fixture-generated-artifact-1/ }))

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/admin/automation/generated-artifacts/review")
      expect(router.state.location.search).toMatchObject({
        artifact_id: "fixture-generated-artifact-1",
        decision: "accept",
      })
    })
  })

  it("keeps fixture Admin generated artifact review mutations disabled", async () => {
    renderRoute(
      "/admin/automation/generated-artifacts/review?artifact_id=fixture-generated-artifact-1&decision=accept",
    )

    expect(await screen.findByRole("heading", { name: "生成产物审核" }, { timeout: 10000 })).toBeInTheDocument()
    expect(await screen.findByText("连接 live Admin API 后才能执行管理操作", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "准备确认" })).toBeDisabled()
  })

  it("does not allow live generated artifact review when review-plan falls back to fixture", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const fetcher = vi.fn<typeof fetch>(async () =>
      jsonResponse({ code: "review_plan_unavailable", message: "offline" }, 503),
    )

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts/review?artifact_id=artifact-live&decision=accept")

      expect(await screen.findByText("审核计划不是 live Admin API 返回，不能执行确认。", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByRole("button", { name: "准备确认" })).toBeDisabled()
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("runs live Admin generated artifact review through review-plan and confirmed mutation", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      const method = init?.method ?? "GET"
      calls.push({
        method,
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/review-plan":
          return jsonResponse(adminGeneratedArtifactReviewPlanResponse("artifact-live", "accept"))
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/review":
          return jsonResponse(adminGeneratedArtifactReviewResponse("artifact-live", "accept"))
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts/review?artifact_id=artifact-live&decision=accept")

      expect(await screen.findByText("artifact-live", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("Metadata Authority apply")).toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe generated payload title")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("F:\\private\\source\\Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("file:///mnt/private/source/Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()

      await user.click(screen.getByRole("button", { name: "准备确认" }))
      await user.click(screen.getByRole("button", { name: "确认接受" }))

      expect(await screen.findByText("idempotent replay", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("accepted")).toBeInTheDocument()

      expect(calls).toContainEqual({
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact-live/review-plan",
        body: { decision: "accept" },
        authorization: "Bearer admin-token",
      })
      expect(calls).toContainEqual({
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact-live/review",
        body: { decision: "accept" },
        authorization: "Bearer admin-token",
      })
      await waitFor(() => {
        expect(
          calls.filter(
            (call) =>
              call.method === "POST" &&
              call.path === "/admin/v1/automation/generated-artifacts/artifact-live/review-plan",
          ).length,
        ).toBeGreaterThanOrEqual(2)
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("routes accepted generated artifact review into Metadata Authority apply", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"

      switch (`${method} ${url.pathname}`) {
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/review-plan":
          return jsonResponse(adminGeneratedArtifactReviewPlanResponse("artifact-live", "accept"))
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/review":
          return jsonResponse(adminGeneratedArtifactReviewResponse("artifact-live", "accept"))
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      const { router } = renderRoute("/admin/automation/generated-artifacts/review?artifact_id=artifact-live&decision=accept")

      await user.click(await screen.findByRole("button", { name: "准备确认" }, { timeout: 10000 }))
      await user.click(screen.getByRole("button", { name: "确认接受" }))
      await user.click(await screen.findByRole("button", { name: "进入 Metadata Authority apply" }, { timeout: 10000 }))

      await waitFor(() => {
        expect(router.state.location.pathname).toBe("/admin/automation/generated-artifacts/metadata-apply")
        expect(router.state.location.search).toMatchObject({ artifact_id: "artifact-live" })
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("does not allow live generated artifact metadata apply when apply-plan falls back to fixture", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const fetcher = vi.fn<typeof fetch>(async () =>
      jsonResponse({ code: "metadata_apply_plan_unavailable", message: "offline" }, 503),
    )

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts/metadata-apply?artifact_id=artifact-live")

      expect(await screen.findByText("应用计划不是 live Admin API 返回，不能执行确认。", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByRole("button", { name: "准备应用" })).toBeDisabled()
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("runs live Admin generated artifact metadata apply through apply-plan and confirmed mutation", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      const method = init?.method ?? "GET"
      calls.push({
        method,
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/metadata-apply-plan":
          return jsonResponse(adminGeneratedArtifactMetadataApplyPlanResponse("artifact-live"))
        case "POST /admin/v1/automation/generated-artifacts/artifact-live/metadata-apply":
          return jsonResponse(adminGeneratedArtifactMetadataApplyResponse("artifact-live"))
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts/metadata-apply?artifact_id=artifact-live")

      expect(await screen.findByRole("heading", { name: "Metadata Authority apply" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("title", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("field_locked", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("Provider Mapping 计划", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("tmdb-123", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("incoming_provider_subject")).toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe generated payload title")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("F:\\private\\source\\Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("file:///mnt/private/source/Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()

      await user.click(screen.getByRole("button", { name: "准备应用" }))
      await user.click(screen.getByRole("button", { name: "确认应用" }))

      expect(await screen.findByText("元数据应用结果已存在，已返回幂等结果", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("applied")).toBeInTheDocument()
      expect(screen.getByText("Apply mappings")).toBeInTheDocument()
      expect(screen.getByText("Provider Mapping 结果")).toBeInTheDocument()

      const applyCall = calls.find(
        (call) =>
          call.method === "POST" &&
          call.path === "/admin/v1/automation/generated-artifacts/artifact-live/metadata-apply",
      )
      expect(applyCall).toBeDefined()
      expect(applyCall?.body).toMatchObject({
        idempotency_key: expect.stringMatching(/^web-generated-artifact-metadata-apply:artifact-live:/),
      })
      expect(applyCall?.authorization).toBe("Bearer admin-token")
      expect(calls).toContainEqual({
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/artifact-live/metadata-apply-plan",
        body: undefined,
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("runs live Admin metadata candidate review apply without exposing unsafe review fields", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      const method = init?.method ?? "GET"
      calls.push({
        method,
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "GET /admin/v1/metadata/candidate-reviews/review-live":
          return jsonResponse(adminMetadataCandidateReviewResponse("review-live"))
        case "POST /admin/v1/metadata/candidate-reviews/review-live/apply":
          return jsonResponse(adminMetadataCandidateReviewApplyResponse("review-live"))
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/metadata/candidate-reviews?review_id=review-live")

      expect(await screen.findByRole("heading", { name: "Metadata Candidate Review" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("review-live", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("Live Candidate", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("1437/1", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("Root Provider Mapping")).toBeInTheDocument()
      expect(screen.getByText("Related preview only")).toBeInTheDocument()
      expect(screen.queryByText("secret candidate overview")).not.toBeInTheDocument()
      expect(screen.queryByText("secret related overview")).not.toBeInTheDocument()
      expect(screen.queryByText("secret-candidate-tag")).not.toBeInTheDocument()
      expect(screen.queryByText("local:///Private/Live.S01E01.mkv?token=secret")).not.toBeInTheDocument()
      expect(screen.queryByText("sha256-private-candidate")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()

      await user.click(screen.getByRole("button", { name: "准备应用" }))
      await user.click(screen.getByRole("button", { name: "确认应用" }))

      expect(await screen.findByText("Candidate Review 已应用，未产生新的 Provider Mapping 变更", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("idempotent replay")).toBeInTheDocument()
      expect(screen.getByText("0123456789abcdef")).toBeInTheDocument()
      expect(screen.getByText("mapping-live")).toBeInTheDocument()
      expect(screen.queryByText("web-metadata-candidate-review-apply:review-live")).not.toBeInTheDocument()

      expect(calls).toContainEqual({
        method: "GET",
        path: "/admin/v1/metadata/candidate-reviews/review-live",
        body: undefined,
        authorization: "Bearer admin-token",
      })
      const applyCall = calls.find(
        (call) =>
          call.method === "POST" &&
          call.path === "/admin/v1/metadata/candidate-reviews/review-live/apply",
      )
      expect(applyCall).toBeDefined()
      expect(applyCall?.body).toMatchObject({
        item_id: "item-live",
        expected_updated_at_ms: 300,
        idempotency_key: expect.stringMatching(
          /^web-metadata-candidate-review-apply:review-live:/,
        ),
      })
      expect(applyCall?.authorization).toBe("Bearer admin-token")
      await waitFor(() => {
        expect(
          calls.filter(
            (call) =>
              call.method === "GET" &&
              call.path === "/admin/v1/metadata/candidate-reviews/review-live",
          ).length,
        ).toBeGreaterThanOrEqual(2)
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("navigates item-scoped Admin metadata candidate review lists into existing detail", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      calls.push({
        method,
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "GET /admin/v1/metadata/items/item-live/candidate-reviews":
          return jsonResponse(adminMetadataCandidateReviewListResponse("item-live"))
        case "GET /admin/v1/metadata/candidate-reviews/review-live-older":
          return jsonResponse(adminMetadataCandidateReviewResponse("review-live-older"))
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      const { router } = renderRoute(
        "/admin/metadata/candidate-reviews?item_id=item-live&limit=25&offset=0",
      )

      expect(await screen.findByText("review-live-newer", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("Newer Candidate", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("review-live-older")).toBeInTheDocument()
      expect(screen.queryByText("secret candidate overview")).not.toBeInTheDocument()
      expect(screen.queryByText("local:///Private/Live.S01E02.mkv?token=secret")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()

      await user.click(
        screen.getByRole("button", {
          name: /查看 Candidate Review review-live-older/,
        }),
      )

      expect(router.state.location.pathname).toBe("/admin/metadata/candidate-reviews")
      expect(router.state.location.search).toMatchObject({
        item_id: "item-live",
        review_id: "review-live-older",
        limit: 25,
      })
      expect(await screen.findByText("Root Provider Mapping", {}, { timeout: 10000 })).toBeInTheDocument()

      const candidateReviewCalls = calls.filter((call) =>
        call.path.includes("/admin/v1/metadata/"),
      )
      expect(candidateReviewCalls).toEqual([
        {
          method: "GET",
          path: "/admin/v1/metadata/items/item-live/candidate-reviews?limit=25&offset=0",
          authorization: "Bearer admin-token",
        },
        {
          method: "GET",
          path: "/admin/v1/metadata/candidate-reviews/review-live-older",
          authorization: "Bearer admin-token",
        },
      ])
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("runs live Admin generated artifact metadata bulk apply through plan, confirm, and status", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      const method = init?.method ?? "GET"
      calls.push({
        method,
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "GET /admin/v1/automation/generated-artifacts/proposals":
          return jsonResponse({
            admin_api_version: "v1",
            public_api_version: "v1",
            proposals: [adminGeneratedArtifactAcceptedProposal("artifact-bulk-accepted")],
            page: routePage({ limit: 50, offset: 0 }),
          })
        case "POST /admin/v1/automation/generated-artifacts/metadata-apply-plan":
          return jsonResponse(
            adminGeneratedArtifactMetadataBulkApplyPlanResponse(["artifact-bulk-accepted"]),
          )
        case "POST /admin/v1/automation/generated-artifacts/metadata-apply-batches":
          return jsonResponse(
            adminGeneratedArtifactMetadataBulkApplyBatchResponse(["artifact-bulk-accepted"]),
          )
        case "GET /admin/v1/automation/generated-artifacts/metadata-apply-batches/bulk-batch-live":
          return jsonResponse(
            adminGeneratedArtifactMetadataBulkApplyBatchResponse(["artifact-bulk-accepted"]),
          )
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts")

      expect(await screen.findByText("artifact-bulk-accepted", {}, { timeout: 10000 })).toBeInTheDocument()
      await user.click(
        screen.getByRole("checkbox", { name: "选择批量应用 artifact-bulk-accepted" }),
      )
      await user.click(screen.getByRole("button", { name: "生成批量计划" }))

      expect(await screen.findByText("Apply fields", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getAllByText("Apply mappings").length).toBeGreaterThan(0)
      expect(screen.getByText("mapping apply 1 · skip 0 · noop 1")).toBeInTheDocument()
      await user.click(screen.getByRole("button", { name: "准备确认批量应用" }))
      await user.click(screen.getByRole("button", { name: "确认批量应用" }))

      expect(await screen.findByText("批量元数据应用已完成", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(screen.getByText("bulk-batch-live")).toBeInTheDocument()
      expect(screen.getAllByText("Provider Mapping").length).toBeGreaterThan(0)
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe generated payload title")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe-bulk-idempotency-key")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()

      expect(calls).toContainEqual({
        method: "POST",
        path: "/admin/v1/automation/generated-artifacts/metadata-apply-plan",
        body: { artifact_ids: ["artifact-bulk-accepted"] },
        authorization: "Bearer admin-token",
      })
      const confirmCall = calls.find(
        (call) =>
          call.method === "POST" &&
          call.path === "/admin/v1/automation/generated-artifacts/metadata-apply-batches",
      )
      expect(confirmCall?.body).toMatchObject({
        artifact_ids: ["artifact-bulk-accepted"],
        idempotency_key: expect.stringMatching(
          /^web-generated-artifact-metadata-bulk-apply:1:artifact-bulk-accepted:/,
        ),
      })
      expect(confirmCall?.authorization).toBe("Bearer admin-token")
      expect(calls).toContainEqual({
        method: "GET",
        path: "/admin/v1/automation/generated-artifacts/metadata-apply-batches/bulk-batch-live",
        body: undefined,
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("writes Media search submits to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/search")

    const input = await screen.findByPlaceholderText("搜索电影、剧集、演员...", {}, { timeout: 10000 })
    await user.type(input, "dune")
    const searchButtons = screen.getAllByRole("button", { name: "搜索" })
    await user.click(searchButtons[searchButtons.length - 1])

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ q: "dune" })
    })
  })

  it("writes Media playlist tab and view state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/my-list")

    const favoritesTab = await screen.findByRole("tab", { name: /收藏/ }, { timeout: 10000 })
    await user.click(favoritesTab)

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/media/my-list")
      expect(router.state.location.search).toMatchObject({ playlist: "fixture-favorites" })
    })

    await user.click(await screen.findByRole("button", { name: "列表视图" }))

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        playlist: "fixture-favorites",
        view: "list",
      })
    })
  })

  it("creates a Media playlist and writes the selected playlist to the URL", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlists = [publicUserPlaylist()]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists,
          page: {
            limit: 20,
            offset: 0,
            returned: playlists.length,
          },
        })
      }

      if (method === "POST" && url.pathname === "/users/me/playlists") {
        const created = publicUserPlaylist({
          id: "playlist-new",
          name: (body as { name: string }).name,
          item_count: 0,
          version: 1,
        })
        playlists = [...playlists, created]

        return jsonResponse({ playlist: created })
      }

      if (method === "GET" && url.pathname.startsWith("/users/me/playlists/")) {
        const playlistId = url.pathname.split("/")[4]
        const playlist = playlists.find((entry) => entry.id === playlistId) ?? playlists[0]

        return jsonResponse({
          playlist,
          items: [],
          page: {
            limit: 50,
            offset: 0,
            returned: 0,
          },
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    const { router } = renderRoute("/media/my-list?playlist=playlist-live")

    expect(await screen.findAllByText("Live Playlist", {}, { timeout: 10000 })).toHaveLength(2)
    await user.click(await screen.findByRole("button", { name: "新建播放列表" }))
    await user.type(await screen.findByLabelText("播放列表名称"), "Weekend Queue")
    await user.click(screen.getByRole("button", { name: "创建播放列表" }))

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ playlist: "playlist-new" })
    })
    expect(calls).toContainEqual({
      method: "POST",
      path: "/users/me/playlists",
      body: { name: "Weekend Queue" },
    })
  })

  it("renames and deletes the active Media playlist without leaving stale URL state", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlists = [
      publicUserPlaylist(),
      publicUserPlaylist({
        id: "playlist-archive",
        name: "Archive Playlist",
        item_count: 0,
        version: 1,
      }),
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists,
          page: {
            limit: 20,
            offset: 0,
            returned: playlists.length,
          },
        })
      }

      if (method === "PATCH" && url.pathname === "/users/me/playlists/playlist-live") {
        const renamed = publicUserPlaylist({
          name: (body as { name: string }).name,
          version: 3,
        })
        playlists = playlists.map((playlist) => (playlist.id === renamed.id ? renamed : playlist))

        return jsonResponse({ playlist: renamed })
      }

      if (method === "DELETE" && url.pathname === "/users/me/playlists/playlist-live") {
        playlists = playlists.filter((playlist) => playlist.id !== "playlist-live")

        return jsonResponse({ playlist_id: "playlist-live", deleted: true })
      }

      if (method === "GET" && url.pathname.startsWith("/users/me/playlists/")) {
        const playlistId = url.pathname.split("/")[4]
        const playlist = playlists.find((entry) => entry.id === playlistId) ?? playlists[0]

        return jsonResponse({
          playlist,
          items: [],
          page: {
            limit: 50,
            offset: 0,
            returned: 0,
          },
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    const { router } = renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findAllByText("Live Playlist", {}, { timeout: 10000 })).toHaveLength(2)
    await user.click(await screen.findByRole("button", { name: "重命名播放列表" }))
    const nameInput = await screen.findByLabelText("播放列表名称")
    await user.clear(nameInput)
    await user.type(nameInput, "Renamed Playlist")
    await user.click(screen.getByRole("button", { name: "保存名称" }))

    await waitFor(() => {
      expect(screen.getAllByText("Renamed Playlist")).toHaveLength(2)
    })
    expect(calls).toContainEqual({
      method: "PATCH",
      path: "/users/me/playlists/playlist-live",
      body: { name: "Renamed Playlist", expected_version: 2 },
    })

    await user.click(screen.getByRole("button", { name: "删除播放列表" }))
    await user.click(await screen.findByRole("button", { name: "确认删除" }))

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        playlist: "playlist-archive",
        view: "list",
      })
    })
    expect(calls).toContainEqual({
      method: "DELETE",
      path: "/users/me/playlists/playlist-live",
      body: undefined,
    })
  })

  it("keeps fixture Media playlist mutations truthful and does not update URL state", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/my-list")

    await user.click(await screen.findByRole("button", { name: "新建播放列表" }, { timeout: 10000 }))
    await user.type(await screen.findByLabelText("播放列表名称"), "Fixture Queue")
    await user.click(screen.getByRole("button", { name: "创建播放列表" }))

    expect(
      await screen.findByText("Fixture mode does not persist playlist mutations.", {}, { timeout: 10000 }),
    ).toBeInTheDocument()
    expect(router.state.location.search).not.toMatchObject({ playlist: "Fixture Queue" })
  })

  it("removes an item from the active Media playlist through the Public Client route", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlistItems = [
      {
        playlist_id: "playlist-live",
        item_id: "live-movie",
        position: 0,
        added_at: "2026-05-29T01:00:00Z",
        item: publicMediaItem(),
        images: [],
      },
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist({ item_count: playlistItems.length })],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist({ item_count: playlistItems.length }),
          items: playlistItems,
          page: {
            limit: 50,
            offset: 0,
            returned: playlistItems.length,
          },
        })
      }

      if (method === "DELETE" && url.pathname === "/users/me/playlists/playlist-live/items/live-movie") {
        playlistItems = []

        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: 0,
            version: 3,
          }),
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findByText("Live Movie", {}, { timeout: 10000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "从播放列表移除 Live Movie" }))

    await waitFor(() => {
      expect(screen.getByText("列表为空")).toBeInTheDocument()
    })
    expect(calls).toContainEqual({
      method: "DELETE",
      path: "/users/me/playlists/playlist-live/items/live-movie",
      body: undefined,
    })
  })

  it("reorders Media playlist items through the Public Client route without changing URL state", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlistVersion = 2
    let playlistItems = [
      publicPlaylistItem("live-movie-a", "Live Movie A", 0),
      publicPlaylistItem("live-movie-b", "Live Movie B", 1),
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion })],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion }),
          items: playlistItems,
          page: {
            limit: 50,
            offset: 0,
            returned: playlistItems.length,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/reorder") {
        const itemIds = (body as { item_ids: string[] }).item_ids
        playlistItems = itemIds.map((itemId, index) => {
          const item = playlistItems.find((entry) => entry.item_id === itemId)
          if (!item) {
            throw new Error(`unknown playlist item: ${itemId}`)
          }

          return { ...item, position: index }
        })
        playlistVersion = 3

        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: playlistItems.length,
            version: playlistVersion,
          }),
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    const { router } = renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findByText("Live Movie A", {}, { timeout: 10000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "将 Live Movie B 上移" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/reorder",
        body: {
          item_ids: ["live-movie-b", "live-movie-a"],
          expected_version: 2,
        },
      })
    })
    await waitFor(() => {
      expect(
        screen
          .getAllByRole("heading", { level: 3 })
          .map((heading) => heading.textContent),
      ).toEqual(["Live Movie B", "Live Movie A"])
    })
    expect(router.state.location.search).toMatchObject({
      playlist: "playlist-live",
      view: "list",
    })
  })

  it("refetches playlist items when reorder returns a stale version conflict", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlistVersion = 2
    let playlistItems = [
      publicPlaylistItem("live-movie-a", "Live Movie A", 0),
      publicPlaylistItem("live-movie-b", "Live Movie B", 1),
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion })],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion }),
          items: playlistItems,
          page: {
            limit: 50,
            offset: 0,
            returned: playlistItems.length,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/reorder") {
        playlistItems = [
          publicPlaylistItem("live-movie-b", "Live Movie B", 0),
          publicPlaylistItem("live-movie-a", "Live Movie A", 1),
        ]
        playlistVersion = 4

        return jsonResponse(
          {
            code: "conflict",
            message: "Playlist version conflict",
          },
          409,
        )
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findByText("Live Movie A", {}, { timeout: 10000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "将 Live Movie B 上移" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/reorder",
        body: {
          item_ids: ["live-movie-b", "live-movie-a"],
          expected_version: 2,
        },
      })
    })
    expect(await screen.findByText("Playlist version conflict", {}, { timeout: 10000 })).toBeInTheDocument()
    await waitFor(() => {
      expect(
        calls.filter((call) => call.method === "GET" && call.path === "/users/me/playlists/playlist-live/items")
          .length,
      ).toBeGreaterThan(1)
    })
    await waitFor(() => {
      expect(
        screen
          .getAllByRole("heading", { level: 3 })
          .map((heading) => heading.textContent),
      ).toEqual(["Live Movie B", "Live Movie A"])
    })
  })

  it("adds detail media to a selected playlist through the Public Client route", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/items/live-movie") {
        return jsonResponse({
          item: publicMediaItem(),
          sources: [],
          images: [],
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist()],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/live-movie") {
        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: 2,
            version: 3,
          }),
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    renderRoute("/media/detail?id=live-movie&type=movie")

    expect(await screen.findByRole("heading", { name: "Live Movie" }, { timeout: 10000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "添加到播放列表" }))
    await user.click(await screen.findByRole("menuitem", { name: "添加到 Live Playlist" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: {},
      })
    })
  })

  it("renders live Media detail Management Context Links without leaking unsafe targets", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const calls: string[] = []
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))
      calls.push(`${url.pathname}${url.search}`)

      if (url.pathname === "/items/live-movie") {
        return jsonResponse({
          item: publicMediaItem(),
          sources: [publicMediaSource()],
          images: [],
        })
      }

      if (url.pathname === "/management/context-links") {
        return jsonResponse(
          managementContextLinksResponse({
            context: {
              library_id: "library-a",
              item_id: "live-movie",
              source_id: "source-live",
              playback_session_id: null,
            },
            links: [
              managementContextLink({
                action: "refresh_item_metadata",
                method: "POST",
                route_name: "item.metadata_refresh",
                target: {
                  library_id: "library-a",
                  item_id: "live-movie",
                  source_id: "source-live",
                  playback_session_id: null,
                },
              }),
              managementContextLink({
                enabled: false,
                disabled_reason: "insufficient_permission",
                route_name: "library.scan",
                target: {
                  library_id: "library-a",
                  item_id: "live-movie",
                  source_id: "source-live",
                  playback_session_id: null,
                },
              }),
              managementContextLink({
                action: "view_playback_diagnostics",
                method: "GET",
                route_name: "playback.support",
                target: {
                  library_id: "library-a",
                  item_id: "live-movie",
                  source_id: "file:///mnt/private/source/Movie.mkv",
                  playback_session_id: null,
                },
              }),
              managementContextLink({
                route_name: "unknown.future_route",
              }),
            ],
          }),
        )
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/media/detail?id=live-movie&type=movie")

      const refreshLink = await screen.findByRole("link", { name: /刷新元数据/ }, { timeout: 10000 })
      expect(refreshLink).toHaveAttribute(
        "href",
        "/admin/libraries?library_id=library-a&item_id=live-movie&media_type=movie&source_id=source-live&intent=refresh_item_metadata",
      )
      expect(screen.getByRole("button", { name: /扫描媒体库\s*权限不足/ })).toBeDisabled()
      expect(screen.queryByRole("link", { name: /播放诊断/ })).not.toBeInTheDocument()
      expect(screen.queryByText(/unknown.future_route/)).not.toBeInTheDocument()
      expect(document.body.textContent).not.toContain("file:///mnt/private/source/Movie.mkv")
      expect(calls).toContain(
        "/management/context-links?library_id=library-a&item_id=live-movie&source_id=source-live",
      )
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
    }
  })

  it("renders live library Management Context Links from the Public Client route", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))

      if (url.pathname === "/libraries/movies") {
        return jsonResponse({
          library: publicLibrary("movies"),
        })
      }

      if (url.pathname === "/libraries/movies/sources") {
        return jsonResponse({
          library: publicLibrary("movies"),
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
          sources: [publicLibrarySource()],
        })
      }

      if (url.pathname === "/libraries/movies/items") {
        return jsonResponse({
          library: publicLibrary("movies"),
          page: {
            limit: 50,
            offset: 0,
            returned: 1,
          },
          items: [publicMediaItem()],
        })
      }

      if (url.pathname === "/management/context-links") {
        return jsonResponse(
          managementContextLinksResponse({
            context: {
              library_id: "movies",
              item_id: null,
              source_id: null,
              playback_session_id: null,
            },
            links: [
              managementContextLink({
                route_name: "library.scan",
                target: {
                  library_id: "movies",
                  item_id: null,
                  source_id: null,
                  playback_session_id: null,
                },
              }),
            ],
          }),
        )
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/media/library?id=movies")

      const scanLink = await screen.findByRole("link", { name: /扫描媒体库/ }, { timeout: 10000 })
      expect(scanLink).toHaveAttribute(
        "href",
        "/admin/libraries?library_id=movies&intent=scan_library",
      )
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
    }
  })

  it("renders Admin task management context from safe route params without leaking unsafe targets", async () => {
    renderRoute(
      "/admin/tasks?context=management_link&library_id=library-a&item_id=live-movie&source_id=file:///mnt/private/source.mkv&playback_session_id=session-a",
    )

    expect(await screen.findByText("管理上下文", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByText("library-a")).toBeInTheDocument()
    expect(screen.getByText("live-movie")).toBeInTheDocument()
    expect(screen.getByText("session-a")).toBeInTheDocument()
    expect(document.body.textContent).not.toContain("file:///mnt/private/source.mkv")
  })

  it("renders Admin playback management panels from safe route params", async () => {
    const support = renderRoute(
      "/admin/transcoding?panel=support&source_id=source-live&playback_session_id=session-a",
    )

    expect(await screen.findByText("播放诊断", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByText("source-live")).toBeInTheDocument()
    expect(screen.getByText("session-a")).toBeInTheDocument()
    support.unmount()

    renderRoute(
      "/admin/transcoding?panel=runtime&library_id=library-a&item_id=live-movie&source_id=file:///mnt/private/source.mkv",
    )

    expect(await screen.findByText("转码运行时", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByText("library-a")).toBeInTheDocument()
    expect(screen.getByText("live-movie")).toBeInTheDocument()
    expect(document.body.textContent).not.toContain("file:///mnt/private/source.mkv")
  })

  it("renders Admin library access management panel from safe route params", async () => {
    renderRoute(
      "/admin/users?panel=library_access&library_id=library-a&source_id=file:///mnt/private/source.mkv",
    )

    expect(await screen.findByText("访问策略", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByText("library-a")).toBeInTheDocument()
    expect(document.body.textContent).not.toContain("file:///mnt/private/source.mkv")
  })

  it("runs Admin library scan intent from Management Context Link route with safe Media returns", async () => {
    const user = userEvent.setup()
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{ method: string; path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      calls.push({
        method,
        path: url.pathname,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "GET /admin/v1/overview":
          return jsonResponse(adminRouteOverviewResponse())
        case "GET /admin/v1/system/config":
          return jsonResponse(adminRouteSystemConfigResponse())
        case "GET /admin/v1/jobs":
          return jsonResponse(adminRouteJobsResponse())
        case "GET /admin/v1/playback/sessions":
          return jsonResponse(adminRoutePlaybackSessionsResponse())
        case "GET /admin/v1/playback/runtime":
          return jsonResponse(adminRoutePlaybackRuntimeResponse())
        case "POST /admin/v1/libraries/library-a/scan":
          return jsonResponse(adminRouteJob())
        case "GET /items/live-movie":
          return jsonResponse({
            item: publicMediaItem(),
            sources: [publicMediaSource()],
            images: [],
          })
        case "GET /libraries/library-a":
          return jsonResponse({
            library: publicLibrary("library-a"),
          })
        case "GET /libraries/library-a/sources":
          return jsonResponse({
            library: publicLibrary("library-a"),
            page: routePage(),
            sources: [publicLibrarySource({ library_id: "library-a" })],
          })
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    vi.spyOn(window, "confirm").mockReturnValue(true)
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute(
        "/admin/libraries?library_id=library-a&item_id=live-movie&media_type=movie&source_id=file:///mnt/private/source.mkv&intent=scan_library",
      )

      expect(await screen.findByText("扫描媒体库", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByRole("link", { name: "返回媒体库" })).toHaveAttribute(
        "href",
        "/media/library?id=library-a",
      )
      expect(await screen.findByRole("link", { name: "返回媒体详情" })).toHaveAttribute(
        "href",
        "/media/detail?id=live-movie&type=movie",
      )
      expect(document.body.textContent).not.toContain("file:///mnt/private/source.mkv")

      const scanButton = screen.getByRole("button", { name: "确认扫描媒体库" })
      await waitFor(() => {
        expect(scanButton).not.toBeDisabled()
      })
      await user.click(scanButton)

      await waitFor(() => {
        expect(calls).toContainEqual({
          method: "POST",
          path: "/admin/v1/libraries/library-a/scan",
          authorization: "Bearer admin-token",
        })
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("renders Admin library metadata and item refresh handoff targets", async () => {
    const profileTarget = renderRoute("/admin/libraries?library_id=library-a&panel=metadata_profile")

    expect(await screen.findByText("元数据配置", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByText("library-a")).toBeInTheDocument()
    profileTarget.unmount()

    renderRoute("/admin/libraries?library_id=library-a&item_id=live-movie&intent=refresh_item_metadata")

    expect(await screen.findByText("刷新媒体项元数据", {}, { timeout: 10000 })).toBeInTheDocument()
    expect(screen.getByText("live-movie")).toBeInTheDocument()
    expect(screen.getByRole("link", { name: /查看相关任务/ })).toHaveAttribute(
      "href",
      "/admin/tasks?context=management_link&library_id=library-a&item_id=live-movie",
    )
  })

  it("adds browse media cards to a selected playlist through the Public Client route", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/items") {
        return jsonResponse({
          items: [publicMediaItem()],
          page: {
            limit: 40,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist()],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/live-movie") {
        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: 2,
            version: 3,
          }),
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })
    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)
    renderRoute("/media")

    await user.click(await screen.findByRole("button", { name: "添加 Live Movie 到播放列表" }, { timeout: 10000 }))
    await user.click(await screen.findByRole("menuitem", { name: "添加到 Live Playlist" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: {},
      })
    })
  })
})

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
}

function restoreStorage(storage: Storage, key: string, value: string | null) {
  if (value === null) {
    storage.removeItem(key)
    return
  }

  storage.setItem(key, value)
}

function routePage(overrides: Record<string, unknown> = {}) {
  return {
    limit: 50,
    offset: 0,
    returned: 1,
    ...overrides,
  }
}

function adminRouteOverviewResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    status: "healthy",
    storage: {
      total_backends: 1,
      ready_backends: 1,
      degraded_backends: 0,
      unavailable_backends: 0,
      backends: [
        {
          library_id: "library-a",
          library_name: "Movies",
          backend_kind: "local",
          status: "ready",
        },
      ],
    },
    metadata: {
      total_providers: 1,
      available_providers: 1,
      disabled_providers: 0,
      unavailable_providers: 0,
      providers: [],
    },
    runtime: {
      active_tasks: 1,
      completed_tasks: 0,
      failed_tasks: 0,
      succeeded_jobs: 0,
      cancelled_jobs: 0,
      failed_jobs: 0,
      shutdown_requested: false,
    },
    startup: {
      configured_libraries: 1,
      recovered_transcode_sessions: 0,
      recovered_jobs: 0,
      staging_deleted_records: 0,
      staging_deleted_files: 0,
      metadata_raw_cache_deleted: 0,
      metadata_lifecycle_tasks_started: 0,
      artwork_ingest_worker_started: true,
    },
  }
}

function adminRouteSystemConfigResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    auth: {
      enabled: true,
      token_env: "NAKO_ADMIN_TOKEN",
    },
    network: {
      exposure_mode: "private_network",
      readiness: {
        status: "ready",
        reason: "ready",
        checks: [],
      },
      external_endpoint: {
        configured: false,
        scheme: null,
        host_fingerprint: null,
      },
      trusted_proxy: {
        headers_enabled: false,
        source_count: 0,
      },
      origins: {
        allowed_origin_count: 1,
        configured: true,
      },
      tunnel_providers: [],
    },
    database: {
      configured_backend_kind: "sqlite",
      active_backend_kind: "sqlite",
      url_scheme: "sqlite",
      runtime_supported: true,
      migrated_on_startup: true,
      capabilities: {
        lifecycle: true,
        libraries: true,
        jobs: true,
        job_leases: true,
        media: true,
        scan_commits: true,
        metadata: true,
        catalog: true,
        playback_state: true,
        playback_sessions: true,
        transcode_sessions: true,
        event_outbox: true,
        addons: true,
        automation: true,
        managed_artwork: true,
        vfs_cache: true,
        webhooks: true,
        search_index: true,
      },
    },
    runtime: {
      listen_addr: "0.0.0.0:8096",
      scan_concurrency: 2,
      probe_concurrency: 4,
      metadata_concurrency: 3,
      remux_concurrency: 2,
      webhook_concurrency: 1,
      remux_timeout_ms: 120000,
    },
    libraries: [
      {
        id: "library-a",
        name: "Movies",
        preset: "movie",
        backend_kind: "local",
        root_scheme: "file",
        has_webdav_password_env: false,
        webdav_timeout_ms: null,
        webdav_max_attempts: null,
      },
    ],
    metadata: {
      raw_cache_retention_ms: 86400000,
      raw_cache_cleanup_on_startup: true,
      raw_cache_cleanup_interval_ms: 3600000,
      maintenance_policies: 1,
      providers: [
        {
          provider: "tmdb",
          enabled: true,
          token_env: null,
          api_key_env: "TMDB_API_KEY",
          has_api_base_url: true,
          has_image_base_url: true,
          language: "zh-CN",
          include_adult: false,
          header_count: 0,
          secret_header_count: 0,
          has_provider_runtime_override: false,
        },
      ],
      runtime: {
        timeout_ms: 10000,
        max_attempts: 3,
        min_interval_ms: 100,
        concurrency: 4,
        user_agent: "nako-test",
        has_proxy: false,
        circuit_breaker_failures: 3,
        circuit_breaker_backoff_ms: 1000,
      },
    },
    transcode: {
      hardware_policy: {},
      cpu_concurrency: 2,
      gpu_concurrency: 1,
    },
    staging: {
      max_bytes: 1024,
      retention_ms: 86400000,
      cleanup_on_startup: true,
    },
    playback: {
      remote_stream_concurrency: 4,
      remote_stage_concurrency: 2,
      transcode_artifact_retention_ms: 86400000,
      transcode_artifact_cleanup_on_startup: true,
      hls_segment_cleanup_enabled: true,
      hls_segment_keep_ms: 60000,
      transcode_throttle_enabled: false,
      transcode_throttle_delay_ms: 0,
    },
    artwork: {
      artifact_root_configured: true,
      fetch_timeout_ms: 10000,
      fetch_max_attempts: 3,
      fetch_max_bytes: 4096,
      fetch_concurrency: 2,
      ingest_worker_enabled: true,
      ingest_worker_idle_ms: 1000,
      fetch_user_agent: "nako-test",
      has_fetch_proxy: false,
      max_width: 3000,
      max_height: 3000,
    },
  }
}

function adminRouteJob() {
  return {
    id: "job-1",
    kind: "library_scan",
    status: "running",
    resource_class: "library",
    library_id: "library-a",
    source_id: null,
    has_input: true,
    has_summary: false,
    has_error: false,
    queued_at: "2026-05-30T01:00:00Z",
    started_at: "2026-05-30T01:01:00Z",
    completed_at: null,
  }
}

function adminRouteJobsResponse() {
  return {
    jobs: [adminRouteJob()],
    page: routePage(),
  }
}

function adminRoutePlaybackSessionsResponse() {
  return {
    sessions: [],
    page: routePage({ returned: 0 }),
  }
}

function adminRoutePlaybackRuntimeResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    readiness: {
      status: "ready",
      reason: "ready",
      checks: [],
    },
  }
}

function adminAcquisitionIntakeCandidatesResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    candidates: [
      {
        id: "candidate-live",
        target_library_id: "library-a",
        source_kind: "watch_folder",
        custom_source_kind: false,
        source_scheme: "file",
        source_ref_redacted: "file://<redacted>/Live.mkv",
        source_key_fingerprint: "sha256:candidate-live",
        has_display_name: true,
        has_intended_locator: true,
        size_bytes: 123456789,
        has_fingerprint: true,
        managed_import_artifact_id: "artifact-live",
        state: "ready",
        has_diagnostics: true,
        first_seen_at_ms: 1710468000000,
        last_seen_at_ms: 1710468300000,
        created_at_ms: 1710468000000,
        updated_at_ms: 1710468300000,
        intended_locator: "file:///mnt/private/raw/Live.mkv",
        prompt_body: "unsafe prompt body",
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function adminGeneratedArtifactProposalsResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    proposals: [
      {
        id: "artifact-live",
        kind: "metadata_suggestion",
        capability: "item_metadata_suggest",
        status: "pending_review",
        target: {
          kind: "media_item",
          library_id: "library-a",
          item_id: "item-live",
          source_id: "source-live",
          local_path: "F:\\private\\source\\Movie.mkv",
          source_locator: "file:///mnt/private/source/Movie.mkv",
        },
        provenance: {
          provider_id: "provider-live",
          provider_name: "Live Automation Provider",
          job_id: "job-live",
          capability: "item_metadata_suggest",
          idempotency_key_fingerprint: "sha256:idempotency-live",
          prompt_fingerprint: "sha256:prompt-live",
          attempt_count: 2,
          artifact_created_at: "2026-05-29T01:00:00Z",
          raw_prompt: "unsafe prompt body",
          provider_raw_response: "provider secret response",
        },
        payload: {
          valid_json: true,
          shape: "object",
          payload_fingerprint: "sha256:payload-live",
          payload_bytes: 4096,
          object_field_count: 9,
          array_item_count: null,
          has_textual_values: true,
          has_explanation: true,
          confidence_milli: 910,
          raw_payload: {
            title: "unsafe generated payload title",
          },
        },
        readiness: {
          status: "ready",
          actionable: true,
          reasons: ["ready_for_review"],
        },
        created_at: "2026-05-29T01:01:00Z",
        updated_at: "2026-05-29T01:05:00Z",
        accepted_at: null,
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function adminMetadataCandidateReviewResponse(reviewId = "review-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    review: adminMetadataCandidateReviewDetail(reviewId),
    application_plan: adminMetadataCandidateReviewApplicationPlan(reviewId, "apply", ["ready"]),
    boundary: adminMetadataCandidateReviewBoundary(),
    raw_provider_response: "provider secret response",
    idempotency_key: "candidate-review:operator-secret",
  }
}

function adminMetadataCandidateReviewListResponse(itemId = "item-live") {
  const newer = adminMetadataCandidateReviewDetail("review-live-newer")
  const older = adminMetadataCandidateReviewDetail("review-live-older")

  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    item_id: itemId,
    reviews: [
      {
        review_id: newer.review_id,
        item_id: itemId,
        source: { provider: "bangumi" },
        source_key: "bangumi:newer",
        status: "pending",
        root: {
          ...newer.root,
          metadata: adminMetadataCandidateSummary("Newer Candidate", {
            raw_overview: "secret candidate overview",
            raw_tags: ["secret-candidate-tag"],
            source_locator: "local:///Private/Live.S01E02.mkv?token=secret",
            source_fingerprint: "sha256-private-list-candidate",
          }),
        },
        related_count: 1,
        relationship_count: 1,
        application_plan: adminMetadataCandidateReviewApplicationPlan(
          newer.review_id,
          "skip",
          ["review_status_not_accepted"],
        ),
        boundary: adminMetadataCandidateReviewBoundary({
          apply_mutation_required: false,
          apply_updates_root_provider_subject: false,
          apply_updates_root_provider_mapping: false,
        }),
        expires_at_ms: null,
        created_at_ms: 100,
        updated_at_ms: 500,
        raw_provider_response: "provider secret response",
      },
      {
        review_id: older.review_id,
        item_id: itemId,
        source: { provider: "bangumi" },
        source_key: "bangumi:older",
        status: "accepted",
        root: older.root,
        related_count: 0,
        relationship_count: 0,
        application_plan: adminMetadataCandidateReviewApplicationPlan(
          older.review_id,
          "apply",
          ["ready"],
        ),
        boundary: adminMetadataCandidateReviewBoundary(),
        expires_at_ms: null,
        created_at_ms: 90,
        updated_at_ms: 300,
      },
    ],
    page: {
      limit: 25,
      offset: 0,
      returned: 2,
    },
  }
}

function adminMetadataCandidateReviewApplyResponse(reviewId = "review-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    review_id: reviewId,
    item_id: "item-live",
    applied: true,
    changed: false,
    idempotent_replay: true,
    idempotency_key_fingerprint: "0123456789abcdef",
    plan: adminMetadataCandidateReviewApplicationPlan(
      reviewId,
      "noop",
      ["existing_accepted_mapping"],
    ),
    provider_subject: {
      subject_id: "subject-live",
      provider: "bangumi",
      subject_kind: "subject",
      subject_key: "1437",
      title: "Live Candidate",
      release_year: 2026,
      locale: "zh-CN",
      raw_subject_payload: "provider secret response",
    },
    provider_mapping: {
      mapping_id: "mapping-live",
      item_id: "item-live",
      subject_id: "subject-live",
      status: "accepted",
      confidence_milli: 940,
      source: "user",
      raw_provider_mapping: "provider secret response",
    },
    boundary: adminMetadataCandidateReviewBoundary({
      read_only: false,
      applies_on_read: false,
      apply_mutation_required: true,
    }),
    idempotency_key: "web-metadata-candidate-review-apply:review-live:test",
  }
}

function adminMetadataCandidateReviewDetail(reviewId = "review-live") {
  const rootSubject = adminMetadataCandidateSubject("subject", "1437", "Live Candidate")
  const childSubject = adminMetadataCandidateSubject("episode", "1437/1", "Episode One")

  return {
    review_id: reviewId,
    item_id: "item-live",
    source: { provider: "bangumi" },
    source_key: "bangumi:1437",
    status: "accepted",
    root: {
      source: { provider: "bangumi" },
      kind: "series",
      subject: rootSubject,
      metadata: adminMetadataCandidateSummary("Live Candidate", {
        raw_overview: "secret candidate overview",
        raw_tags: ["secret-candidate-tag"],
        source_locator: "local:///Private/Live.S01E01.mkv?token=secret",
        source_fingerprint: "sha256-private-candidate",
      }),
    },
    related: [
      {
        source: { provider: "bangumi" },
        kind: "episode",
        subject: childSubject,
        metadata: adminMetadataCandidateSummary("Episode One", {
          raw_overview: "secret related overview",
        }),
      },
    ],
    relationships: [
      {
        parent_subject: rootSubject,
        child_subject: childSubject,
        kind: "contains",
      },
    ],
    related_count: 1,
    relationship_count: 1,
    expires_at_ms: null,
    created_at_ms: 100,
    updated_at_ms: 300,
  }
}

function adminMetadataCandidateReviewApplicationPlan(
  reviewId: string,
  action: "apply" | "skip" | "noop",
  reasons: string[],
) {
  return {
    review_id: reviewId,
    item_id: "item-live",
    action,
    reasons,
    source: "user",
    root_subject: adminMetadataCandidateSubject("subject", "1437", "Live Candidate"),
    existing_mapping_id: action === "noop" ? "mapping-live" : null,
    existing_mapping_status: action === "noop" ? "accepted" : null,
    raw_provider_response: "provider secret response",
  }
}

function adminMetadataCandidateReviewBoundary(
  overrides: Record<string, boolean> = {},
) {
  return {
    read_only: true,
    applies_on_read: false,
    apply_mutation_required: true,
    apply_updates_root_provider_subject: true,
    apply_updates_root_provider_mapping: true,
    apply_updates_related_provider_subjects: false,
    apply_updates_related_provider_mappings: false,
    updates_canonical_metadata: false,
    updates_hierarchy: false,
    writes_nfo: false,
    writes_library_files: false,
    ...overrides,
  }
}

function adminMetadataCandidateSubject(
  subjectKind: string,
  subjectKey: string,
  title: string,
) {
  return {
    provider: "bangumi",
    subject_kind: subjectKind,
    subject_key: subjectKey,
    title,
    release_year: 2026,
    locale: "zh-CN",
    raw_subject_payload: "provider secret response",
  }
}

function adminMetadataCandidateSummary(
  title: string,
  unsafeFields: Record<string, unknown> = {},
) {
  return {
    title,
    original_title: null,
    sort_title: null,
    release_date: "2026-06-01",
    runtime_minutes: null,
    description_present: true,
    tagline_present: false,
    genre_count: 1,
    tag_count: 1,
    rating_count: 1,
    image_count: 1,
    credit_count: 0,
    collection_count: 0,
    studio_count: 0,
    external_id_count: 1,
    ...unsafeFields,
  }
}

function adminGeneratedArtifactAcceptedProposal(artifactId = "artifact-bulk-accepted") {
  return {
    id: artifactId,
    kind: "metadata_suggestion",
    capability: "item_metadata_suggest",
    status: "accepted",
    target: {
      kind: "media_item",
      library_id: "library-a",
      item_id: "item-live",
      source_id: "source-live",
      local_path: "F:\\private\\source\\Movie.mkv",
      source_locator: "file:///mnt/private/source/Movie.mkv",
    },
    provenance: {
      provider_id: "provider-live",
      provider_name: "Live Automation Provider",
      job_id: "job-live",
      capability: "item_metadata_suggest",
      idempotency_key_fingerprint: "sha256:idempotency-live",
      prompt_fingerprint: "sha256:prompt-live",
      attempt_count: 2,
      artifact_created_at: "2026-05-29T01:00:00Z",
      raw_prompt: "unsafe prompt body",
      provider_raw_response: "provider secret response",
    },
    payload: {
      valid_json: true,
      shape: "object",
      payload_fingerprint: "sha256:payload-live",
      payload_bytes: 4096,
      object_field_count: 9,
      array_item_count: null,
      has_textual_values: true,
      has_explanation: true,
      confidence_milli: 910,
      raw_payload: {
        title: "unsafe generated payload title",
      },
    },
    readiness: {
      status: "accepted",
      actionable: true,
      reasons: ["accepted_generated_artifact"],
    },
    created_at: "2026-05-29T01:01:00Z",
    updated_at: "2026-05-29T01:10:00Z",
    accepted_at: "2026-05-29T01:10:00Z",
  }
}

function adminGeneratedArtifactReviewPlanResponse(
  artifactId = "artifact-live",
  decision: "accept" | "reject" = "accept",
) {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    plan: adminGeneratedArtifactAcceptancePlan(artifactId, decision),
  }
}

function adminGeneratedArtifactReviewResponse(
  artifactId = "artifact-live",
  decision: "accept" | "reject" = "accept",
) {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    artifact_id: artifactId,
    decision,
    artifact_status: decision === "accept" ? "accepted" : "rejected",
    accepted_at: decision === "accept" ? "2026-05-29T01:10:00Z" : null,
    idempotent_replay: true,
    plan: adminGeneratedArtifactAcceptancePlan(artifactId, decision),
  }
}

function adminGeneratedArtifactMetadataApplyPlanResponse(artifactId = "artifact-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    plan: adminGeneratedArtifactMetadataApplyPlan(artifactId),
  }
}

function adminGeneratedArtifactMetadataApplyResponse(artifactId = "artifact-live") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    outcome_id: "metadata-apply-outcome-live",
    artifact_id: artifactId,
    status: "applied",
    applied: true,
    changed: true,
    idempotent_replay: true,
    applied_source: "user",
    plan: adminGeneratedArtifactMetadataApplyPlan(artifactId),
  }
}

function adminGeneratedArtifactMetadataApplyRecoveryResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    summary: {
      returned_entry_count: 1,
      needs_repair_count: 1,
      needs_review_count: 0,
      replay_only_count: 0,
      resolved_count: 0,
    },
    entries: [
      {
        source: "apply_outcome",
        attention: "needs_repair",
        reason: "apply_outcome_failed",
        artifact_id: "artifact-live",
        outcome_id: "outcome-live",
        batch_id: null,
        batch_item_status: null,
        outcome_status: "failed",
        item_id: "item-live",
        plan: {
          ...adminGeneratedArtifactMetadataApplyPlan("artifact-live"),
          raw_prompt: "unsafe prompt body",
          idempotency_key: "unsafe-recovery-idempotency",
        },
        error_code: "target_stale",
        error_message: "target became stale before apply execution",
        created_at: "2026-06-02T12:00:00Z",
        updated_at: "2026-06-02T12:05:00Z",
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function adminGeneratedArtifactMetadataBulkApplyPlanResponse(
  artifactIds = ["artifact-bulk-accepted"],
) {
  const items = artifactIds.map((artifactId) => {
    if (artifactId.includes("missing")) {
      return {
        artifact_id: artifactId,
        status: "missing",
        executable: false,
        reasons: ["generated_artifact_not_found"],
        plan: null,
        raw_artifact_json: "unsafe raw artifact",
      }
    }

    return {
      artifact_id: artifactId,
      status: "planned",
      executable: true,
      reasons: ["accepted_generated_artifact"],
      plan: adminGeneratedArtifactMetadataApplyPlan(artifactId),
      raw_artifact_json: "unsafe raw artifact",
    }
  })
  const executableItems = items.filter((item) => item.executable)
  const missingItems = items.filter((item) => item.status === "missing")

  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    plan: {
      selection: {
        requested_artifact_count: artifactIds.length,
        selected_artifact_count: new Set(artifactIds).size,
        duplicate_artifact_count: artifactIds.length - new Set(artifactIds).size,
        max_artifact_count: 100,
      },
      summary: {
        planned_artifact_count: items.length - missingItems.length,
        missing_artifact_count: missingItems.length,
        ready_artifact_count: executableItems.length,
        blocked_artifact_count: 0,
        stale_artifact_count: 0,
        executable_artifact_count: executableItems.length,
        apply_field_count: executableItems.length,
        skipped_field_count: executableItems.length,
        noop_field_count: executableItems.length,
        apply_provider_mapping_count: executableItems.length,
        skipped_provider_mapping_count: 0,
        noop_provider_mapping_count: executableItems.length,
      },
      items,
    },
  }
}

function adminGeneratedArtifactMetadataBulkApplyBatchResponse(
  artifactIds = ["artifact-bulk-accepted"],
) {
  const plan = adminGeneratedArtifactMetadataBulkApplyPlanResponse(artifactIds).plan

  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    batch: {
      id: "bulk-batch-live",
      job_id: "bulk-job-live",
      status: "completed",
      selection: plan.selection,
      summary: plan.summary,
      execution_summary: {
        total_item_count: plan.items.length,
        pending_item_count: 0,
        skipped_item_count: plan.items.filter((item) => !item.executable).length,
        applied_item_count: plan.items.filter((item) => item.executable).length,
        noop_item_count: 0,
        stale_item_count: 0,
        failed_item_count: 0,
      },
      items: plan.items.map((item, index) => ({
        artifact_id: item.artifact_id,
        position: index,
        status: item.executable ? "applied" : "skipped",
        outcome_id: item.executable ? "metadata-apply-outcome-live" : null,
        error_code: item.executable ? null : "missing_artifact",
        error_message: item.executable ? null : "Generated artifact missing",
        plan_item: item,
        created_at: "2026-05-29T01:15:00Z",
        updated_at: "2026-05-29T01:16:00Z",
        idempotency_key: "unsafe-item-idempotency",
      })),
      created_at: "2026-05-29T01:15:00Z",
      updated_at: "2026-05-29T01:16:00Z",
      idempotency_key: "unsafe-bulk-idempotency-key",
    },
  }
}

function adminGeneratedArtifactAcceptancePlan(
  artifactId = "artifact-live",
  decision: "accept" | "reject" = "accept",
) {
  return {
    artifact_id: artifactId,
    decision,
    status: "ready",
    action: decision === "accept" ? "accept_generated_artifact" : "reject_generated_artifact",
    reasons: ["ready_for_review"],
    capability: "item_metadata_suggest",
    kind: "metadata_suggestion",
    target: {
      kind: "media_item",
      library_id: "library-a",
      item_id: "item-live",
      source_id: "source-live",
      local_path: "F:\\private\\source\\Movie.mkv",
      source_locator: "file:///mnt/private/source/Movie.mkv",
    },
    payload: {
      valid_json: true,
      shape: "object",
      payload_fingerprint: "sha256:payload-live",
      payload_bytes: 4096,
      object_field_count: 9,
      array_item_count: null,
      has_textual_values: true,
      has_explanation: true,
      confidence_milli: 910,
      raw_payload: {
        title: "unsafe generated payload title",
      },
    },
    readiness: {
      status: "ready",
      actionable: true,
      reasons: ["ready_for_review"],
    },
    boundary: {
      accepted_into_canonical_metadata: false,
      writes_sidecar: false,
      writes_library_files: false,
      applies_immediately: false,
      requires_metadata_authority_apply: decision === "accept",
    },
    raw_prompt: "unsafe prompt body",
    provider_raw_response: "provider secret response",
    artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
  }
}

function adminGeneratedArtifactMetadataApplyPlan(artifactId = "artifact-live") {
  return {
    artifact_id: artifactId,
    status: "ready",
    executable: true,
    reasons: ["accepted_generated_artifact"],
    target: {
      kind: "media_item",
      library_id: "library-a",
      item_id: "item-live",
      source_id: "source-live",
      local_path: "F:\\private\\source\\Movie.mkv",
      source_locator: "file:///mnt/private/source/Movie.mkv",
    },
    payload: {
      valid_json: true,
      shape: "object",
      payload_fingerprint: "sha256:payload-live",
      payload_bytes: 4096,
      object_field_count: 9,
      array_item_count: null,
      has_textual_values: true,
      has_explanation: true,
      confidence_milli: 910,
      raw_payload: {
        title: "unsafe generated payload title",
        secret: "provider-secret",
      },
    },
    fields: [
      {
        field: "title",
        action: "apply",
        reasons: ["incoming_differs"],
        current: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:current-title",
          value_bytes: 12,
          item_count: null,
          raw_value: "unsafe current title",
        },
        incoming: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:incoming-title",
          value_bytes: 16,
          item_count: null,
          raw_value: "unsafe generated payload title",
        },
      },
      {
        field: "overview",
        action: "skip",
        reasons: ["field_locked"],
        current: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:current-overview",
          value_bytes: 32,
          item_count: null,
        },
        incoming: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:incoming-overview",
          value_bytes: 64,
          item_count: null,
        },
      },
      {
        field: "genres",
        action: "noop",
        reasons: ["same_value"],
        current: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:genres",
          value_bytes: null,
          item_count: 2,
        },
        incoming: {
          present: true,
          empty: false,
          value_fingerprint: "sha256:genres",
          value_bytes: null,
          item_count: 2,
        },
      },
    ],
    provider_mappings: adminGeneratedArtifactProviderMappingPlans(),
    apply_field_count: 1,
    skipped_field_count: 1,
    noop_field_count: 1,
    apply_provider_mapping_count: 1,
    skipped_provider_mapping_count: 0,
    noop_provider_mapping_count: 1,
    raw_prompt: "unsafe prompt body",
    provider_raw_response: "provider secret response",
    artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
  }
}

function adminGeneratedArtifactProviderMappingPlans() {
  return [
    {
      subject: {
        provider: "tmdb",
        provider_name: "TMDB",
        subject_kind: "movie",
        subject_kind_name: "Movie",
        subject_key: "tmdb-123",
        title: "Live Movie",
        release_year: 2026,
        locale: "zh-CN",
        raw_subject_payload: "provider secret response",
      },
      action: "apply",
      reasons: ["incoming_provider_subject"],
      confidence_milli: 910,
      existing_mapping_status: null,
      raw_provider_mapping: "provider secret response",
    },
    {
      subject: {
        provider: "tmdb",
        provider_name: "TMDB",
        subject_kind: "collection",
        subject_kind_name: "Collection",
        subject_key: "tmdb-collection-9",
        title: "Live Collection",
        release_year: null,
        locale: "zh-CN",
      },
      action: "noop",
      reasons: ["existing_mapping_same_subject"],
      confidence_milli: 870,
      existing_mapping_status: "accepted",
    },
  ]
}

function publicUserPlaylist(overrides: Record<string, unknown> = {}) {
  return {
    id: "playlist-live",
    name: "Live Playlist",
    visibility: "private",
    item_count: 1,
    created_at: "2026-05-29T00:00:00Z",
    updated_at: "2026-05-29T01:00:00Z",
    version: 2,
    ...overrides,
  }
}

function publicPlaylistItem(itemId: string, title: string, position: number) {
  const baseItem = publicMediaItem()

  return {
    playlist_id: "playlist-live",
    item_id: itemId,
    position,
    added_at: "2026-05-29T01:00:00Z",
    item: publicMediaItem({
      id: itemId,
      metadata: {
        ...baseItem.metadata,
        original_title: title,
        title,
      },
    }),
    images: [],
  }
}

function publicMediaItem(overrides: Record<string, unknown> = {}) {
  return {
    id: "live-movie",
    kind: "movie",
    parent_id: null,
    metadata: {
      collections: [],
      credits: [],
      external_ids: [],
      genres: ["Sci-Fi"],
      original_title: "Live Original",
      overview: "A routed public API item.",
      ratings: [{ source: "tmdb", value: "8.1" }],
      release_date: "2026-01-02",
      runtime_minutes: 125,
      sort_title: null,
      studios: [],
      tagline: null,
      tags: [],
      title: "Live Movie",
    },
    ...overrides,
  }
}

function publicMediaSource(overrides: Record<string, unknown> = {}) {
  return {
    id: "source-live",
    item_id: "live-movie",
    library_id: "library-a",
    file_name: "Live Movie.mkv",
    fingerprint: null,
    size_bytes: 1024,
    ...overrides,
  }
}

function publicLibrarySource(overrides: Record<string, unknown> = {}) {
  return {
    source: publicMediaSource({
      library_id: "movies",
      ...overrides,
    }),
    item: publicMediaItem(),
  }
}

function publicLibrary(id: string) {
  return {
    id,
    name: "Live Library",
    roots: ["/media/live"],
    options: {
      domain: "video",
      preset: "movies",
      naming_strategy: "movie",
      scan: {
        max_depth: null,
        realtime_monitor: true,
      },
      metadata_profile: {
        country: null,
        image_providers: [],
        item_kinds: ["movie"],
        language: null,
        local_metadata_policy: "read_only",
        local_readers: [],
        metadata_providers: [],
        refresh_mode: "default",
        scan: {
          addon_scrape: true,
          addon_writeback: false,
          enabled: true,
        },
      },
    },
  }
}

function managementContextLinksResponse(overrides: Record<string, unknown> = {}) {
  return {
    context: {
      library_id: "library-a",
      item_id: "live-movie",
      source_id: "source-live",
      playback_session_id: null,
    },
    links: [managementContextLink()],
    ...overrides,
  }
}

function managementContextLink(overrides: Record<string, unknown> = {}) {
  return {
    action: "scan_library",
    disabled_reason: null,
    enabled: true,
    method: "POST",
    required_access: "library_manage",
    route_name: "library.scan",
    surface: "management",
    target: {
      library_id: "library-a",
      item_id: "live-movie",
      source_id: "source-live",
      playback_session_id: null,
    },
    ...overrides,
  }
}
