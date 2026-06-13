import type { AdminDataSource } from "../adminApi/dataSource";
import {
  StorageStagingPage,
  type StorageStagingSearch,
} from "../features/storage/StorageStagingPage";
import { RouteI18n } from "./RouteI18n";

export function StorageStagingRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<StorageStagingSearch>): void;
  search: StorageStagingSearch;
}) {
  return (
    <RouteI18n namespace="storage">
      <StorageStagingPage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
