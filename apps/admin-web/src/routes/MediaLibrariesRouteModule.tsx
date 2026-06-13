import {
  MediaLibrariesPage,
} from "../surfaces/media/MediaPages";
import type {
  MediaPageSearch,
  MediaSearchChange,
} from "../surfaces/media/MediaCore";

export function MediaLibrariesRouteModule({
  onSearchChange,
  search,
}: {
  onSearchChange: MediaSearchChange<MediaPageSearch>;
  search: MediaPageSearch;
}) {
  return (
    <MediaLibrariesPage
      onSearchChange={onSearchChange}
      search={search}
    />
  );
}
