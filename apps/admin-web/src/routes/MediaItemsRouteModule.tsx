import { MediaItemsPage } from "../surfaces/media/MediaPages";
import type {
  MediaItemsBrowseSearch,
  MediaSearchChange,
} from "../surfaces/media/MediaCore";

export function MediaItemsRouteModule({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaItemsBrowseSearch>;
  search: MediaItemsBrowseSearch;
}) {
  return <MediaItemsPage onSearchChange={onSearchChange} search={search} />;
}
