import { describe, expect, it, vi } from "vitest";

const dataSourceModule = vi.hoisted(() => {
  const load = vi.fn(async () => ({ marker: "console-data" }));
  const loadJobs = vi.fn(async (query?: object) => ({
    query,
    source: "live",
    value: { jobs: [] },
  }));
  const registerAddonManifestJson = vi.fn(async (manifestJson: string) => ({
    manifestJson,
    status: "registered",
  }));
  const createAdminDataSource = vi.fn(() => ({
    load,
    loadJobs,
    registerAddonManifestJson,
  }));

  return {
    createAdminDataSource,
    load,
    loadJobs,
    registerAddonManifestJson,
  };
});

vi.mock("./dataSource", () => ({
  createAdminDataSource: dataSourceModule.createAdminDataSource,
}));

import { createLazyAdminDataSource } from "./lazyDataSource";

describe("createLazyAdminDataSource", () => {
  it("loads the concrete Admin data source on first method call", async () => {
    const dataSource = createLazyAdminDataSource({ baseUrl: "/admin" });

    expect(dataSourceModule.createAdminDataSource).not.toHaveBeenCalled();
    expect(typeof dataSource.load).toBe("function");
    expect(typeof dataSource.registerAddonManifestJson).toBe("function");
    expect(dataSource.loadAddonTaskRun).toBeUndefined();
    expect(dataSource.previewAddonManifestJson).toBeUndefined();

    await expect(dataSource.load()).resolves.toEqual({
      marker: "console-data",
    });
    expect(dataSourceModule.createAdminDataSource).toHaveBeenCalledTimes(1);
    expect(dataSourceModule.createAdminDataSource).toHaveBeenCalledWith({
      baseUrl: "/admin",
    });

    await dataSource.loadJobs?.({ limit: 1 });
    await expect(
      dataSource.registerAddonManifestJson?.('{"id":"addon-subtitle-lab"}'),
    ).resolves.toEqual({
      manifestJson: '{"id":"addon-subtitle-lab"}',
      status: "registered",
    });

    expect(dataSourceModule.createAdminDataSource).toHaveBeenCalledTimes(1);
    expect(dataSourceModule.loadJobs).toHaveBeenCalledWith({ limit: 1 });
    expect(dataSourceModule.registerAddonManifestJson).toHaveBeenCalledWith(
      '{"id":"addon-subtitle-lab"}',
    );
  });
});
