import type { AdminDataSource } from "../adminApi/dataSource";
import {
  PlaybackSupportPage,
  type PlaybackSupportSearch,
} from "../features/playback/PlaybackSupportPage";
import { RouteI18n } from "./RouteI18n";

export function PlaybackSupportRouteModule({
  dataSource,
  search,
}: {
  dataSource: AdminDataSource;
  search: PlaybackSupportSearch;
}) {
  return (
    <RouteI18n namespace="playbackSupport">
      <PlaybackSupportPage dataSource={dataSource} search={search} />
    </RouteI18n>
  );
}
