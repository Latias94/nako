import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { BrowserPlaybackTicketResponse, ItemsResponse } from "@nako/sdk";
import { afterEach, describe, expect, it, vi } from "vitest";

import { App } from "../../App";
import type { MediaDataSourceFactory, MediaWebDataSource } from "./mediaDataSource";
import { createFixtureMediaDataSource } from "./mediaDataSource";

afterEach(() => {
  delete (globalThis as typeof globalThis & { Hls?: unknown }).Hls;
  vi.restoreAllMocks();
  window.history.pushState(null, "", "/");
});

describe("Media Web surface", () => {
  it("renders the Media connect surface under the shared app shell", async () => {
    window.history.pushState(null, "", "/media");

    render(<App dataSource={emptyAdminDataSource()} />);

    expect(await screen.findByRole("heading", { name: "Enter a Nako server" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Admin/ })).toBeInTheDocument();
    expect(screen.getByLabelText("Server URL")).toBeInTheDocument();
    expect(screen.queryByText("Admin API")).not.toBeInTheDocument();
  });

  it("connects through the live Public Client data source without rendering the token", async () => {
    window.history.pushState(null, "", "/media/libraries");
    const dataSource = createFixtureMediaDataSource();
    const checkConnection = vi.fn(async () => undefined);
    const factory = vi.fn((connection) => ({
      ...dataSource,
      checkConnection,
      label: connection.mode === "live" ? "Live Public Client API" : dataSource.label,
      source: connection.mode === "live" ? "live" : dataSource.source,
    }) as MediaWebDataSource) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App dataSource={emptyAdminDataSource()} mediaDataSourceFactory={factory} />,
    );

    fireEvent.change(await screen.findByLabelText("Server URL"), {
      target: { value: "http://nako.test" },
    });
    fireEvent.change(screen.getByLabelText("Access token"), {
      target: { value: "secret-token" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(await screen.findByRole("heading", { name: "Media Libraries" })).toBeInTheDocument();
    expect(screen.getAllByText("Live Public Client API").length).toBeGreaterThan(0);
    expect(await screen.findByText("Anime Vault")).toBeInTheDocument();
    expect(factory).toHaveBeenCalledWith({
      baseUrl: "http://nako.test",
      bearerToken: "secret-token",
      mode: "live",
    });
    expect(container.textContent).not.toContain("secret-token");
  });

  it("keeps fixture mode visible for development data", async () => {
    window.history.pushState(null, "", "/media/libraries");

    render(<App dataSource={emptyAdminDataSource()} />);

    fireEvent.click(await screen.findByRole("button", { name: "Use fixture demo" }));

    expect(await screen.findByRole("heading", { name: "Media Libraries" })).toBeInTheDocument();
    expect(screen.getAllByText("Fixture mode").length).toBeGreaterThan(0);
    expect(await screen.findByText("Films")).toBeInTheDocument();
  });

  it("renders Recently Added item cards from the home item list", async () => {
    window.history.pushState(null, "", "/media");
    const dataSource = createFixtureMediaDataSource();
    const listItems = vi.fn<MediaWebDataSource["listItems"]>(async (page) => {
      const result = await dataSource.listItems(page);
      return {
        ...result,
        value: {
          ...result.value,
          items: result.value.items.map((item) => ({
            ...item,
            raw_locator: "/sources/source-episode-1?ticket=nako_bpt_fixture",
            source: {
              fingerprint: "fingerprint-raw-backend",
              local_path: "F:\\media\\library\\Pilot.mkv",
              root: "redacted-root",
            },
            stream_url:
              "https://fixture.nako.test/sources/source-episode-1/stream?ticket=nako_bpt_fixture",
          })),
        },
      };
    });
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          listItems,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const recentlyAdded = await findRecentlyAddedSection();

    expect(within(recentlyAdded).getByText("2 shown")).toBeInTheDocument();
    expect(within(recentlyAdded).getByRole("link", { name: /Pilot/ })).toHaveAttribute(
      "href",
      "/media/items/item-episode-1",
    );
    expect(
      within(recentlyAdded).getByRole("link", { name: /After the Rain/ }),
    ).toHaveAttribute("href", "/media/items/item-film-1");
    expect(screen.queryByRole("heading", { name: "Media Items" })).not.toBeInTheDocument();
    await waitFor(() => {
      expect(listItems).toHaveBeenCalledWith({ limit: 8, offset: 0 });
    });
    expect(container.textContent).not.toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("fingerprint");
    expect(container.textContent).not.toContain("F:\\media");
    expect(container.textContent).not.toContain("redacted-root");
  });

  it("shows an empty state when Recently Added has no items", async () => {
    window.history.pushState(null, "", "/media");
    const dataSource = createFixtureMediaDataSource();
    const emptyItems: ItemsResponse = {
      items: [],
      page: {
        limit: 8,
        offset: 0,
        returned: 0,
      },
    };
    const listItems = vi.fn(async () => ({
      source: "fixture" as const,
      value: emptyItems,
    }));
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          listItems,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const recentlyAdded = await findRecentlyAddedSection();

    expect(await within(recentlyAdded).findByText("No recently added media")).toBeInTheDocument();
    expect(within(recentlyAdded).getByText("0 shown")).toBeInTheDocument();
    expect(screen.getByText("Continue Watching")).toBeInTheDocument();
    await waitFor(() => {
      expect(listItems).toHaveBeenCalledWith({ limit: 8, offset: 0 });
    });
  });

  it("keeps Continue Watching visible when Recently Added fails safely", async () => {
    window.history.pushState(null, "", "/media");
    const dataSource = createFixtureMediaDataSource();
    const rawListError =
      "HTTP 500 for /sources/source-episode-1?ticket=nako_bpt_secret with bearer secret-token and fingerprint raw-backend";
    const listItems = vi.fn(async () => {
      throw new Error(rawListError);
    });
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          listItems,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const recentlyAdded = await findRecentlyAddedSection();

    expect(screen.getByText("Continue Watching")).toBeInTheDocument();
    expect(await screen.findByText("38% complete - resume at 9 min")).toBeInTheDocument();
    expect(
      await within(recentlyAdded).findByText("Recently added media could not be loaded."),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(listItems).toHaveBeenCalledWith({ limit: 8, offset: 0 });
    });
    expect(container.textContent).not.toContain(rawListError);
    expect(container.textContent).not.toContain("HTTP 500");
    expect(container.textContent).not.toContain("raw-backend");
    expect(container.textContent).not.toContain("nako_bpt_secret");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("secret-token");
    expect(container.textContent).not.toContain("fingerprint");
  });

  it("redacts a Recently Added data-source error result", async () => {
    window.history.pushState(null, "", "/media");
    const dataSource = createFixtureMediaDataSource();
    const rawListError =
      "HTTP 500 for /sources/source-episode-1?ticket=nako_bpt_secret with bearer secret-token and fingerprint raw-backend";
    const emptyItems: ItemsResponse = {
      items: [],
      page: {
        limit: 8,
        offset: 0,
        returned: 0,
      },
    };
    const listItems = vi.fn(async () => ({
      error: rawListError,
      source: "fixture" as const,
      value: emptyItems,
    }));
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          listItems,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const recentlyAdded = await findRecentlyAddedSection();

    expect(
      await within(recentlyAdded).findByText("Recently added media could not be loaded."),
    ).toBeInTheDocument();
    expect(within(recentlyAdded).queryByText("No recently added media")).not.toBeInTheDocument();
    expect(screen.getByText("Continue Watching")).toBeInTheDocument();
    await waitFor(() => {
      expect(listItems).toHaveBeenCalledWith({ limit: 8, offset: 0 });
    });
    expect(container.textContent).not.toContain(rawListError);
    expect(container.textContent).not.toContain("HTTP 500");
    expect(container.textContent).not.toContain("raw-backend");
    expect(container.textContent).not.toContain("nako_bpt_secret");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("secret-token");
    expect(container.textContent).not.toContain("fingerprint");
  });

  it("links Continue Watching entries to the watch route with source continuity", async () => {
    window.history.pushState(null, "", "/media");

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Watch next" })).toBeInTheDocument();
    expect(screen.getByText("38% complete - resume at 9 min")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Resume" })).toHaveAttribute(
      "href",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );
    expect(container.textContent).not.toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("fingerprint");
  });

  it("clears Continue Watching progress from the home row action", async () => {
    window.history.pushState(null, "", "/media");
    const dataSource = createFixtureMediaDataSource();
    const setUserWatchedState = vi.fn(dataSource.setUserWatchedState);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          setUserWatchedState,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Watch next" })).toBeInTheDocument();
    expect(screen.getByText("38% complete - resume at 9 min")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Start over" }));

    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenCalledWith("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 0,
        source_id: "source-episode-1",
        watched: false,
      });
    });
    expect(await screen.findByText("No active playback state")).toBeInTheDocument();
    expect(screen.queryByText("38% complete - resume at 9 min")).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Resume" })).not.toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("fingerprint");
  });

  it("keeps Continue Watching visible and retryable when Start over fails", async () => {
    window.history.pushState(null, "", "/media");
    const dataSource = createFixtureMediaDataSource();
    const rawMutationError =
      "HTTP 500 for /sources/source-episode-1?ticket=nako_bpt_secret with bearer secret-token and fingerprint raw-backend";
    const safeMutationError = "Continue Watching progress could not be cleared.";
    const successfulRetry: {
      resolve?: (
        result: Awaited<ReturnType<MediaWebDataSource["setUserWatchedState"]>>,
      ) => void;
    } = {};
    const setUserWatchedState = vi.fn(dataSource.setUserWatchedState);
    setUserWatchedState.mockRejectedValueOnce(new Error(rawMutationError));
    setUserWatchedState.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          successfulRetry.resolve = resolve;
        }),
    );
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          setUserWatchedState,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Watch next" })).toBeInTheDocument();
    expect(screen.getByText("38% complete - resume at 9 min")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Start over" }));

    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByText(safeMutationError)).toBeInTheDocument();
    expect(screen.getByText("38% complete - resume at 9 min")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Resume" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start over" })).not.toBeDisabled();
    expect(container.textContent).not.toContain(rawMutationError);
    expect(container.textContent).not.toContain("HTTP 500");
    expect(container.textContent).not.toContain("backend");
    expect(container.textContent).not.toContain("raw-backend");
    expect(container.textContent).not.toContain("nako_bpt_secret");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("secret-token");
    expect(container.textContent).not.toContain("fingerprint");

    fireEvent.click(screen.getByRole("button", { name: "Start over" }));

    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenCalledTimes(2);
    });
    await waitFor(() => {
      expect(screen.queryByText(safeMutationError)).not.toBeInTheDocument();
    });
    expect(screen.getByText("38% complete - resume at 9 min")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start over" })).toBeDisabled();

    const resolveRetry = successfulRetry.resolve;
    if (!resolveRetry) {
      throw new Error("Missing retry resolver");
    }
    resolveRetry(
      await dataSource.setUserWatchedState("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 0,
        source_id: "source-episode-1",
        watched: false,
      }),
    );
    expect(await screen.findByText("No active playback state")).toBeInTheDocument();
    expect(screen.queryByText("38% complete - resume at 9 min")).not.toBeInTheDocument();
    expect(screen.queryByText(safeMutationError)).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Resume" })).not.toBeInTheDocument();
    expect(setUserWatchedState).toHaveBeenNthCalledWith(1, "item-episode-1", {
      duration_ms: 1440000,
      position_ms: 0,
      source_id: "source-episode-1",
      watched: false,
    });
    expect(setUserWatchedState).toHaveBeenNthCalledWith(2, "item-episode-1", {
      duration_ms: 1440000,
      position_ms: 0,
      source_id: "source-episode-1",
      watched: false,
    });
  });

  it("renders a Media Library detail route from Public Client fixture data", async () => {
    window.history.pushState(
      null,
      "",
      "/media/libraries/library-anime?limit=1&offset=0",
    );

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Anime Vault" })).toBeInTheDocument();
    expect(screen.getByText("anime")).toBeInTheDocument();
    expect(screen.getByText("Library sources")).toBeInTheDocument();
    expect(screen.getByText("Pilot.mkv")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Pilot" })).toHaveAttribute(
      "href",
      "/media/items/item-episode-1",
    );
    expect(container.textContent).not.toContain("redacted-root");
    expect(container.textContent).not.toContain("fingerprint");
  });

  it("keeps Media search state URL-owned", async () => {
    window.history.pushState(null, "", "/media/search?q=rain&limit=5&offset=10");
    const dataSource = createFixtureMediaDataSource();
    const searchItems = vi.fn(dataSource.searchItems);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          searchItems,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const input = await screen.findByLabelText("Search media");
    expect(input).toHaveValue("rain");
    await waitFor(() => {
      expect(searchItems).toHaveBeenCalledWith({
        facet: undefined,
        limit: 5,
        offset: 10,
        q: "rain",
      });
    });

    fireEvent.change(input, { target: { value: "pilot" } });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));

    await waitFor(() => {
      expect(window.location.pathname).toBe("/media/search");
      expect(window.location.search).toContain("q=pilot");
      expect(window.location.search).toContain("limit=5");
      expect(window.location.search).toContain("offset=0");
    });
  });

  it("renders a Media Item detail route without unsafe source internals", async () => {
    window.history.pushState(null, "", "/media/items/item-episode-1");

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Pilot" })).toBeInTheDocument();
    expect(screen.getByText("A carefully kept local episode.")).toBeInTheDocument();
    expect(screen.getByText("Available sources")).toBeInTheDocument();
    expect(screen.getAllByText("Pilot.mkv").length).toBeGreaterThan(0);
    expect(container.textContent).not.toContain("raw source locator");
    expect(container.textContent).not.toContain("fingerprint");
    expect(container.textContent).not.toContain("redacted-root");
  });

  it("renders source selection and playback decision preview without stream URLs", async () => {
    window.history.pushState(
      null,
      "",
      "/media/items/item-episode-1?source_id=source-episode-1",
    );

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Pilot" })).toBeInTheDocument();
    expect(screen.getByText("Source versions")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Pilot\.mkv/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Pilot\.alt\.mp4/ })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Watch" })).toHaveAttribute(
      "href",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );
    expect(screen.getByText("Playback decision")).toBeInTheDocument();
    expect(screen.getAllByText("direct_play").length).toBeGreaterThan(0);
    expect(screen.getByText("Continue from 9 min")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Resume" })).toHaveAttribute(
      "href",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("secret-token");
  });

  it("keeps item detail Resume linked to the saved playback source", async () => {
    window.history.pushState(
      null,
      "",
      "/media/items/item-episode-1?source_id=source-episode-1-alt",
    );

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Pilot" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Watch" })).toHaveAttribute(
      "href",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    expect(screen.getByRole("link", { name: "Resume" })).toHaveAttribute(
      "href",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );
  });

  it("keeps selected source in the URL and writes watched state through Public Client data", async () => {
    window.history.pushState(null, "", "/media/items/item-episode-1");
    const dataSource = createFixtureMediaDataSource();
    const getPlaybackDecision = vi.fn(dataSource.getPlaybackDecision);
    const setUserWatchedState = vi.fn(dataSource.setUserWatchedState);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          getPlaybackDecision,
          setUserWatchedState,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: /Pilot\.alt\.mp4/ }));

    await waitFor(() => {
      expect(window.location.search).toContain("source_id=source-episode-1-alt");
      expect(getPlaybackDecision).toHaveBeenLastCalledWith(
        "source-episode-1-alt",
        expect.objectContaining({
          container: expect.arrayContaining(["mp4"]),
          direct_play: true,
          hls_variant_policy: "single_variant",
          video_codec: expect.arrayContaining(["h264"]),
        }),
      );
    });

    fireEvent.click(screen.getByRole("button", { name: "Mark watched" }));

    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenCalledWith("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 1440000,
        source_id: "source-episode-1-alt",
        watched: true,
      });
    });

    const markUnwatched = screen.getByRole("button", { name: "Mark unwatched" });
    await waitFor(() => {
      expect(markUnwatched).not.toBeDisabled();
    });
    fireEvent.click(markUnwatched);

    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenLastCalledWith(
        "item-episode-1",
        expect.objectContaining({
          duration_ms: 1440000,
          source_id: "source-episode-1-alt",
          watched: false,
        }),
      );
    });
  });

  it("clears item detail playback progress with Start over", async () => {
    window.history.pushState(
      null,
      "",
      "/media/items/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const setUserWatchedState = vi.fn(dataSource.setUserWatchedState);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          setUserWatchedState,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    expect(await screen.findByText("Continue from 9 min")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Start over" }));

    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenCalledWith("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 0,
        source_id: "source-episode-1-alt",
        watched: false,
      });
    });
    expect(await screen.findByText("Start from beginning")).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Resume" })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("link", { name: "Home" }));

    expect(await screen.findByRole("heading", { name: "Watch next" })).toBeInTheDocument();
    expect(screen.getByText("No active playback state")).toBeInTheDocument();
  });

  it("renders a ticketed browser player without exposing transport secrets as text", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );
    const dataSource = createFixtureMediaDataSource();
    const getPlaybackDecision = vi.fn(dataSource.getPlaybackDecision);
    const createBrowserPlaybackTicket = vi.fn(dataSource.createBrowserPlaybackTicket);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          createBrowserPlaybackTicket,
          getPlaybackDecision,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Pilot" })).toBeInTheDocument();
    expect(screen.getByText("Player")).toBeInTheDocument();
    expect(screen.getByText("Resume from 9 min")).toBeInTheDocument();
    expect(screen.getByText("38% complete - current source")).toBeInTheDocument();
    const player = await screen.findByLabelText("Pilot player");
    expect(player.tagName).toBe("VIDEO");
    expect(player).toHaveAttribute("controls");
    expect(screen.getByRole("button", { name: /Pilot\.mkv/ })).toBeInTheDocument();
    expect(screen.getAllByText("direct_play").length).toBeGreaterThan(0);
    await waitFor(() => {
      expect(getPlaybackDecision).toHaveBeenCalledWith(
        "source-episode-1",
        expect.objectContaining({
          container: expect.arrayContaining(["mp4"]),
          direct_play: true,
          hls_variant_policy: "single_variant",
          video_codec: expect.arrayContaining(["h264"]),
        }),
      );
    });
    await waitFor(() => {
      expect(createBrowserPlaybackTicket).toHaveBeenCalledWith(
        "source-episode-1",
        expect.objectContaining({
          capabilities: expect.objectContaining({
            container: expect.arrayContaining(["mp4"]),
            direct_play: true,
            output_container: undefined,
            video_codec: expect.arrayContaining(["h264"]),
          }),
          mode: "direct",
        }),
      );
    });
    expect((player as HTMLVideoElement).getAttribute("src")).toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("Bearer");
  });

  it("auto-resumes the browser player after metadata loads for the saved source", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );
    const dataSource = createFixtureMediaDataSource();
    const updateUserPlaybackProgress = vi.fn(dataSource.updateUserPlaybackProgress);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          updateUserPlaybackProgress,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    setMediaTiming(player, 0, 1440);
    fireEvent.loadedMetadata(player);

    expect(player).toHaveProperty("currentTime", 547.2);
    expect(updateUserPlaybackProgress).not.toHaveBeenCalled();

    setMediaTiming(player, 12, 1440);
    fireEvent.loadedMetadata(player);

    expect(player).toHaveProperty("currentTime", 12);
  });

  it("does not auto-resume when the saved source differs from the selected source", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    setMediaTiming(player, 0, 1440);
    fireEvent.loadedMetadata(player);

    expect(player).toHaveProperty("currentTime", 0);
    expect(screen.getByText("38% complete - different saved source")).toBeInTheDocument();
  });

  it("lets the user start over before metadata auto-resume runs", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1",
    );

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    setMediaTiming(player, 321, 1440);
    const playerPanel = screen.getByRole("region", { name: "Player" });
    fireEvent.click(within(playerPanel).getByRole("button", { name: "Start over" }));
    expect(player).toHaveProperty("currentTime", 0);

    fireEvent.loadedMetadata(player);

    expect(player).toHaveProperty("currentTime", 0);
  });

  it("shows a safe ticket retry state without exposing ticket issuance internals", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const createBrowserPlaybackTicket = vi
      .fn<MediaWebDataSource["createBrowserPlaybackTicket"]>()
      .mockRejectedValueOnce(
        new Error("ticket failure for /sources/source-episode-1-alt?ticket=nako_bpt_secret"),
      )
      .mockImplementation(dataSource.createBrowserPlaybackTicket);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          createBrowserPlaybackTicket,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    expect(await screen.findByText("Playback ticket could not be issued. Request a fresh ticket and try again.")).toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_secret");
    expect(container.textContent).not.toContain("/sources/");

    fireEvent.click(screen.getByRole("button", { name: "Retry ticket" }));

    expect(await screen.findByLabelText("Pilot player")).toBeInTheDocument();
    expect(createBrowserPlaybackTicket).toHaveBeenCalledTimes(2);
    expect(container.textContent).not.toContain("nako_bpt_fixture");
  });

  it("can recover from a failed browser path by trying the next ticket URL", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const createBrowserPlaybackTicket = vi.fn(async () => ({
      source: "fixture" as const,
      value: multiplePathTicket,
    })) satisfies MediaWebDataSource["createBrowserPlaybackTicket"];
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          createBrowserPlaybackTicket,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    expect(player).toHaveAttribute("src", expect.stringContaining("path=primary"));

    fireEvent.error(player);

    expect(
      await screen.findByText("Playback failed before the browser could start the stream."),
    ).toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_primary");
    expect(container.textContent).not.toContain("nako_bpt_secondary");

    fireEvent.click(screen.getByRole("button", { name: "Try next path" }));

    const recoveredPlayer = await screen.findByLabelText("Pilot player");
    expect(recoveredPlayer).toHaveAttribute("src", expect.stringContaining("path=secondary"));
    expect(
      screen.queryByText("Playback failed before the browser could start the stream."),
    ).not.toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_secondary");
  });

  it("can leave an unsupported HLS playlist for the next playable ticket URL", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    vi.spyOn(HTMLMediaElement.prototype, "canPlayType").mockReturnValue("");
    const dataSource = createFixtureMediaDataSource();
    const createBrowserPlaybackTicket = vi.fn(async () => ({
      source: "fixture" as const,
      value: hlsThenDirectTicket,
    })) satisfies MediaWebDataSource["createBrowserPlaybackTicket"];
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          createBrowserPlaybackTicket,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    expect(
      await screen.findByText(
        "This browser cannot open the HLS playlist without a compatible playback adapter.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Try next path" })).toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_hls_primary");

    fireEvent.click(screen.getByRole("button", { name: "Try next path" }));

    const player = await screen.findByLabelText("Pilot player");
    expect(player).toHaveAttribute("src", expect.stringContaining("path=mp4-fallback"));
    expect(
      screen.queryByText(
        "This browser cannot open the HLS playlist without a compatible playback adapter.",
      ),
    ).not.toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_mp4_fallback");
  });

  it("starts a new ticket on its first candidate after source changes", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const createBrowserPlaybackTicket = vi.fn(async (sourceId: string) => ({
      source: "fixture" as const,
      value: multiplePathTicketForSource(sourceId),
    })) satisfies MediaWebDataSource["createBrowserPlaybackTicket"];
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          createBrowserPlaybackTicket,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    expect(player).toHaveAttribute(
      "src",
      expect.stringContaining("source-episode-1-alt-primary"),
    );
    fireEvent.error(player);
    fireEvent.click(await screen.findByRole("button", { name: "Try next path" }));
    expect(await screen.findByLabelText("Pilot player")).toHaveAttribute(
      "src",
      expect.stringContaining("source-episode-1-alt-secondary"),
    );

    fireEvent.click(screen.getByRole("button", { name: /Pilot\.mkv/ }));

    await waitFor(() => {
      expect(createBrowserPlaybackTicket).toHaveBeenLastCalledWith(
        "source-episode-1",
        expect.anything(),
      );
    });
    expect(await screen.findByLabelText("Pilot player")).toHaveAttribute(
      "src",
      expect.stringContaining("source-episode-1-primary"),
    );
  });

  it("uses an available hls.js adapter for playlist tickets without rendering playlist URLs", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    vi.spyOn(HTMLMediaElement.prototype, "canPlayType").mockReturnValue("");
    const loadSource = vi.fn();
    const attachMedia = vi.fn();
    const destroy = vi.fn();
    class TestHls {
      static Events = { ERROR: "hls-error" };
      static isSupported() {
        return true;
      }
      attachMedia = attachMedia;
      destroy = destroy;
      loadSource = loadSource;
      on = vi.fn();
    }
    (globalThis as typeof globalThis & { Hls?: unknown }).Hls = TestHls;
    const dataSource = createFixtureMediaDataSource();
    const createBrowserPlaybackTicket = vi.fn(async () => ({
      source: "fixture" as const,
      value: hlsPlaylistTicket,
    })) satisfies MediaWebDataSource["createBrowserPlaybackTicket"];
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          createBrowserPlaybackTicket,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    await waitFor(() => {
      expect(loadSource).toHaveBeenCalledWith(expect.stringContaining("nako_bpt_hls"));
      expect(attachMedia).toHaveBeenCalledWith(player);
    });
    expect(player).not.toHaveAttribute("src");
    expect(screen.getByText("hls.js")).toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_hls");
    expect(container.textContent).not.toContain("/stream/hls/");
  });

  it("shows a safe retry state when the browser player reports an error", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    fireEvent.error(player);

    expect(
      await screen.findByText("Playback failed before the browser could start the stream."),
    ).toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("/sources/");

    fireEvent.click(screen.getByRole("button", { name: "Retry playback" }));

    expect(
      screen.queryByText("Playback failed before the browser could start the stream."),
    ).not.toBeInTheDocument();
    expect(await screen.findByLabelText("Pilot player")).toBeInTheDocument();
    expect(container.textContent).not.toContain("nako_bpt_fixture");
  });

  it("writes throttled playback progress only after browser playback starts", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const updateUserPlaybackProgress = vi.fn(dataSource.updateUserPlaybackProgress);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          updateUserPlaybackProgress,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    setMediaTiming(player, 45, 1440);
    fireEvent.timeUpdate(player);
    expect(updateUserPlaybackProgress).not.toHaveBeenCalled();

    fireEvent.play(player);
    setMediaTiming(player, 12, 1440);
    fireEvent.timeUpdate(player);
    expect(updateUserPlaybackProgress).not.toHaveBeenCalled();

    setMediaTiming(player, 31, 1440);
    fireEvent.timeUpdate(player);
    await waitFor(() => {
      expect(updateUserPlaybackProgress).toHaveBeenCalledWith("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 31000,
        source_id: "source-episode-1-alt",
      });
    });

    setMediaTiming(player, 45, 1440);
    fireEvent.timeUpdate(player);
    expect(updateUserPlaybackProgress).toHaveBeenCalledTimes(1);

    setMediaTiming(player, 61, 1440);
    fireEvent.timeUpdate(player);
    await waitFor(() => {
      expect(updateUserPlaybackProgress).toHaveBeenCalledTimes(2);
    });
    expect(updateUserPlaybackProgress).toHaveBeenLastCalledWith("item-episode-1", {
      duration_ms: 1440000,
      position_ms: 61000,
      source_id: "source-episode-1-alt",
    });
  });

  it("flushes progress on pause and marks the source watched on ended", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const updateUserPlaybackProgress = vi.fn(dataSource.updateUserPlaybackProgress);
    const setUserWatchedState = vi.fn(dataSource.setUserWatchedState);
    const factory = vi.fn(
      () =>
        ({
          ...dataSource,
          setUserWatchedState,
          updateUserPlaybackProgress,
        }) as MediaWebDataSource,
    ) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    fireEvent.pause(player);
    expect(updateUserPlaybackProgress).not.toHaveBeenCalled();
    expect(setUserWatchedState).not.toHaveBeenCalled();

    fireEvent.play(player);
    setMediaTiming(player, 10, 1440);
    fireEvent.pause(player);
    await waitFor(() => {
      expect(updateUserPlaybackProgress).toHaveBeenCalledWith("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 10000,
        source_id: "source-episode-1-alt",
      });
    });

    setMediaTiming(player, 1439, 1440);
    fireEvent.ended(player);
    await waitFor(() => {
      expect(setUserWatchedState).toHaveBeenCalledWith("item-episode-1", {
        duration_ms: 1440000,
        position_ms: 1440000,
        source_id: "source-episode-1-alt",
        watched: true,
      });
    });
  });

  it("refreshes Continue Watching from fixture progress after returning home", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const factory = vi.fn(() => dataSource) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    fireEvent.play(player);
    setMediaTiming(player, 61, 1440);
    fireEvent.timeUpdate(player);

    await waitFor(() => {
      expect(screen.getByText("Resume from 1 min")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("link", { name: "Home" }));

    expect(await screen.findByRole("heading", { name: "Watch next" })).toBeInTheDocument();
    expect(screen.getByText("4% complete - resume at 1 min")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Resume" })).toHaveAttribute(
      "href",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    expect(container.textContent).not.toContain("nako_bpt_fixture");
    expect(container.textContent).not.toContain("/sources/");
    expect(container.textContent).not.toContain("fingerprint");
  });

  it("removes watched fixture items from Continue Watching after returning home", async () => {
    window.history.pushState(
      null,
      "",
      "/media/watch/item-episode-1?source_id=source-episode-1-alt",
    );
    const dataSource = createFixtureMediaDataSource();
    const factory = vi.fn(() => dataSource) satisfies MediaDataSourceFactory;

    render(
      <App
        dataSource={emptyAdminDataSource()}
        initialMediaConnection={{ mode: "fixture" }}
        mediaDataSourceFactory={factory}
      />,
    );

    const player = await screen.findByLabelText("Pilot player");
    fireEvent.play(player);
    setMediaTiming(player, 1439, 1440);
    fireEvent.ended(player);

    await waitFor(() => {
      return expect(
        dataSource.getUserPlaybackState("item-episode-1").then((result) => result.value.state),
      ).resolves.toEqual(expect.objectContaining({ watched: true }));
    });

    fireEvent.click(screen.getByRole("link", { name: "Home" }));

    expect(await screen.findByRole("heading", { name: "Watch next" })).toBeInTheDocument();
    expect(screen.getByText("No active playback state")).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Resume" })).not.toBeInTheDocument();
  });
});

