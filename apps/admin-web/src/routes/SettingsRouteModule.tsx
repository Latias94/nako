import type { AdminDataSource } from "../adminApi/dataSource";
import { SettingsPage } from "../features/settings/SettingsPage";
import { RouteI18n } from "./RouteI18n";

export function SettingsRouteModule({
  dataSource,
}: {
  dataSource: AdminDataSource;
}) {
  return (
    <RouteI18n namespace="settings">
      <SettingsPage dataSource={dataSource} />
    </RouteI18n>
  );
}
