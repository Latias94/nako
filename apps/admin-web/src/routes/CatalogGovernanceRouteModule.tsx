import type { AdminDataSource } from "../adminApi/dataSource";
import {
  CatalogGovernancePage,
  type CatalogGovernanceSearch,
} from "../features/catalog/CatalogGovernancePage";
import { RouteI18n } from "./RouteI18n";

export function CatalogGovernanceRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<CatalogGovernanceSearch>): void;
  search: CatalogGovernanceSearch;
}) {
  return (
    <RouteI18n namespace="catalogGovernance">
      <CatalogGovernancePage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
