import {
  MediaSearchPage,
} from "../surfaces/media/MediaPages";
import type {
  MediaSearchChange,
  MediaSearchRouteSearch,
} from "../surfaces/media/MediaCore";

export function MediaSearchRouteModule({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaSearchRouteSearch>;
  search: MediaSearchRouteSearch;
}) {
  return (
    <MediaSearchPage
      onSearchChange={onSearchChange}
      search={search}
    />
  );
}
