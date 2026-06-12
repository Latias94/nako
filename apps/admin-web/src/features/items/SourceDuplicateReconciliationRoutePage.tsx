import { useMemo } from "react";

import type { AdminDataSource } from "../../adminApi/dataSource";
import { useI18n } from "../../i18n/I18nProvider";
import {
  SourceDuplicateReconciliationPage,
  type SourceDuplicateReconciliationSearch,
} from "./SourceDuplicateReconciliationPage";
import { createSourceDuplicateReconciliationDataAdapter } from "./sourceDuplicateReconciliationData";

export type SourceDuplicateReconciliationRoutePageProps = {
  dataSource: AdminDataSource;
  itemId: string;
  onSearchChange(next: Partial<SourceDuplicateReconciliationSearch>): void;
  search: SourceDuplicateReconciliationSearch;
  sourceId: string;
};

export function SourceDuplicateReconciliationRoutePage({
  dataSource,
  itemId,
  onSearchChange,
  search,
  sourceId,
}: SourceDuplicateReconciliationRoutePageProps) {
  const { t } = useI18n();
  const dataAdapter = useMemo(
    () =>
      createSourceDuplicateReconciliationDataAdapter(dataSource, {
        applyUnavailableMessage: t("sourceDuplicate.applyUnavailable"),
        planUnavailableMessage: t("sourceDuplicate.planUnavailable"),
      }),
    [dataSource, t],
  );

  return (
    <SourceDuplicateReconciliationPage
      dataAdapter={dataAdapter}
      itemId={itemId}
      onSearchChange={onSearchChange}
      search={search}
      sourceId={sourceId}
    />
  );
}
