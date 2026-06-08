import { useEffect, useState } from "react";

export interface UpdateInfo {
  version: string;
  available: boolean;
  downloading: boolean;
}

/**
 * Hook that checks for app updates on startup.
 *
 * On mount, calls the Tauri updater plugin's `check()` to
 * see if a new version is available. If so, downloads and
 * installs automatically with progress tracking.
 *
 * Returns current update state for UI rendering.
 */
export function useAppUpdater() {
  const [update, setUpdate] = useState<UpdateInfo>({
    version: "",
    available: false,
    downloading: false,
  });

  useEffect(() => {
    let cancelled = false;

    async function checkForUpdate() {
      try {
        const { check } = await import("@tauri-apps/plugin-updater");
        const { relaunch } = await import("@tauri-apps/plugin-process");

        const result = await check();

        if (cancelled) return;
        if (!result) {
          return;
        }

        setUpdate({
          version: result.version,
          available: true,
          downloading: true,
        });

        await result.downloadAndInstall((event) => {
          switch (event.event) {
            case "Started":
              console.log(`Downloading update v${result.version}...`);
              break;
            case "Progress":
              break;
            case "Finished":
              console.log("Download complete \u2014 relaunching...");
              break;
          }
        });

        if (cancelled) return;
        await relaunch();
      } catch {
        // Not in Tauri, updater unavailable, or no update \u2014 silent
      }
    }

    const timer = setTimeout(checkForUpdate, 5000);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  return update;
}

/**
 * Simple update notification badge component.
 * Renders nothing when no update is available.
 */
export function UpdateBadge({ update }: { update: UpdateInfo }) {
  if (!update.available) return null;

  return (
    <div
      className="hex-update-badge"
      style={{
        position: "fixed",
        bottom: 8,
        right: 8,
        background: "rgba(255, 200, 0, 0.9)",
        color: "#000",
        padding: "4px 10px",
        borderRadius: 4,
        fontSize: 11,
        fontWeight: 600,
        zIndex: 9999,
        pointerEvents: "auto",
      }}
    >
      {update.downloading
        ? `\u2b07 Updating to v${update.version}...`
        : `\u2b07 Update v${update.version} available`}
    </div>
  );
}
