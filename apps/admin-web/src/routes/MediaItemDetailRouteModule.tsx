import { MediaItemDetailPage } from "../surfaces/media/MediaItemDetailPage";
import type {
  MediaItemSearch,
  MediaSearchChange,
} from "../surfaces/media/MediaCore";

export function MediaItemDetailRouteModule({
  itemId,
  onSearchChange,
  search,
}: {
  itemId: string;
  onSearchChange: MediaSearchChange<MediaItemSearch>;
  search: MediaItemSearch;
}) {
  return (
    <MediaItemDetailPage
      itemId={itemId}
      onSearchChange={onSearchChange}
      search={search}
    />
  );
}
