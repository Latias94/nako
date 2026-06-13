import {
  MediaLibraryDetailPage,
} from "../surfaces/media/MediaPages";
import type {
  MediaPageSearch,
  MediaSearchChange,
} from "../surfaces/media/MediaCore";

export function MediaLibraryDetailRouteModule({
  libraryId,
  onSearchChange,
  search,
}: {
  libraryId: string;
  onSearchChange: MediaSearchChange<MediaPageSearch>;
  search: MediaPageSearch;
}) {
  return (
    <MediaLibraryDetailPage
      libraryId={libraryId}
      onSearchChange={onSearchChange}
      search={search}
    />
  );
}
