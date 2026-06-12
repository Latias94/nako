import type { ReactNode } from "react";
import { createContext, useContext, useMemo, useState } from "react";

import type {
  MediaConnection,
  MediaDataSourceFactory,
  MediaWebDataSource,
} from "./mediaDataSource";

type MediaSessionContextValue = {
  connection: MediaConnection | null;
  dataSource: MediaWebDataSource | null;
  connect(connection: MediaConnection): Promise<void>;
  clearConnection(): void;
  connectionError: string | null;
  connecting: boolean;
};

const MediaSessionContext = createContext<MediaSessionContextValue | null>(null);

export function MediaSessionProvider({
  children,
  dataSourceFactory,
  initialConnection = null,
}: {
  children: ReactNode;
  dataSourceFactory?: MediaDataSourceFactory;
  initialConnection?: MediaConnection | null;
}) {
  const [connection, setConnection] = useState<MediaConnection | null>(initialConnection);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const resolvedDataSourceFactory = dataSourceFactory ?? createLazyMediaDataSource;
  const dataSource = useMemo(
    () => (connection ? resolvedDataSourceFactory(connection) : null),
    [connection, resolvedDataSourceFactory],
  );

  async function connect(nextConnection: MediaConnection) {
    setConnecting(true);
    setConnectionError(null);
    try {
      const nextDataSource = resolvedDataSourceFactory(nextConnection);
      await nextDataSource.checkConnection();
      setConnection(nextConnection);
    } catch (error: unknown) {
      setConnectionError(error instanceof Error ? error.message : "Connection failed");
    } finally {
      setConnecting(false);
    }
  }

  function clearConnection() {
    setConnection(null);
    setConnectionError(null);
  }

  return (
    <MediaSessionContext.Provider
      value={{
        clearConnection,
        connect,
        connection,
        connectionError,
        connecting,
        dataSource,
      }}
    >
      {children}
    </MediaSessionContext.Provider>
  );
}

export function useMediaSession() {
  const value = useContext(MediaSessionContext);
  if (!value) {
    throw new Error("Media session context is missing");
  }
  return value;
}

function createLazyMediaDataSource(connection: MediaConnection): MediaWebDataSource {
  let dataSourcePromise: Promise<MediaWebDataSource> | null = null;
  const loadDataSource = () => {
    dataSourcePromise ??= import("./mediaDataSource").then((module) =>
      module.createMediaWebDataSource(connection),
    );
    return dataSourcePromise;
  };

  return {
    source: connection.mode === "fixture" ? "fixture" : "live",
    label: connection.mode === "fixture" ? "Fixture mode" : "Live Public Client API",
    async checkConnection() {
      return (await loadDataSource()).checkConnection();
    },
    async listLibraries(page) {
      return (await loadDataSource()).listLibraries(page);
    },
    async getLibrary(libraryId) {
      return (await loadDataSource()).getLibrary(libraryId);
    },
    async listLibrarySources(libraryId, page) {
      return (await loadDataSource()).listLibrarySources(libraryId, page);
    },
    async listItems(page) {
      return (await loadDataSource()).listItems(page);
    },
    async searchItems(query) {
      return (await loadDataSource()).searchItems(query);
    },
    async getItem(itemId) {
      return (await loadDataSource()).getItem(itemId);
    },
    async getPlaybackDecision(sourceId, capabilities) {
      return (await loadDataSource()).getPlaybackDecision(sourceId, capabilities);
    },
    async createBrowserPlaybackTicket(sourceId, body) {
      return (await loadDataSource()).createBrowserPlaybackTicket(sourceId, body);
    },
    async getUserPlaybackState(itemId) {
      return (await loadDataSource()).getUserPlaybackState(itemId);
    },
    async updateUserPlaybackProgress(itemId, body) {
      return (await loadDataSource()).updateUserPlaybackProgress(itemId, body);
    },
    async setUserWatchedState(itemId, body) {
      return (await loadDataSource()).setUserWatchedState(itemId, body);
    },
    async listContinueWatching(page) {
      return (await loadDataSource()).listContinueWatching(page);
    },
  };
}
