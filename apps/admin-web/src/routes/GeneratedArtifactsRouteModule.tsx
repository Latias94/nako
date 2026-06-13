import type { AdminDataSource } from "../adminApi/dataSource";
import {
  GeneratedArtifactsPage,
  type GeneratedArtifactsSearch,
} from "../features/automation/GeneratedArtifactsPage";
import { RouteI18n } from "./RouteI18n";

export function GeneratedArtifactsRouteModule({
  dataSource,
  onSearchChange,
  search,
}: {
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<GeneratedArtifactsSearch>): void;
  search: GeneratedArtifactsSearch;
}) {
  return (
    <RouteI18n namespace="generatedArtifacts">
      <GeneratedArtifactsPage
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
