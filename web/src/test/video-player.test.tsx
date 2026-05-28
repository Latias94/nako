import { render } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"
import { VideoPlayer } from "@/src/features/media/video-player"

afterEach(() => {
  vi.restoreAllMocks()
})

describe("VideoPlayer subtitle track contract", () => {
  it("renders browser-ticket media and subtitle URLs as native video tracks", () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})

    const { container } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        sources={[
          {
            quality: "DIRECT",
            url: "http://nako.test/sources/source-1/stream?ticket=video-ticket",
            contentType: "video/x-matroska",
          },
        ]}
        subtitles={[
          {
            id: "subtitle-2",
            language: "en",
            srcLang: "en",
            url: "http://nako.test/sources/source-1/subtitles/2?ticket=subtitle-ticket",
            contentType: "application/x-subrip; charset=utf-8",
            default: true,
          },
        ]}
      />,
    )

    const video = container.querySelector("video")
    const source = container.querySelector("source")
    const track = container.querySelector("track")

    expect(video).toHaveAttribute("data-testid", "nako-video-player")
    expect(source).toHaveAttribute(
      "src",
      "http://nako.test/sources/source-1/stream?ticket=video-ticket",
    )
    expect(source).toHaveAttribute("type", "video/x-matroska")
    expect(track).toHaveAttribute(
      "src",
      "http://nako.test/sources/source-1/subtitles/2?ticket=subtitle-ticket",
    )
    expect(track).toHaveAttribute("srclang", "en")
    expect(track).toHaveAttribute("label", "en")
    expect(track?.getAttribute("src")).not.toContain("public-token")
  })
})
