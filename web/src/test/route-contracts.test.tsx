import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"

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
    path: "/admin",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "仪表盘" }, { timeout: 5000 })).toBeInTheDocument()
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
})
