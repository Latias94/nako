import type { AdminDataSource } from "../adminApi/dataSource";
import {
  ManagedArtworkMaintenancePage,
  type ManagedArtworkMaintenanceSearch,
} from "../features/artwork/ManagedArtworkMaintenancePage";
import { RouteI18n } from "./RouteI18n";

export function ManagedArtworkMaintenanceRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<ManagedArtworkMaintenanceSearch>): void;
  search: ManagedArtworkMaintenanceSearch;
}) {
  return (
    <RouteI18n namespace="artworkMaintenance">
      <ManagedArtworkMaintenancePage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
