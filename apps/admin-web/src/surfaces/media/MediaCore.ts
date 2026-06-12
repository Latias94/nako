import { useEffect, useState } from "react";

import type { MediaLoadResult, MediaWebDataSource } from "./mediaDataSource";

export type MediaPageSearch = {
  limit: number;
  offset: number;
};

export type MediaSearchRouteSearch = MediaPageSearch & {
  facet?: string;
  q?: string;
};

export type MediaItemSearch = {
  source_id?: string;
};

export type MediaSearchChange<TSearch> = (next: Partial<TSearch>) => void;

export type MediaAsyncState<T> = {
  error: string | null;
  loading: boolean;
  value: T | null;
};

export function useMediaLoad<T>(
  dataSource: MediaWebDataSource | null,
  load: (dataSource: MediaWebDataSource) => Promise<MediaLoadResult<T>>,
  deps: readonly unknown[] = [],
): MediaAsyncState<T> {
  const [state, setState] = useState<MediaAsyncState<T>>({
    error: null,
    loading: Boolean(dataSource),
    value: null,
  });

  useEffect(() => {
    let cancelled = false;
    if (!dataSource) {
      setState({ error: null, loading: false, value: null });
      return;
    }

    setState((current) => ({ ...current, error: null, loading: true }));
    load(dataSource)
      .then((result) => {
        if (!cancelled) {
          setState({ error: result.error ?? null, loading: false, value: result.value });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({
            error: error instanceof Error ? error.message : "Media request failed",
            loading: false,
            value: null,
          });
        }
      });

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dataSource, ...deps]);

  return state;
}

export function formatBytes(value: number | null) {
  if (!value) {
    return "size unknown";
  }

  const gib = value / 1024 / 1024 / 1024;
  return `${gib.toFixed(gib >= 10 ? 0 : 1)} GiB`;
}

export function formatRuntimeMinutes(value: number | null) {
  return value ? `${value} min` : "Runtime unknown";
}

export function formatRuntimeMs(value: number | null | undefined) {
  return value ? `${Math.round(value / 60_000)} min` : "duration unknown";
}
