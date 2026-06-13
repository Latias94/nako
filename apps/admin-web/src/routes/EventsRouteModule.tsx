import type { AdminDataSource } from "../adminApi/dataSource";
import {
  EventsPage,
  type EventsSearch,
} from "../features/events/EventsPage";
import { RouteI18n } from "./RouteI18n";

export function EventsRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<EventsSearch>): void;
  search: EventsSearch;
}) {
  return (
    <RouteI18n namespace="events">
      <EventsPage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
