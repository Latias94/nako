export type ApiSourceMode = "live" | "fixture";

export type ApiLoadResult<T> = {
  value: T;
  source: ApiSourceMode;
  error?: string;
};

export type FetchLike = (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;

export function liveResult<T>(value: T): ApiLoadResult<T> {
  return {
    source: "live",
    value,
  };
}

export function fixtureResult<T>(value: T, error?: string): ApiLoadResult<T> {
  return {
    error,
    source: "fixture",
    value,
  };
}

export function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.trim().replace(/\/+$/, "");
}

export function apiErrorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}
