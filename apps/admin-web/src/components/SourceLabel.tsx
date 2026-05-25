import { Badge } from "./ui/Badge";
import type { DataSourceMode } from "../adminApi/dataSource";

export function SourceLabel({ source }: { source: DataSourceMode }) {
  if (source === "live") {
    return <Badge tone="success">Live Admin API</Badge>;
  }

  if (source === "hybrid") {
    return <Badge tone="info">Live + fallback</Badge>;
  }

  if (source === "planned") {
    return <Badge tone="warning">Planned</Badge>;
  }

  return <Badge tone="neutral">Mock fallback</Badge>;
}
