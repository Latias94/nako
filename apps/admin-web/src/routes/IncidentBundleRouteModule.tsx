import type { AdminDataSource } from "../adminApi/dataSource";
import { IncidentBundlePage } from "../features/diagnostics/IncidentBundlePage";
import { RouteI18n } from "./RouteI18n";

export function IncidentBundleRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return (
    <RouteI18n namespace="incidentBundle">
      <IncidentBundlePage dataSource={dataSource} />
    </RouteI18n>
  );
}
