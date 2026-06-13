import type { AdminDataSource } from "../adminApi/dataSource";
import {
  CatalogGovernanceRepairPage,
  type CatalogGovernanceRepairSearch,
} from "../features/catalog/CatalogGovernanceRepairPage";
import { RouteI18n } from "./RouteI18n";

export function CatalogGovernanceRepairRouteModule({
  dataSource,
  itemId,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  itemId: string;
  onSearchChange(next: Partial<CatalogGovernanceRepairSearch>): void;
  search: CatalogGovernanceRepairSearch;
}) {
  return (
    <RouteI18n namespace="catalogGovernance">
      <CatalogGovernanceRepairPage
        dataSource={dataSource}
        itemId={itemId}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
