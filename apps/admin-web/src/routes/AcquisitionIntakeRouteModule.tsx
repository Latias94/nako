import type { AdminDataSource } from "../adminApi/dataSource";
import {
  AcquisitionIntakePage,
  type AcquisitionIntakeSearch,
} from "../features/acquisition/AcquisitionIntakePage";
import { RouteI18n } from "./RouteI18n";

export function AcquisitionIntakeRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<AcquisitionIntakeSearch>): void;
  search: AcquisitionIntakeSearch;
}) {
  return (
    <RouteI18n namespace="acquisition">
      <AcquisitionIntakePage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
