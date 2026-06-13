import type { AdminDataSource } from "../adminApi/dataSource";
import { OverviewPage } from "../features/overview/OverviewPage";
import { RouteI18n } from "./RouteI18n";

export function OverviewRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return (
    <RouteI18n namespace="overview">
      <OverviewPage dataSource={dataSource} />
    </RouteI18n>
  );
}
