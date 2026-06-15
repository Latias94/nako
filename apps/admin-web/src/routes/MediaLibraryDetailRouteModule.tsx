import {
  MediaLibraryDetailPage,
} from "../surfaces/media/MediaPages";
import type {
  MediaItemsBrowseSearch,
  MediaSearchChange,
} from "../surfaces/media/MediaCore";

export function MediaLibraryDetailRouteModule({
  libraryId,
  onSearchChange,
  search,
}: {
  libraryId: string;
  onSearchChange: MediaSearchChange<MediaItemsBrowseSearch>;
  search: MediaItemsBrowseSearch;
}) {
  return (
    <MediaLibraryDetailPage
      libraryId={libraryId}
      onSearchChange={onSearchChange}
      search={search}
    />
  );
}
