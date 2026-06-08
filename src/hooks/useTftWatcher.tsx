import { useEffect, useState, useCallback, useRef } from "react";

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

// —— Overlay window manager ———————————————————————————————————

/**
 * Resize and reposition the Tauri overlay window to match the
 * TFT game window geometry, then ensure it's visible.
 */
async function snapToGameWindow(info: TftWindowInfo): Promise<void> {
  try {
    const { getCurrentWindow, LogicalSize, LogicalPosition } =
      await import("@tauri-apps/api/window");

    const win = getCurrentWindow();

    // Move to match TFT window position
    await win.setPosition(new LogicalPosition(info.x, info.y));

    // Resize to match TFT window dimensions
    await win.setSize(new LogicalSize(info.width, info.height));

    // Ensure visible
    await win.show();
  } catch {
    // Not in Tauri or API unavailable — silent
  }
}

/**
 * Hide the overlay window when TFT is not running.
 */
async function hideOverlay(): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();
    await win.hide();
  } catch {
    // Not in Tauri — silent
  }
}

// —— Hook —————————————————————————————————————————

/**
 * Hook that listens for TFT process attach/detach events from the
 * Rust backend's process watcher thread.
 *
 * On mount, starts listening for \`tft-attached\` and \`tft-detached\`
 * events. When attached with geometry info, automatically resizes
 * and repositions the Tauri overlay to match the TFT game window.
 * When detached, hides the overlay.
 *
 * Returns the current connection state + window geometry.
 */
export function useTftWatcher(options?: { autoManageWindow?: boolean }) {
  const autoManage = options?.autoManageWindow ?? true;
  const [state, setState] = useState<TftConnectionState>("searching");
  const [windowInfo, setWindowInfo] = useState<TftWindowInfo | null>(null);
  const lastAttachedRef = useRef<TftWindowInfo | null>(null);

  const handleAttached = useCallback(
    (payload: TftStatePayload) => {
      const s = payload?.state;
      setState("attached");
      if (s && typeof s.x === "number") {
        const info: TftWindowInfo = {
          x: s.x ?? 0,
          y: s.y ?? 0,
          width: s.width ?? 1920,
          height: s.height ?? 1080,
        };
        setWindowInfo(info);

        // Only snap if geometry actually changed
        const prev = lastAttachedRef.current;
        if (
          autoManage &&
          (!prev ||
            prev.x !== info.x ||
            prev.y !== info.y ||
            prev.width !== info.width ||
            prev.height !== info.height)
        ) {
          lastAttachedRef.current = info;
          snapToGameWindow(info);
        }
      }
    },
    [autoManage]
  );

  const handleDetached = useCallback(() => {
    setState("detached");
    setWindowInfo(null);
    lastAttachedRef.current = null;

    if (autoManage) {
      hideOverlay();
    }
  }, [autoManage]);

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

        // Listen for cache-clear events (PUUID cleared after 30s detach)
        await listen<{ reason: string }>("clear-puuid", () => {
          setWindowInfo(null);
        });
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
