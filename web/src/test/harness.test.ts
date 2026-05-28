import { describe, expect, it } from "vitest"

describe("web test harness", () => {
  it("provides browser API shims needed by route and component tests", () => {
    expect(window.matchMedia("(prefers-color-scheme: dark)").matches).toBe(false)
    expect(new window.ResizeObserver(() => {}).disconnect).toBeTypeOf("function")
    expect(new window.IntersectionObserver(() => {}).disconnect).toBeTypeOf("function")
    expect(document.createElement("div").scrollIntoView).toBeTypeOf("function")
  })
})
