import type { AdminDataSource } from "../adminApi/dataSource";
import {
  JobsPage,
  type JobsSearch,
} from "../features/jobs/JobsPage";
import { RouteI18n } from "./RouteI18n";

export function JobsRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<JobsSearch>): void;
  search: JobsSearch;
}) {
  return (
    <RouteI18n namespace="jobs">
      <JobsPage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
