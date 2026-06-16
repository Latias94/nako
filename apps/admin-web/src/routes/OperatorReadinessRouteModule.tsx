import type { AdminDataSource } from "../adminApi/dataSource";
import { OperatorReadinessPage } from "../features/overview/OperatorReadinessPage";
import { RouteI18n } from "./RouteI18n";

export function OperatorReadinessRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return (
    <RouteI18n namespace={["overview", "operatorReadiness"]}>
      <OperatorReadinessPage dataSource={dataSource} />
    </RouteI18n>
  );
}
