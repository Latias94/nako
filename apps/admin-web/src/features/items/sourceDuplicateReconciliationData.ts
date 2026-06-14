import type { AdminDataSource, AdminSectionResult } from "../../adminApi/dataSource";
import { mockSourceDuplicateReconciliationPlan } from "../../adminApi/mockData";
import type {
  AdminSourceDuplicateReconciliationApplyResponse,
  AdminSourceDuplicateReconciliationApplyExpectedAction,
  AdminSourceDuplicateReconciliationPlanQuery,
  AdminSourceDuplicateReconciliationPlanResponse,
} from "../../adminApi/types";

type SourceDuplicateReconciliationDataSource = Pick<
  AdminDataSource,
  "applySourceDuplicateReconciliation" | "loadSourceDuplicateReconciliationPlan"
>;

export type SourceDuplicateReconciliationDataAdapter = {
  createFallbackPlan(
    libraryId: string,
    sourceId: string,
  ): AdminSourceDuplicateReconciliationPlanResponse;
  loadPlan(
    libraryId: string,
    sourceId: string,
    query?: AdminSourceDuplicateReconciliationPlanQuery,
  ): Promise<AdminSectionResult<AdminSourceDuplicateReconciliationPlanResponse>>;
  applySuggestion(
    libraryId: string,
    sourceId: string,
    duplicateSourceId: string,
    expectedAction: AdminSourceDuplicateReconciliationApplyExpectedAction,
  ): Promise<AdminSourceDuplicateReconciliationApplyResponse>;
};

export function createSourceDuplicateReconciliationDataAdapter(
  dataSource: SourceDuplicateReconciliationDataSource,
  options: { applyUnavailableMessage: string; planUnavailableMessage: string },
): SourceDuplicateReconciliationDataAdapter {
  return {
    createFallbackPlan(libraryId, sourceId) {
      return mockSourceDuplicateReconciliationPlan(libraryId, sourceId);
    },
    async loadPlan(libraryId, sourceId, query = {}) {
      if (!dataSource.loadSourceDuplicateReconciliationPlan) {
        return {
          value: mockSourceDuplicateReconciliationPlan(libraryId, sourceId),
          source: "mock",
          error: options.planUnavailableMessage,
        };
      }

      return dataSource.loadSourceDuplicateReconciliationPlan(libraryId, sourceId, query);
    },
    async applySuggestion(libraryId, sourceId, duplicateSourceId, expectedAction) {
      if (!dataSource.applySourceDuplicateReconciliation) {
        throw new Error(options.applyUnavailableMessage);
      }

      return dataSource.applySourceDuplicateReconciliation(
        libraryId,
        sourceId,
        duplicateSourceId,
        expectedAction,
      );
    },
  };
}
