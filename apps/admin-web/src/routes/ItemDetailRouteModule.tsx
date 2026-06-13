import type { AdminDataSource } from "../adminApi/dataSource";
import { ItemDetailPage } from "../features/items/ItemDetailPage";
import { RouteI18n } from "./RouteI18n";

export function ItemDetailRouteModule({
  dataSource,
  itemId,
}: {
  dataSource: AdminDataSource;
  itemId: string;
}) {
  return (
    <RouteI18n namespace="itemDetail">
      <ItemDetailPage dataSource={dataSource} itemId={itemId} />
    </RouteI18n>
  );
}
