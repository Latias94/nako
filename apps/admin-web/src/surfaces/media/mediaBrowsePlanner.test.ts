import { describe, expect, it, vi } from "vitest";

import { createFixtureMediaDataSource } from "./mediaDataSource";
import {
  loadLibraryItems,
  loadMediaItems,
  loadMediaSearch,
  mediaItemsBrowseActiveFilterCount,
  mediaItemsBrowseFacetChange,
  mediaItemsBrowseHasPaginationDelta,
  mediaItemsBrowseLimitChange,
  mediaItemsBrowseOrderChange,
  mediaItemsBrowseResetSearch,
  mediaItemsBrowseSearchChange,
  mediaItemsBrowseSortChange,
  mediaItemsBrowseSortValue,
  mediaItemsBrowseWatchStateChange,
  mediaItemsBrowseWatchStateValue,
  mediaSearchSubmitChange,
} from "./mediaBrowsePlanner";

describe("media browse planner", () => {
  it("derives browse filter state and reset behavior", () => {
    expect(
      mediaItemsBrowseActiveFilterCount({
        facet: "kind:movie",
        limit: 20,
        offset: 0,
        q: undefined,
        sort: "title",
        watch_state: "in_progress",
      }),
    ).toBe(3);
    expect(
      mediaItemsBrowseHasPaginationDelta({
        limit: 20,
        offset: 0,
      }),
    ).toBe(false);
    expect(
      mediaItemsBrowseHasPaginationDelta({
        limit: 1,
        offset: 0,
      }),
    ).toBe(true);
    expect(mediaItemsBrowseResetSearch()).toEqual({
      facet: undefined,
      limit: 20,
      offset: 0,
      order: undefined,
      q: undefined,
      sort: undefined,
      watch_state: undefined,
    });
    expect(mediaItemsBrowseSearchChange("")).toEqual({ offset: 0, q: undefined });
    expect(mediaItemsBrowseFacetChange("kind:movie")).toEqual({ facet: "kind:movie", offset: 0 });
    expect(mediaItemsBrowseSortChange("title")).toEqual({ offset: 0, sort: "title" });
    expect(mediaItemsBrowseOrderChange("asc")).toEqual({ offset: 0, order: "asc" });
    expect(mediaItemsBrowseWatchStateChange("in_progress")).toEqual({
      offset: 0,
      watch_state: "in_progress",
    });
    expect(mediaItemsBrowseLimitChange("10", 20)).toEqual({ limit: 10, offset: 0 });
    expect(mediaSearchSubmitChange("  Rain  ")).toEqual({ offset: 0, q: "Rain" });
    expect(mediaItemsBrowseSortValue("last_played")).toBe("last_played");
    expect(mediaItemsBrowseWatchStateValue("any")).toBeUndefined();
  });

  it("routes media item browse queries through search or list as appropriate", async () => {
    const dataSource = createFixtureMediaDataSource();
    const searchItems = vi.spyOn(dataSource, "searchItems");
    const listItems = vi.spyOn(dataSource, "listItems");

    await loadMediaItems(
      dataSource,
      {
        facet: "kind:movie",
        limit: 1,
        offset: 2,
        q: "Rain",
      },
      "Media items could not be loaded.",
    );
    await loadMediaItems(
      dataSource,
      {
        facet: "kind:movie",
        limit: 1,
        offset: 2,
        order: "asc",
        sort: "title",
        watch_state: "in_progress",
      },
      "Media items could not be loaded.",
    );

    expect(searchItems).toHaveBeenCalledWith({
      facet: "kind:movie",
      limit: 1,
      offset: 2,
      q: "Rain",
    });
    expect(listItems).toHaveBeenCalledWith({
      facet: "kind:movie",
      limit: 1,
      offset: 2,
      order: "asc",
      sort: "title",
      watch_state: "in_progress",
    });
  });

  it("builds the library and search query seams explicitly", async () => {
    const dataSource = createFixtureMediaDataSource();
    const listLibraryItems = vi.spyOn(dataSource, "listLibraryItems");
    const searchItems = vi.spyOn(dataSource, "searchItems");

    await loadLibraryItems(
      dataSource,
      "library-anime",
      {
        facet: "kind:movie",
        limit: 5,
        offset: 10,
        order: "desc",
        sort: "last_played",
        watch_state: "in_progress",
      },
      "Library items could not be loaded.",
    );
    await loadMediaSearch(
      dataSource,
      {
        facet: "kind:movie",
        limit: 5,
        offset: 10,
        q: "Rain",
      },
      "Search results could not be loaded.",
    );

    expect(listLibraryItems).toHaveBeenCalledWith("library-anime", {
      facet: "kind:movie",
      limit: 5,
      offset: 10,
      order: "desc",
      sort: "last_played",
      watch_state: "in_progress",
    });
    expect(searchItems).toHaveBeenCalledWith({
      facet: "kind:movie",
      limit: 5,
      offset: 10,
      q: "Rain",
    });
  });
});
