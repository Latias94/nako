import type { AdminDataSource } from "../adminApi/dataSource";
import {
  PlaybackSessionsPage,
  type PlaybackSessionsSearch,
} from "../features/playback/PlaybackSessionsPage";
import { RouteI18n } from "./RouteI18n";

const playbackSessionsI18nNamespaces = ["playback", "playbackSupport"] as const;

export function PlaybackSessionsRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<PlaybackSessionsSearch>): void;
  search: PlaybackSessionsSearch;
}) {
  return (
    <RouteI18n namespace={playbackSessionsI18nNamespaces}>
      <PlaybackSessionsPage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
