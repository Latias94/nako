import { MediaItemsPage } from "../surfaces/media/MediaPages";
import type { MediaPageSearch, MediaSearchChange } from "../surfaces/media/MediaCore";

export function MediaItemsRouteModule({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaPageSearch>;
  search: MediaPageSearch;
}) {
  return <MediaItemsPage onSearchChange={onSearchChange} search={search} />;
}
