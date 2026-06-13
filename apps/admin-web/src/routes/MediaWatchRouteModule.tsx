import { MediaWatchPage } from "../surfaces/media/MediaWatchPage";
import type {
  MediaItemSearch,
  MediaSearchChange,
} from "../surfaces/media/MediaCore";

export function MediaWatchRouteModule({
  itemId,
  onSearchChange,
  search,
}: {
  itemId: string;
  onSearchChange: MediaSearchChange<MediaItemSearch>;
  search: MediaItemSearch;
}) {
  return (
    <MediaWatchPage
      itemId={itemId}
      onSearchChange={onSearchChange}
      search={search}
    />
  );
}
