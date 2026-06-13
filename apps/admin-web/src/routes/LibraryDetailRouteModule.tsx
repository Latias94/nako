import type { AdminDataSource } from "../adminApi/dataSource";
import { LibraryDetailPage } from "../features/libraries/LibraryDetailPage";
import { RouteI18n } from "./RouteI18n";

const libraryDetailI18nNamespaces = ["libraryDetail", "libraries"] as const;

export function LibraryDetailRouteModule({
  dataSource,
  libraryId,
}: {
  dataSource: AdminDataSource;
  libraryId: string;
}) {
  return (
    <RouteI18n namespace={libraryDetailI18nNamespaces}>
      <LibraryDetailPage dataSource={dataSource} libraryId={libraryId} />
    </RouteI18n>
  );
}
