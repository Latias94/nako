import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"
import { VideoPlayer } from "@/src/features/media/video-player"

afterEach(() => {
  vi.restoreAllMocks()
})

describe("VideoPlayer subtitle track contract", () => {
  it("renders browser-ticket media and subtitle URLs as native video tracks", () => {
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
