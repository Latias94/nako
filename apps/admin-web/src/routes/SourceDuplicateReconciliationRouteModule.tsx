import type { AdminDataSource } from "../adminApi/dataSource";
import {
  SourceDuplicateReconciliationRoutePage,
} from "../features/items/SourceDuplicateReconciliationRoutePage";
import type { SourceDuplicateReconciliationSearch } from "../features/items/SourceDuplicateReconciliationPage";
import { RouteI18n } from "./RouteI18n";

export function SourceDuplicateReconciliationRouteModule({
  dataSource,
  itemId,
  onSearchChange,
  search,
  sourceId,
}: {
  dataSource: AdminDataSource;
  itemId: string;
  onSearchChange(next: Partial<SourceDuplicateReconciliationSearch>): void;
  search: SourceDuplicateReconciliationSearch;
  sourceId: string;
}) {
  return (
    <RouteI18n namespace="sourceDuplicate">
      <SourceDuplicateReconciliationRoutePage
        dataSource={dataSource}
        itemId={itemId}
        onSearchChange={onSearchChange}
        search={search}
        sourceId={sourceId}
      />
    </RouteI18n>
  );
}
