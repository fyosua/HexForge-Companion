import { useEffect, useState, useCallback } from "react";
import { LegalFooter } from "./components/LegalFooter";
import { useAppUpdater, UpdateBadge } from "./hooks/useAppUpdater";
import { DisplayModeWarning } from "./components/DisplayModeWarning";
import { PlayerSearch } from "./components/PlayerSearch";
import { MatchHistory } from "./components/MatchHistory";
import { PlayerStats } from "./components/PlayerStats";
import { RankDisplay } from "./components/RankDisplay";
import { InGameIndicator } from "./components/InGameIndicator";
import { LeaderboardDisplay } from "./components/LeaderboardDisplay";
import { PlatformStatus } from "./components/PlatformStatus";
import { PinnedWidget } from "./components/PinnedWidget";
import { useTftWatcher, useWindowLabel } from "./hooks/useTftWatcher";
import "./App.css";

interface PlayerInfo {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_level: number;
  summoner_id: string;
}

interface RankInfo {
  tier?: string;
  rank?: string;
  league_points?: number;
  queue_type?: string;
}

function isTauri(): boolean {
  try { return !!(window as any).__TAURI__; }
  catch { return false; }
}

const PROXY_URL = window.location.origin;

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  }
  const res = await fetch(`${PROXY_URL}/api/${cmd.replace(/_/g, "-")}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args || {}),
  });
  // Check for HTML response (proxy error page) before JSON parse
  const contentType = res.headers.get("content-type") || "";
  if (!res.ok || contentType.includes("text/html")) {
    const text = await res.text();
    throw new Error(text.startsWith("<") ? `Server returned HTML (status ${res.status})` : text);
  }
  return res.json();
}

function App() {
  const [player, setPlayer] = useState<PlayerInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [inTauri, setInTauri] = useState(false);
  const [refreshLoading, setRefreshLoading] = useState(false);
  const [refreshCounter, setRefreshCounter] = useState(0);
  const [pinned, setPinned] = useState(false);
  const [pinnedRank, setPinnedRank] = useState("");
  const [pinnedInGame, setPinnedInGame] = useState(false);
  const [apiMode, setApiMode] = useState("---");
  const [dbPath, setDbPath] = useState("---");

  const { state: tftState } = useTftWatcher();
  const windowLabel = useWindowLabel();
  const updateInfo = useAppUpdater();
  const isOverlay = windowLabel === "overlay";
  const tftLoading = tftState === "searching";

  // Fetch API mode + DB path on mount (dashboard only)
  useEffect(() => {
    if (!inTauri) return;
    tauriInvoke<string>("get_api_mode").then(setApiMode).catch(() => {});
    tauriInvoke<string>("get_db_path").then(setDbPath).catch(() => {});
  }, [inTauri]);

  // Listen for clear-puuid event — reset player state after 30s detach
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    async function setup() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<{ reason: string }>("clear-puuid", () => {
          setPlayer(null);
          setError(null);
          setRefreshCounter(0);
          setPinned(false);
        });
      } catch { /* not in Tauri */ }
    }
    setup();
    return () => { unlisten?.(); };
  }, []);

  useEffect(() => {
    setInTauri(isTauri());
    if (!isTauri()) return;

    let invoke: (cmd: string) => void;
    import("@tauri-apps/api/core").then((mod) => {
      invoke = mod.invoke;
    }).catch(() => {});

    const onMouseOver = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest(".hex-hud-interactive")) {
        invoke?.("hud_bounds_enter");
      }
    };
    const onMouseOut = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest(".hex-hud-interactive")) {
        invoke?.("hud_bounds_leave");
      }
    };
    document.addEventListener("mouseover", onMouseOver);
    document.addEventListener("mouseout", onMouseOut);
    return () => {
      document.removeEventListener("mouseover", onMouseOver);
      document.removeEventListener("mouseout", onMouseOut);
    };
  }, []);

  // Poll rank + game status for pin widget
  useEffect(() => {
    if (!pinned || !player) return;
    const interval = setInterval(async () => {
      try {
        const [ranks, gameStatus] = await Promise.all([
          tauriInvoke<RankInfo[]>("get_player_rank"),
          tauriInvoke<{ in_game: boolean }>("get_active_game_status"),
        ]);
        const tft = (ranks ?? []).find(
          (r) => r.queue_type === "RANKED_TFT" || r.queue_type === "RANKED_TFT_TURBO"
        );
        if (tft?.tier) setPinnedRank(`${tft.tier} ${tft.rank ?? ""} ${tft.league_points ?? 0}LP`);
        setPinnedInGame(gameStatus?.in_game ?? false);
      } catch { /* poll silently */ }
    }, 30000);
    return () => clearInterval(interval);
  }, [pinned, player]);

  // ── Handlers ─────────────────────────────────────────

  const handleRefreshMatches = async () => {
    setRefreshLoading(true);
    try {
      await tauriInvoke<any>("refresh_matches", { count: 20 });
      setRefreshCounter((c) => c + 1);
    } catch (err) {
      setError(`Refresh error: ${err}`);
    } finally {
      setRefreshLoading(false);
    }
  };

  const handlePlayerResolved = useCallback((p: PlayerInfo) => {
    setPlayer(p);
    setError(null);
    handleRefreshMatches();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handlePin = () => {
    if (!player) return;
    setPinned(!pinned);
    tauriInvoke<RankInfo[]>("get_player_rank").then((ranks) => {
      const tft = (ranks ?? []).find(
        (r) => r.queue_type === "RANKED_TFT" || r.queue_type === "RANKED_TFT_TURBO"
      );
      if (tft?.tier) setPinnedRank(`${tft.tier} ${tft.rank ?? ""} ${tft.league_points ?? 0}LP`);
    }).catch(() => {});
    tauriInvoke<{ in_game: boolean }>("get_active_game_status").then((s) => {
      setPinnedInGame(s?.in_game ?? false);
    }).catch(() => {});
  };

  const handleLaunchOverlay = () => {
    tauriInvoke("show_overlay");
  };

  const handleBackToDashboard = () => {
    tauriInvoke("show_dashboard");
  };

  // On mount (overlay or dashboard), restore player state from backend
  useEffect(() => {
    if (!inTauri) return;
    tauriInvoke<PlayerInfo | null>("get_active_player").then((p) => {
      if (p) setPlayer(p);
    }).catch(() => {});
  }, [inTauri]);

  // ── Render helpers ────────────────────────────────────

  /** Dashboard TFT status banner */
  function TftStatusBanner() {
    if (tftLoading) {
      return (
        <div className="hex-tft-banner hex-tft-loading">
          <div className="hex-loading-pulse" style={{ width: 200, height: 32 }} />
        </div>
      );
    }
    if (tftState === "attached") {
      return (
        <div className="hex-tft-banner hex-tft-attached">
          <div className="hex-tft-banner-content">
            <span className="hex-tft-icon">🟢</span>
            <span className="hex-tft-text">TFT Detected — Ready to Overlay</span>
            <button className="hex-launch-overlay-btn" onClick={handleLaunchOverlay}>
              🚀 Launch Overlay
            </button>
          </div>
        </div>
      );
    }
    return (
      <div className="hex-tft-banner hex-tft-waiting">
        <div className="hex-tft-banner-content">
          <span className="hex-tft-icon">🎮</span>
          <span className="hex-tft-text">Waiting for TFT...</span>
          <span className="hex-tft-sub">Launch Teamfight Tactics to enable the overlay</span>
        </div>
      </div>
    );
  }

  /** Dashboard header with mode + DB path */
  function DashboardHeader() {
    return (
      <header className="hex-header">
        <div className="hex-header-left">
          <h1>HexForge Companion</h1>
          {inTauri && (
            <span className="hex-header-mode">
              <span className="hex-header-label">Mode:</span> {apiMode}
            </span>
          )}
        </div>
        {inTauri && (
          <div className="hex-header-right" title={dbPath}>
            <span className="hex-header-db">
              📁 {dbPath.split("/").pop()}
            </span>
          </div>
        )}
      </header>
    );
  }

  /** Overlay "Back to Dashboard" button (small, top-right, visible on hover) */
  function OverlayBackButton() {
    const [hovered, setHovered] = useState(false);
    return (
      <div
        className={`hex-overlay-topbar ${hovered ? "hex-overlay-topbar-visible" : ""}`}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
      >
        <button className="hex-back-to-dashboard-btn" onClick={handleBackToDashboard}>
          ← Back to Dashboard
        </button>
      </div>
    );
  }

  /** Overlay idle state — shown when no player is searched */
  function OverlayIdleState() {
    return (
      <div className="hex-overlay-idle">
        <div className="hex-overlay-idle-icon">🔍</div>
        <div className="hex-overlay-idle-text">Search a player to begin</div>
        <div className="hex-overlay-idle-hint">
          Go to the Dashboard to search for a Riot ID
        </div>
      </div>
    );
  }

  // ── Main render ───────────────────────────────────────

  // ── OVERLAY VIEW ──────────────────────────────────────
  if (isOverlay) {
    return (
      <div className="app-container overlay-view">
        <OverlayBackButton />

        {pinned && player && (
          <PinnedWidget
            data={player}
            rankLabel={pinnedRank}
            inGame={pinnedInGame}
            onUnpin={() => setPinned(false)}
          />
        )}

        <main className="hex-main-overlay">
          {player ? (
            <div className="hex-dashboard overlay-dashboard">
              <div className="hex-profile-header">
                <div className="hex-profile-name">
                  {player.game_name}#{player.tag_line}
                  <span className="hex-profile-level">Lv.{player.summoner_level}</span>
                </div>
                <button
                  className="hex-pin-btn"
                  onClick={handlePin}
                  title={pinned ? "Unpin" : "Pin as compact widget"}
                >
                  {pinned ? "📌 Unpin" : "📌 Pin"}
                </button>
              </div>

              <InGameIndicator />
              <PlatformStatus />
              <LeaderboardDisplay />
              <RankDisplay key={`rank-${refreshCounter}`} />
              <PlayerStats key={`stats-${refreshCounter}`} />

              <div className="hex-refresh-bar">
                <button
                  className="hex-refresh-btn"
                  onClick={handleRefreshMatches}
                  disabled={refreshLoading}
                >
                  {refreshLoading ? (
                    <span className="hex-soft-refresh">
                      <span className="hex-spinner" />
                      Syncing...
                    </span>
                  ) : (
                    "Refresh Matches"
                  )}
                </button>
              </div>

              <MatchHistory key={`history-${refreshCounter}`} />
            </div>
          ) : (
            <OverlayIdleState />
          )}
        </main>

        <LegalFooter />
        <UpdateBadge update={updateInfo} />
      </div>
    );
  }

  // ── DASHBOARD VIEW ────────────────────────────────────
  return (
    <div className="app-container dashboard-view">
      {!inTauri && (
        <div className="hex-browser-banner">
          <span>⚡ Browser preview — mock API on port 1421.</span>
          <a className="hex-download-link" href="http://raspberrypi.local:1421/download/" target="_blank">⬇ Download .exe</a>
        </div>
      )}

      {pinned && player && (
        <PinnedWidget
          data={player}
          rankLabel={pinnedRank}
          inGame={pinnedInGame}
          onUnpin={() => setPinned(false)}
        />
      )}

      {!pinned && <DashboardHeader />}

      <main className="hex-main" style={pinned ? { paddingTop: 8 } : undefined}>
        <TftStatusBanner />

        {tftState !== "attached" && !pinned && (
          <DisplayModeWarning />
        )}

        <PlayerSearch onPlayerResolved={handlePlayerResolved} onError={setError} />
        {error && <div className="hex-error">{error}</div>}

        {player && !pinned && (
          <div className="hex-dashboard">
            <div className="hex-profile-header">
              <div className="hex-profile-name">
                {player.game_name}#{player.tag_line}
                <span className="hex-profile-level">Lv.{player.summoner_level}</span>
              </div>
              {tftState === "attached" && (
                <button
                  className="hex-pin-btn"
                  onClick={handlePin}
                  title="Pin as overlay widget"
                >
                  📌 Pin Overlay
                </button>
              )}
            </div>

            <InGameIndicator />
            <PlatformStatus />
            <LeaderboardDisplay />
            <RankDisplay key={`rank-${refreshCounter}`} />
            <PlayerStats key={`stats-${refreshCounter}`} />

            <div className="hex-refresh-bar">
              <button
                className="hex-refresh-btn"
                onClick={handleRefreshMatches}
                disabled={refreshLoading}
              >
                {refreshLoading ? (
                  <span className="hex-soft-refresh">
                    <span className="hex-spinner" />
                    Syncing...
                  </span>
                ) : (
                  "Refresh Matches"
                )}
              </button>
            </div>

            <MatchHistory key={`history-${refreshCounter}`} />
          </div>
        )}

        {player && pinned && (
          <div style={{ fontSize: 10, color: "#666", textAlign: "center", marginTop: 4 }}>
            <button
              className="hex-pin-btn"
              onClick={handlePin}
              style={{ fontSize: 10, padding: "2px 8px" }}
            >
              📌 Unpin & Open Dashboard
            </button>
          </div>
        )}
      </main>

      <LegalFooter />
      <UpdateBadge update={updateInfo} />
    </div>
  );
}

export default App;