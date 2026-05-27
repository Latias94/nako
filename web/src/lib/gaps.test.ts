import { describe, expect, it } from "vitest";

import { capabilityGaps, isRouteActive, surfaceDefinitions } from "@/lib/navigation";

describe("surface navigation model", () => {
  it("keeps release surfaces separated", () => {
    expect(surfaceDefinitions.media.nav[0]?.to).toBe("/media");
    expect(surfaceDefinitions.admin.nav[0]?.to).toBe("/admin");
    expect(surfaceDefinitions.media.nav.some((item) => item.to.startsWith("/admin"))).toBe(false);
    expect(surfaceDefinitions.admin.nav.some((item) => item.to.startsWith("/media"))).toBe(false);
  });

  it("records the current Nako capability gaps to track", () => {
    expect(capabilityGaps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ area: "Desktop native playback core" }),
        expect.objectContaining({ area: "Management Context Links route matrix" }),
      ]),
    );
  });

  it("matches active routes without highlighting sibling routes", () => {
    const mediaHome = surfaceDefinitions.media.nav[0]!;
    const mediaLibraries = surfaceDefinitions.media.nav[1]!;

    expect(isRouteActive("/media", mediaHome)).toBe(true);
    expect(isRouteActive("/media/libraries", mediaHome)).toBe(false);
    expect(isRouteActive("/media/libraries", mediaLibraries)).toBe(true);
    expect(isRouteActive("/media/libraries/movies", mediaLibraries)).toBe(true);
    expect(isRouteActive("/media/library", mediaLibraries)).toBe(false);
  });
});
