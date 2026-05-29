import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"
import { CONNECTION_PROFILE_STORAGE_KEY } from "@/src/api/connection-profile"

interface RouteContract {
  path: string
  assert: () => Promise<void>
}

function renderRoute(path: string) {
  const router = createNakoRouter({
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  return render(
    <ThemeProvider defaultTheme="dark">
      <QueryProvider>
        <NakoRouter router={router} />
      </QueryProvider>
    </ThemeProvider>,
  )
}

const routeContracts: RouteContract[] = [
  {
    path: "/",
    assert: async () => {
      expect(await screen.findByText(/继续观看/, {}, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media",
    assert: async () => {
      expect(await screen.findByText(/继续观看/, {}, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/search?q=dune",
    assert: async () => {
      expect(await screen.findByDisplayValue("dune", {}, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/detail?id=1&type=movie",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "沙丘2" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/library?id=movies",
    assert: async () => {
      expect(await screen.findByRole("button", { name: /按 日期已添加 排序/ }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/library?id=movies&view=table&sort=title&order=asc&filter=unwatched",
    assert: async () => {
      expect(await screen.findByRole("button", { name: /按 标题 排序/ }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "仪表盘" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/libraries",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "媒体库管理" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/users",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "用户管理" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/tasks",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "计划任务" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/logs",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "系统日志" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/logs?q=database&levels=error,warn&sources=database&tab=errors&time=7d",
    assert: async () => {
      expect(await screen.findByDisplayValue("database", {}, { timeout: 5000 })).toBeInTheDocument()
      expect(await screen.findByText("最近7天", {}, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/settings",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "高级设置" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/notifications",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "通知中心" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/settings",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "设置" }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/setup",
    assert: async () => {
      expect(await screen.findByText(/Welcome to Nako/, {}, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/account",
    assert: async () => {
      expect(await screen.findByText(/谁在观看？/, {}, { timeout: 5000 })).toBeInTheDocument()
    },
  },
  {
    path: "/tv",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: /沙丘2/ }, { timeout: 5000 })).toBeInTheDocument()
    },
  },
]

describe("top-level route contracts", () => {
  it.each(routeContracts)("$path renders expected surface", async ({ path, assert }) => {
    renderRoute(path)

    await assert()
  })

  it("renders live scoped library items from the Public Client browse route", async () => {
    const originalFetch = globalThis.fetch
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = new URL(String(input))

      if (url.pathname === "/libraries/movies") {
        return jsonResponse({
          library: publicLibrary("movies"),
        })
      }

      if (url.pathname === "/libraries/movies/sources") {
        return jsonResponse({
          library: publicLibrary("movies"),
          page,
          sources: [],
        })
      }

      if (url.pathname === "/libraries/movies/items") {
        return jsonResponse({
          library: publicLibrary("movies"),
          page,
          items: [publicMediaItem()],
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

    try {
      renderRoute("/media/library?id=movies")

      expect(await screen.findByText("Live Movie", {}, { timeout: 5000 })).toBeInTheDocument()
      const calledTargets = fetcher.mock.calls.map(([input]) => {
        const url = new URL(String(input))
        return `${url.pathname}${url.search}`
      })

      expect(calledTargets).toContain(
        "/libraries/movies/items?limit=50&offset=0&sort=date_added&order=desc&watch_state=any",
      )
    } finally {
      vi.stubGlobal("fetch", originalFetch)
    }
  })
})

const page = {
  limit: 50,
  offset: 0,
  returned: 1,
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
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

function publicMediaItem() {
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
  }
}
