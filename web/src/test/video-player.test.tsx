import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"
import { VideoPlayer } from "@/src/features/media/video-player"

const hlsMock = vi.hoisted(() => ({
  instances: [] as Array<{
    attachMedia: ReturnType<typeof vi.fn>
    loadSource: ReturnType<typeof vi.fn>
    destroy: ReturnType<typeof vi.fn>
    on: ReturnType<typeof vi.fn>
  }>,
  isSupported: vi.fn(() => true),
}))

vi.mock("hls.js", () => {
  class MockHls {
    static Events = { ERROR: "hlsError" }
    static isSupported = hlsMock.isSupported

    attachMedia = vi.fn()
    loadSource = vi.fn()
    destroy = vi.fn()
    on = vi.fn()

    constructor() {
      hlsMock.instances.push(this)
    }
  }

  return { default: MockHls }
})

afterEach(() => {
  vi.restoreAllMocks()
  hlsMock.instances.length = 0
  hlsMock.isSupported.mockReset()
  hlsMock.isSupported.mockReturnValue(true)
})

describe("VideoPlayer subtitle track contract", () => {
  it("renders direct browser-ticket media and subtitle URLs as native video tracks", () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    const onPlaybackHeartbeat = vi.fn()

    const { container } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        playbackSessionId="playback-session-1"
        onPlaybackHeartbeat={onPlaybackHeartbeat}
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

  it("uses native HLS playback when the browser supports playlist media", async () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    vi.spyOn(HTMLMediaElement.prototype, "canPlayType").mockImplementation((contentType) =>
      String(contentType).toLowerCase().includes("mpegurl") ? "probably" : "",
    )

    const { container } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        sources={[
          {
            quality: "HLS",
            url: "http://nako.test/sources/source-1/hls/master.m3u8?ticket=hls-ticket",
            contentType: "application/vnd.apple.mpegurl",
          },
        ]}
        subtitles={[]}
      />,
    )

    await waitFor(() =>
      expect(container.querySelector("source")).toHaveAttribute(
        "src",
        "http://nako.test/sources/source-1/hls/master.m3u8?ticket=hls-ticket",
      ),
    )
    expect(hlsMock.instances).toHaveLength(0)
  })

  it("lazy-loads HLS playback without rendering playlist tickets into source markup", async () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    vi.spyOn(HTMLMediaElement.prototype, "canPlayType").mockReturnValue("")

    const { container } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        sources={[
          {
            quality: "HLS",
            url: "http://nako.test/sources/source-1/hls/master.m3u8?ticket=hls-ticket",
            contentType: "application/vnd.apple.mpegurl",
          },
        ]}
        subtitles={[]}
      />,
    )

    await waitFor(() => {
      expect(hlsMock.instances).toHaveLength(1)
      expect(hlsMock.instances[0].attachMedia).toHaveBeenCalledWith(
        screen.getByTestId("nako-video-player"),
      )
      expect(hlsMock.instances[0].loadSource).toHaveBeenCalledWith(
        "http://nako.test/sources/source-1/hls/master.m3u8?ticket=hls-ticket",
      )
    })
    expect(container.querySelector("source")).toBeNull()
    expect(container.innerHTML).not.toContain("hls-ticket")
  })

  it("shows a redacted HLS fallback when Media Source playback is unavailable", async () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    vi.spyOn(HTMLMediaElement.prototype, "canPlayType").mockReturnValue("")
    hlsMock.isSupported.mockReturnValue(false)

    render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        sources={[
          {
            quality: "HLS",
            url: "http://nako.test/sources/source-1/hls/master.m3u8?ticket=hls-ticket",
            contentType: "application/vnd.apple.mpegurl",
          },
        ]}
        subtitles={[]}
      />,
    )

    expect(await screen.findByText("HLS 播放不可用")).toBeInTheDocument()
    expect(document.body.textContent).not.toContain("hls-ticket")
    expect(document.body.textContent).not.toContain("public-token")
  })

  it("heartbeats through the playback session id instead of media URLs", async () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    vi.spyOn(HTMLMediaElement.prototype, "play").mockResolvedValue(undefined)
    const onPlaybackHeartbeat = vi.fn()
    const { container } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        playbackSessionId="playback-session-1"
        onPlaybackHeartbeat={onPlaybackHeartbeat}
        sources={[
          {
            quality: "DIRECT",
            url: "http://nako.test/sources/source-1/stream?ticket=video-ticket",
            contentType: "video/x-matroska",
          },
        ]}
        subtitles={[]}
      />,
    )
    const video = container.querySelector("video")
    expect(video).not.toBeNull()
    Object.defineProperty(video!, "currentTime", { configurable: true, value: 12 })
    Object.defineProperty(video!, "duration", { configurable: true, value: 120 })

    fireEvent.playing(video!)

    await waitFor(() =>
      expect(onPlaybackHeartbeat).toHaveBeenCalledWith("playback-session-1", {
        state: "active",
        position_ms: 12000,
        duration_ms: 120000,
      }),
    )
    expect(onPlaybackHeartbeat.mock.calls[0][0]).not.toContain("ticket=")
  })

  it("cancels the playback session id on player teardown without passing media URLs", () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    const onPlaybackCancel = vi.fn()

    const { unmount } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        playbackSessionId="playback-session-1"
        onPlaybackCancel={onPlaybackCancel}
        sources={[
          {
            quality: "DIRECT",
            url: "http://nako.test/sources/source-1/stream?ticket=video-ticket",
            contentType: "video/x-matroska",
          },
        ]}
        subtitles={[]}
      />,
    )

    unmount()

    expect(onPlaybackCancel).toHaveBeenCalledWith("playback-session-1")
    expect(onPlaybackCancel.mock.calls[0][0]).not.toContain("ticket=")
  })

  it("renders diagnostic actions when no playable source is available", () => {
    render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        sources={[]}
        subtitles={[]}
        diagnosticActions={<a href="/admin/transcoding?source_id=source-live">播放诊断</a>}
      />,
    )

    expect(screen.getByRole("link", { name: "播放诊断" })).toHaveAttribute(
      "href",
      "/admin/transcoding?source_id=source-live",
    )
    expect(document.body.textContent).not.toContain("ticket=")
  })
})
