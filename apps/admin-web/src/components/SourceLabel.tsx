import { Badge } from "./ui/Badge";
import { useI18n } from "../i18n/I18nProvider";

export type SourceLabelMode = "live" | "hybrid" | "mock" | "planned";

export function SourceLabel({ source }: { source: SourceLabelMode }) {
  const { t } = useI18n();

  if (source === "live") {
    return <Badge tone="success">{t("source.live")}</Badge>;
  }

  if (source === "hybrid") {
    return <Badge tone="info">{t("source.hybrid")}</Badge>;
  }

  if (source === "planned") {
    return <Badge tone="warning">{t("source.planned")}</Badge>;
  }

  return <Badge tone="neutral">{t("source.mock")}</Badge>;
}
