import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { useEffect, useMemo } from "react";

import { bootstrapDesktopConnection } from "@/api/runtime";
import { router } from "@/router";

export function App() {
  const queryClient = useMemo(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      }),
    [],
  );

  useEffect(() => {
    let cancelled = false;

    void bootstrapDesktopConnection().then((bootstrap) => {
      if (!cancelled && bootstrap?.profile) {
        void queryClient.invalidateQueries();
      }
    });

    return () => {
      cancelled = true;
    };
  }, [queryClient]);

  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
