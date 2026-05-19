import { AdminApiClient, type AdminApiClientOptions } from "./client";
import { mockAdminConsoleData } from "./mockData";
import type { AdminConsoleData, DataSourceMode } from "./types";

export type { AdminConsoleData, DataSourceMode };

export type AdminDataSource = {
  load(): Promise<AdminConsoleData>;
};

export function createAdminDataSource(options: AdminApiClientOptions = {}): AdminDataSource {
  const client = new AdminApiClient(options);

  return {
    async load() {
      const overview = await client.getOverview();

      return {
        ...mockAdminConsoleData,
        overview,
        overviewSource: "live",
      };
    },
  };
}
