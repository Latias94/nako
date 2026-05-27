import {
  NAKO_ADMIN_ROUTES,
  type AdminAccessSummaryResponse,
  type AdminJobListResponse,
  type AdminJobsQuery,
  type AdminOverviewResponse,
} from "@/api/admin/generated/contract";
import {
  fixtureAdminAccessSummary,
  fixtureAdminJobs,
  fixtureAdminOverview,
} from "@/api/admin/fixtures";
import {
  apiErrorMessage,
  fixtureResult,
  liveResult,
  normalizeBaseUrl,
  type ApiLoadResult,
  type FetchLike,
} from "@/api/shared";

export type AdminApiConnection =
  | {
      mode: "fixture";
    }
  | {
      mode: "live";
      baseUrl: string;
      token?: string;
    };

export type AdminApi = {
  readonly source: "fixture" | "live";
  getOverview(): Promise<ApiLoadResult<AdminOverviewResponse>>;
  getAccessSummary(): Promise<ApiLoadResult<AdminAccessSummaryResponse>>;
  getJobs(query?: AdminJobsQuery): Promise<ApiLoadResult<AdminJobListResponse>>;
};

export function createAdminApi(
  connection: AdminApiConnection = { mode: "fixture" },
  fetcher: FetchLike = globalThis.fetch.bind(globalThis),
): AdminApi {
  if (connection.mode === "fixture") {
    return createFixtureAdminApi();
  }

  return createLiveAdminApi(connection, fetcher);
}

function createLiveAdminApi(
  connection: Extract<AdminApiConnection, { mode: "live" }>,
  fetcher: FetchLike,
): AdminApi {
  const client = new AdminHttpClient(connection, fetcher);

  return {
    source: "live",
    async getOverview() {
      return loadAdminSection(
        () => client.getJson(NAKO_ADMIN_ROUTES.overview),
        fixtureAdminOverview,
      );
    },
    async getAccessSummary() {
      return loadAdminSection(
        () => client.getJson(NAKO_ADMIN_ROUTES.accessSummary),
        fixtureAdminAccessSummary,
      );
    },
    async getJobs(query = {}) {
      return loadAdminSection(
        () => client.getJson(withQuery(NAKO_ADMIN_ROUTES.jobs, query)),
        fixtureAdminJobs,
      );
    },
  };
}

function createFixtureAdminApi(): AdminApi {
  return {
    source: "fixture",
    async getOverview() {
      return fixtureResult(fixtureAdminOverview);
    },
    async getAccessSummary() {
      return fixtureResult(fixtureAdminAccessSummary);
    },
    async getJobs() {
      return fixtureResult(fixtureAdminJobs);
    },
  };
}

async function loadAdminSection<T>(loader: () => Promise<T>, fallback: T): Promise<ApiLoadResult<T>> {
  try {
    return liveResult(await loader());
  } catch (error: unknown) {
    return fixtureResult(fallback, apiErrorMessage(error, "Admin API request failed"));
  }
}

class AdminHttpClient {
  private readonly baseUrl: string;
  private readonly fetcher: FetchLike;
  private readonly token?: string;

  constructor(connection: Extract<AdminApiConnection, { mode: "live" }>, fetcher: FetchLike) {
    this.baseUrl = normalizeBaseUrl(connection.baseUrl);
    this.fetcher = fetcher;
    this.token = connection.token;
  }

  async getJson<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      headers: this.headers(),
    });

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`);
    }

    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.toLowerCase().includes("application/json")) {
      throw new Error("Admin API request returned non-JSON content");
    }

    return (await response.json()) as T;
  }

  private headers(): HeadersInit {
    if (!this.token) {
      return {};
    }

    return {
      Authorization: `Bearer ${this.token}`,
    };
  }
}

function withQuery(path: string, query: object): string {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== null && value !== "") {
      params.set(key, String(value));
    }
  }

  const suffix = params.toString();
  return suffix ? `${path}?${suffix}` : path;
}
