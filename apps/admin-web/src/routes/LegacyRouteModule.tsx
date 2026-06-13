import type { AdminDataSource } from "../adminApi/dataSource";
import { LegacyDashboard } from "../legacy/LegacyDashboard";

export function LegacyRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return <LegacyDashboard dataSource={dataSource} />;
}