function emptyAdminDataSource() {
  return {
    async load() {
      throw new Error("admin data source should not load for media routes");
    },
  };
}

async function findRecentlyAddedSection() {
  const heading = await screen.findByRole("heading", { name: "Recently Added" });
  const section = heading.closest("section");
  if (!section) {
    throw new Error("Recently Added section was not rendered");
  }
  return section;
}

const multiplePathTicket: BrowserPlaybackTicketResponse = {
  expires_at: "2026-05-26T12:00:00Z",
  item_id: "item-episode-1",
  mode: "direct",
  playback_session_id: null,
  source_id: "source-episode-1-alt",
  urls: [
    {
      content_type: "video/mp4",
      kind: "stream",
      supports_range_requests: true,
      url: "https://fixture.nako.test/stream?path=primary&ticket=nako_bpt_primary",
    },
    {
      content_type: "video/mp4",
      kind: "stream",
      supports_range_requests: true,
      url: "https://fixture.nako.test/stream?path=secondary&ticket=nako_bpt_secondary",
    },
  ],
};

function multiplePathTicketForSource(sourceId: string): BrowserPlaybackTicketResponse {
  return {
    expires_at: "2026-05-26T12:00:00Z",
    item_id: "item-episode-1",
    mode: "direct",
    playback_session_id: null,
    source_id: sourceId,
    urls: [
      {
        content_type: "video/mp4",
        kind: "stream",
        supports_range_requests: true,
        url: `https://fixture.nako.test/stream?path=${sourceId}-primary&ticket=nako_bpt_${sourceId}_primary`,
      },
      {
        content_type: "video/mp4",
        kind: "stream",
        supports_range_requests: true,
        url: `https://fixture.nako.test/stream?path=${sourceId}-secondary&ticket=nako_bpt_${sourceId}_secondary`,
      },
    ],
  };
}

