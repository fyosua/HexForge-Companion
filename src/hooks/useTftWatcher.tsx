import { useEffect, useState, useCallback } from "react";

export interface TftWindowInfo {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type TftConnectionState = "searching" | "attached" | "detached";

interface TftStatePayload {
  state: {
    state: string;
    x?: number;
    y?: number;
    width?: number;
    height?: number;
  };
}

/**
 * Hook that listens for TFT process attach/detach events from the
 * Rust backend's process watcher thread.
 *
 * On mount, starts listening for `tft-attached` and `tft-detached`
 * events. Returns the current connection state + window geometry.
 */
export function useTftWatcher() {
  const [state, setState] = useState<TftConnectionState>("searching");
  const [windowInfo, setWindowInfo] = useState<TftWindowInfo | null>(null);

  const handleAttached = useCallback((payload: TftStatePayload) => {
    const s = payload?.state;
    setState("attached");
    if (s && typeof s.x === "number") {
      setWindowInfo({
        x: s.x ?? 0,
        y: s.y ?? 0,
        width: s.width ?? 1920,
        height: s.height ?? 1080,
      });
    }
  }, []);

  const handleDetached = useCallback(() => {
    setState("detached");
    setWindowInfo(null);
  }, []);

  useEffect(() => {
    let unlistenAttach: (() => void) | undefined;
    let unlistenDetach: (() => void) | undefined;

    async function setup() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        unlistenAttach = await listen<TftStatePayload>(
          "tft-attached",
          (event) => handleAttached(event.payload)
        );

        unlistenDetach = await listen<TftStatePayload>(
          "tft-detached",
          () => handleDetached()
        );
      } catch {
        // Not running inside Tauri — skip event listening
        setState("detached");
      }
    }

    setup();

    return () => {
      unlistenAttach?.();
      unlistenDetach?.();
    };
  }, [handleAttached, handleDetached]);

  return { state, windowInfo };
}

/**
 * Simple visual indicator for TFT connection status.
 */
export function TftConnectionBadge({ state }: { state: TftConnectionState }) {
  const colors: Record<TftConnectionState, string> = {
    searching: "#888",
    attached: "#44ff44",
    detached: "#ff4444",
  };

  const labels: Record<TftConnectionState, string> = {
    searching: "Searching for TFT...",
    attached: "TFT Detected",
    detached: "TFT Not Running",
  };

  return (
    <div
      className="hex-inline-indicator"
      style={{ fontSize: 10, opacity: 0.7 }}
    >
      <span
        className={`hex-pulse-dot ${
          state === "attached" ? "hex-pulse-green" : state === "searching" ? "hex-pulse-off" : "hex-pulse-red"
        }`}
        style={{ background: colors[state] }}
      />
      {labels[state]}
    </div>
  );
}