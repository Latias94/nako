import type { AdminDataSource } from "../adminApi/dataSource";
import {
  AddonsPage,
  type AddonsSearch,
} from "../features/addons/AddonsPage";
import { RouteI18n } from "./RouteI18n";

export function AddonsRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<AddonsSearch>): void;
  search: AddonsSearch;
}) {
  return (
    <RouteI18n namespace="addons">
      <AddonsPage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
