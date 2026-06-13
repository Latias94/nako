import type { AdminDataSource } from "../adminApi/dataSource";
import {
  GeneratedArtifactReviewPage,
  type GeneratedArtifactReviewSearch,
} from "../features/automation/GeneratedArtifactReviewPage";
import { RouteI18n } from "./RouteI18n";

export function GeneratedArtifactReviewRouteModule({
  artifactId,
  dataSource,
  onSearchChange,
  search,
}: {
  artifactId: string;
  dataSource: AdminDataSource;
  onSearchChange(next: Partial<GeneratedArtifactReviewSearch>): void;
  search: GeneratedArtifactReviewSearch;
}) {
  return (
    <RouteI18n namespace="generatedArtifactReview">
      <GeneratedArtifactReviewPage
        artifactId={artifactId}
        dataSource={dataSource}
        onSearchChange={onSearchChange}
        search={search}
      />
    </RouteI18n>
  );
}
