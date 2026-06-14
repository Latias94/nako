import { describe, expect, it, vi } from "vitest";

import {
  mockSourceDuplicateReconciliationApply,
  mockSourceDuplicateReconciliationPlan,
} from "../../adminApi/mockData";
import { createSourceDuplicateReconciliationDataAdapter } from "./sourceDuplicateReconciliationData";

describe("source duplicate reconciliation data adapter", () => {
  it("delegates plan loading to the broad Admin data source", async () => {
    const loadSourceDuplicateReconciliationPlan = vi.fn(async (libraryId, sourceId, query) => ({
      value: mockSourceDuplicateReconciliationPlan(libraryId, sourceId),
      source: "live" as const,
      error: undefined,
      query,
    }));
    const adapter = createSourceDuplicateReconciliationDataAdapter(
      { loadSourceDuplicateReconciliationPlan },
      {
        applyUnavailableMessage: "Apply unavailable",
        planUnavailableMessage: "Plan unavailable",
      },
    );

    const result = await adapter.loadPlan("library/unsafe id", "source/unsafe id", {
      limit: 5,
      offset: 10,
    });

    expect(loadSourceDuplicateReconciliationPlan).toHaveBeenCalledWith(
      "library/unsafe id",
      "source/unsafe id",
      { limit: 5, offset: 10 },
    );
    expect(result).toMatchObject({
      source: "live",
      value: {
        library_id: "library/unsafe id",
        source_id: "source/unsafe id",
      },
    });
  });

  it("returns a redaction-safe mock plan when plan loading is unavailable", async () => {
    const adapter = createSourceDuplicateReconciliationDataAdapter(
      {},
      {
        applyUnavailableMessage: "Apply unavailable",
        planUnavailableMessage: "Plan unavailable",
      },
    );

    expect(adapter.createFallbackPlan("library-anime", "source-unknown-1")).toEqual(
      mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1"),
    );
    await expect(adapter.loadPlan("library-anime", "source-unknown-1")).resolves.toMatchObject({
      source: "mock",
      value: mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1"),
      error: "Plan unavailable",
    });
  });

  it("delegates apply suggestion without mock success fallback", async () => {
    const applySourceDuplicateReconciliation = vi.fn(
      async (libraryId: string, sourceId: string, duplicateSourceId: string, expectedAction: string) =>
        mockSourceDuplicateReconciliationApply(
          libraryId,
          sourceId,
          duplicateSourceId,
          expectedAction === "confirm_suggested" ? "confirmed" : "suggested",
        ),
    );
    const adapter = createSourceDuplicateReconciliationDataAdapter(
      { applySourceDuplicateReconciliation },
      {
        applyUnavailableMessage: "Apply unavailable",
        planUnavailableMessage: "Plan unavailable",
      },
    );

    await expect(
      adapter.applySuggestion(
        "library/unsafe id",
        "source/unsafe id",
        "duplicate/source id",
        "suggest_relationship",
      ),
    ).resolves.toMatchObject({
      library_id: "library/unsafe id",
      source_id: "source/unsafe id",
      duplicate_source_id: "duplicate/source id",
      relationship_status: "suggested",
    });
    expect(applySourceDuplicateReconciliation).toHaveBeenCalledWith(
      "library/unsafe id",
      "source/unsafe id",
      "duplicate/source id",
      "suggest_relationship",
    );

    await expect(
      adapter.applySuggestion(
        "library/unsafe id",
        "source/unsafe id",
        "duplicate/source id",
        "confirm_suggested",
      ),
    ).resolves.toMatchObject({
      relationship_status: "confirmed",
      applied_action: "confirm_suggested",
    });
    expect(applySourceDuplicateReconciliation).toHaveBeenLastCalledWith(
      "library/unsafe id",
      "source/unsafe id",
      "duplicate/source id",
      "confirm_suggested",
    );
  });

  it("rejects apply suggestion when the live mutation is unavailable", async () => {
    const adapter = createSourceDuplicateReconciliationDataAdapter(
      {},
      {
        applyUnavailableMessage: "Apply unavailable",
        planUnavailableMessage: "Plan unavailable",
      },
    );

    await expect(
      adapter.applySuggestion(
        "library-anime",
        "source-unknown-1",
        "source-unknown-2",
        "suggest_relationship",
      ),
    ).rejects.toThrow("Apply unavailable");
  });
});
