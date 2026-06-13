import type { AdminDataSource } from "../adminApi/dataSource";
import {
  CatalogBrowsePage,
  type CatalogSearch,
} from "../features/catalog/CatalogBrowsePage";
import { RouteI18n } from "./RouteI18n";

export function CatalogBrowseRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<CatalogSearch>): void;
  search: CatalogSearch;
}) {
  return (
    <RouteI18n namespace="catalogBrowse">
      <CatalogBrowsePage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
