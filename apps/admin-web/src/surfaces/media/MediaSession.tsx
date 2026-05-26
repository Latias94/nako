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
  dataSourceFactory: MediaDataSourceFactory;
  initialConnection?: MediaConnection | null;
}) {
  const [connection, setConnection] = useState<MediaConnection | null>(initialConnection);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const dataSource = useMemo(
    () => (connection ? dataSourceFactory(connection) : null),
    [connection, dataSourceFactory],
  );

  async function connect(nextConnection: MediaConnection) {
    setConnecting(true);
    setConnectionError(null);
    try {
      const nextDataSource = dataSourceFactory(nextConnection);
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
