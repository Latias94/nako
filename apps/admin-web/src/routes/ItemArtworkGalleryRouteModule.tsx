import type { AdminDataSource } from "../adminApi/dataSource";
import {
  ItemArtworkGalleryPage,
  type ItemArtworkGallerySearch,
} from "../features/items/ItemArtworkGalleryPage";
import { RouteI18n } from "./RouteI18n";

export function ItemArtworkGalleryRouteModule({
  dataSource,
  itemId,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  itemId: string;
  onSearchChange(next: Partial<ItemArtworkGallerySearch>): void;
  search: ItemArtworkGallerySearch;
}) {
  return (
    <RouteI18n namespace="itemArtwork">
      <ItemArtworkGalleryPage
        dataSource={dataSource}
        itemId={itemId}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
