import { describe, expect, it } from "vitest";

import { capabilityGaps, surfaceDefinitions } from "@/lib/navigation";

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
});
