import type { AdminDataSource } from "../adminApi/dataSource";
import { LibrariesPage } from "../features/libraries/LibrariesPage";
import { RouteI18n } from "./RouteI18n";

export function LibrariesRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return (
    <RouteI18n namespace="libraries">
      <LibrariesPage dataSource={dataSource} />
    </RouteI18n>
  );
}