const hlsThenDirectTicket: BrowserPlaybackTicketResponse = {
  expires_at: "2026-05-26T12:00:00Z",
  item_id: "item-episode-1",
  mode: "hls",
  playback_session_id: "playback-session-hls-fixture",
  source_id: "source-episode-1-alt",
  urls: [
    {
      content_type: "application/vnd.apple.mpegurl",
      kind: "playlist",
      supports_range_requests: false,
      url: "https://fixture.nako.test/stream/hls/playlist.m3u8?ticket=nako_bpt_hls_primary",
    },
    {
      content_type: "video/mp4",
      kind: "stream",
      supports_range_requests: true,
      url: "https://fixture.nako.test/stream?path=mp4-fallback&ticket=nako_bpt_mp4_fallback",
    },
  ],
};

const hlsPlaylistTicket: BrowserPlaybackTicketResponse = {
  expires_at: "2026-05-26T12:00:00Z",
  item_id: "item-episode-1",
  mode: "hls",
  playback_session_id: "playback-session-hls-fixture",
  source_id: "source-episode-1-alt",
  urls: [
    {
      content_type: "application/vnd.apple.mpegurl",
      kind: "playlist",
      supports_range_requests: false,
      url: "https://fixture.nako.test/stream/hls/playlist.m3u8?ticket=nako_bpt_hls",
    },
  ],
};

function setMediaTiming(player: HTMLElement, currentTimeSeconds: number, durationSeconds: number) {
  Object.defineProperty(player, "currentTime", {
    configurable: true,
    value: currentTimeSeconds,
    writable: true,
  });
  Object.defineProperty(player, "duration", {
    configurable: true,
    value: durationSeconds,
  });
}
