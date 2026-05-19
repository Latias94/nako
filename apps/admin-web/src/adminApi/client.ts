import type { AdminOverviewResponse } from "./types";

export type AdminApiClientOptions = {
  baseUrl?: string;
  token?: string;
  fetcher?: typeof fetch;
};

export class AdminApiClient {
  private readonly baseUrl: string;
  private readonly token?: string;
  private readonly fetcher: typeof fetch;

  constructor(options: AdminApiClientOptions = {}) {
    this.baseUrl = normalizeBaseUrl(options.baseUrl);
    this.token = options.token;
    this.fetcher = options.fetcher ?? fetch;
  }

  async getOverview(): Promise<AdminOverviewResponse> {
    return this.getJson<AdminOverviewResponse>("/admin/v1/overview");
  }

  private async getJson<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      headers: this.headers(),
    });

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`);
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

function normalizeBaseUrl(baseUrl: string | undefined) {
  const value = baseUrl?.trim() || "";

  if (value.endsWith("/")) {
    return value.slice(0, -1);
  }

  return value;
}
