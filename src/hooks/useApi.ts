import { useEffect, useState, useRef, useCallback } from "react";

/** Check if running inside Tauri WebView */
function isTauri(): boolean {
  try {
    return !!(window as any).__TAURI__;
  } catch {
    return false;
  }
}

const PROXY_URL = "http://raspberrypi.local:1421";

type UseApiResult<T> = {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => void;
};

/**
 * Shared data-fetching hook with AbortController cleanup.
 *
 * Prevents StrictMode double-render race conditions by aborting
 * in-flight requests when the component unmounts or deps change.
 *
 * @param cmd    Tauri IPC command name (e.g. "get_player_stats")
 * @param args   Optional arguments object
 * @param deps   Dependency array — re-fetches when values change
 */
export function useApi<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
  deps: unknown[] = []
): UseApiResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const fetchIdRef = useRef(0);

  const fetchData = useCallback(async () => {
    const fetchId = ++fetchIdRef.current;

    setLoading(true);
    setError(null);

    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<T>(cmd, args);
        // Only update state if this is still the latest request
        if (fetchId === fetchIdRef.current) {
          setData(result);
        }
      } else {
        const proxyPath = `/api/${cmd.replace(/_/g, "-")}`;
        const controller = new AbortController();
        // Store controller so it aborts on unmount
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (window as any).__abortControllers = (window as any).__abortControllers || [];
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (window as any).__abortControllers.push(controller);

        const res = await fetch(`${PROXY_URL}${proxyPath}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(args || {}),
          signal: controller.signal,
        });

        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const idx = (window as any).__abortControllers.indexOf(controller);
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        if (idx >= 0) (window as any).__abortControllers.splice(idx, 1);

        if (!res.ok) {
          const errBody = await res.json().catch(() => ({ error: res.statusText }));
          throw new Error(errBody.error || `HTTP ${res.status}`);
        }

        const result = await res.json();
        // Only update state if this is still the latest request
        if (fetchId === fetchIdRef.current) {
          setData(result);
        }
      }
    } catch (err: unknown) {
      if (err instanceof DOMException && err.name === "AbortError") {
        return; // Component unmounted, ignore
      }
      if (fetchId === fetchIdRef.current) {
        setError(String(err));
      }
    } finally {
      if (fetchId === fetchIdRef.current) {
        setLoading(false);
      }
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cmd, JSON.stringify(args), ...deps]);

  useEffect(() => {
    fetchData();
    return () => {
      // Bump fetchId so in-flight callbacks are ignored
      fetchIdRef.current++;
    };
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook for data that needs periodic polling (e.g. active game status).
 * Includes AbortController cleanup to prevent stale updates on re-render.
 */
export function usePollingApi<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
  intervalMs: number = 30000
): UseApiResult<T> {
  const result = useApi<T>(cmd, args, []);
  const { refetch } = result;
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    intervalRef.current = setInterval(refetch, intervalMs);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [refetch, intervalMs]);

  return result;
}