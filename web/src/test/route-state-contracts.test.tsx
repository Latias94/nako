import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"

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

describe("route state contracts", () => {
  it("writes Admin log search state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/logs")

    const input = await screen.findByPlaceholderText("搜索日志内容...", {}, { timeout: 5000 })
    await user.type(input, "database")

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ q: "database" })
    })
  })

  it("writes Media search submits to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/search")

    const input = await screen.findByPlaceholderText("搜索电影、剧集、演员...", {}, { timeout: 5000 })
    await user.type(input, "dune")
    const searchButtons = screen.getAllByRole("button", { name: "搜索" })
    await user.click(searchButtons[searchButtons.length - 1])

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ q: "dune" })
    })
  })
})
