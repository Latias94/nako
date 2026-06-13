import type { AdminDataSource } from "../adminApi/dataSource";
import { AccessPage } from "../features/access/AccessPage";
import { RouteI18n } from "./RouteI18n";

export function AccessRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return (
    <RouteI18n namespace="access">
      <AccessPage dataSource={dataSource} />
    </RouteI18n>
  );
}
